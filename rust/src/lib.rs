pub mod audio_analysis;
pub mod collision_mask;
pub mod midi;
mod nodes;
pub mod render;
pub mod resource_paths;

use crate::audio_analysis::godot::{detect_bpm_aubio_ogg, detect_bpm_aubio_wav};
use crate::collision_mask::isp::{shift_polygon_vertices_down_by_pixels, update_polygons_with_scanline_alpha_buckets};
use crate::midi::godot::{
    make_note_on_off_event_dict_seconds, make_note_on_off_event_dict_ticks, write_samples_to_wav,
};
use crate::midi::util::{inject_program_change, prepare_events, process_midi_events_with_timing, render_sample_frame};
use collision_mask::godot::{
    generate_concave_collision_polygons_pixel_perfect, generate_convex_collision_polygons_pixel_perfect,
};
use godot::builtin::{PackedByteArray, PackedVector2Array, Vector2};
use godot::classes::file_access::ModeFlags;
use godot::classes::{FileAccess, Node2D};
use godot::global::godot_print;
use godot::prelude::{
    gdextension, godot_api, Array, Base, Dictionary, ExtensionLibrary, GString, GodotClass, PackedInt32Array,
};
use midly::{MidiMessage, Smf, TrackEventKind};
use rustysynth::{SoundFont, Synthesizer, SynthesizerSettings};
use std::io::Cursor;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}

#[derive(GodotClass)]
#[class(init, base=Node2D)]
struct RustUtil {
    #[base]
    base: Base<Node2D>,
}

const TARGET_CHANNEL: u8 = 0;
const PROGRAM: u8 = 0; //"Accordion" figure out a better way to do this

#[godot_api]
impl RustUtil {
    #[func]
    fn process_scanline(
        &self,
        delta_time: f32,
        screen_h: f32,
        vel_y: f32,
        depth: f32,
        scanline_alpha_buckets: PackedVector2Array,
        mut collision_polygons: Array<PackedVector2Array>,
        mut scroll_accum: f32,
        mut scanline_count_per_polygon: PackedInt32Array,
    ) -> Dictionary {
        let a = 0.5 * ((depth + 1.0) / (depth - 1.0)).ln();
        let b = 1.5 * (depth * ((depth + 1.0) / (depth - 1.0)).ln() - 2.0);
        let scale_y_top = a + b * -1.0;
        let speed_px_per_sec = vel_y * screen_h / (2.0 * scale_y_top);

        scroll_accum += speed_px_per_sec * delta_time;
        let rows = scroll_accum.floor() as i32;
        scroll_accum -= rows as f32;

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let total_millis = now.as_millis();
        let hours = (total_millis / 1000 / 60 / 60) % 24;
        let minutes = (total_millis / 1000 / 60) % 60;
        let seconds = (total_millis / 1000) % 60;
        let millis = total_millis % 1000;

        godot_print!(
            "INFO [SYSTEM_TIME: {:02}:{:02}:{:02}.{:03}] rows: {}, speed_px_per_sec: {}, delta_time: {:.3}, scroll_accum: {:.3}",
            hours,
            minutes,
            seconds,
            millis,
            rows,
            speed_px_per_sec,
            delta_time,
            scroll_accum
        );

        for row in 0..rows {
            godot_print!("row: {}", row);
            godot_print!("BEFORE");
            godot_print!("scanline_alpha_buckets:");
            for (i, vec) in scanline_alpha_buckets.as_slice().iter().enumerate() {
                godot_print!("  [{}] {:?}", i, vec);
            }
            godot_print!("collision_polygons:");
            for (i, poly) in collision_polygons.iter_shared().enumerate() {
                godot_print!("  [{}] {:?}", i, poly);
            }
            godot_print!("scanline_count_per_polygon:");
            for (i, count) in scanline_count_per_polygon.as_slice().iter().enumerate() {
                godot_print!("  [{}] {}", i, count);
            }

            shift_polygon_vertices_down_by_pixels(&mut collision_polygons, scanline_count_per_polygon.as_slice(), 1);
            update_polygons_with_scanline_alpha_buckets(
                &mut collision_polygons,
                &scanline_alpha_buckets,
                &mut scanline_count_per_polygon,
            );

            godot_print!("AFTER");
            godot_print!("scanline_alpha_buckets:");
            for (i, vec) in scanline_alpha_buckets.as_slice().iter().enumerate() {
                godot_print!("  [{}] {:?}", i, vec);
            }
            godot_print!("collision_polygons:");
            for (i, poly) in collision_polygons.iter_shared().enumerate() {
                godot_print!("  [{}] {:?}", i, poly);
            }
            godot_print!("scanline_count_per_polygon:");
            for (i, count) in scanline_count_per_polygon.as_slice().iter().enumerate() {
                godot_print!("  [{}] {}", i, count);
            }
        }

        let mut out = Dictionary::new();
        let _ = out.insert("scroll_accum", scroll_accum);
        let _ = out.insert("scanline_count_per_polygon", scanline_count_per_polygon);
        let _ = out.insert("collision_polygons", collision_polygons);
        out
    }

    #[func]
    pub fn compute_concave_collision_polygons(
        &self,
        raw_pixel_mask: PackedByteArray,
        image_width_pixels: i32,
        image_height_pixels: i32,
        tile_edge_length: i32,
    ) -> Array<PackedVector2Array> {
        // TODO: because godot complains about unsigned int r8 format, we just convert it here
        //  this is really gross to me and i think i could perhaps learn enough to argue for supporting
        //  R8_UINT in godot's RenderDevice.DataFormat <-> ImageFormat mapping in the source code.
        //  see: https://github.com/godotengine/godot/blob/6c9765d87e142e786f0190783f41a0250a835c99/servers/rendering/renderer_rd/storage_rd/texture_storage.cpp#L2281C1-L2664C1
        let pixel_data: Vec<u8> = raw_pixel_mask
            .to_vec()
            .into_iter()
            .map(|b| if b != 0 { 1 } else { 0 })
            .collect();
        let width: usize = image_width_pixels as usize;
        let height: usize = image_height_pixels as usize;
        let tile_size: usize = tile_edge_length as usize;
        let mut godot_polygons_array: Array<PackedVector2Array> = Array::new();
        let concave_polygons: Vec<Vec<Vector2>> =
            generate_concave_collision_polygons_pixel_perfect(&pixel_data, (width, height), tile_size);
        for concave_polygon in concave_polygons {
            let mut packed_polygon: PackedVector2Array = PackedVector2Array::new();
            for point in concave_polygon {
                packed_polygon.push(point);
            }
            godot_polygons_array.push(&packed_polygon);
        }
        godot_polygons_array
    }

    #[func]
    pub fn compute_convex_collision_polygons(
        &self,
        raw_pixel_mask: PackedByteArray,
        image_width_pixels: i32,
        image_height_pixels: i32,
        tile_edge_length: i32,
    ) -> Array<PackedVector2Array> {
        let pixel_data: Vec<u8> = raw_pixel_mask.to_vec();
        let width: usize = image_width_pixels as usize;
        let height: usize = image_height_pixels as usize;
        let tile_size: usize = tile_edge_length as usize;
        let mut godot_polygons_array: Array<PackedVector2Array> = Array::new();
        let convex_polygons: Vec<Vec<Vector2>> =
            generate_convex_collision_polygons_pixel_perfect(&pixel_data, (width, height), tile_size);

        for convex_polygon in convex_polygons {
            let mut packed_polygon: PackedVector2Array = PackedVector2Array::new();
            for point in convex_polygon {
                packed_polygon.push(point);
            }
            godot_polygons_array.push(&packed_polygon);
        }
        godot_polygons_array
    }

    #[func]
    pub fn detect_bpm_wav(&self, wav_file_path: GString) -> f32 {
        let wav_path = wav_file_path.to_string();
        let wav_file = FileAccess::open(&wav_path, ModeFlags::READ).unwrap();
        let wav_bytes = wav_file.get_buffer(wav_file.get_length() as i64).to_vec();
        detect_bpm_aubio_wav(&wav_bytes)
    }

    #[func]
    pub fn detect_bpm_ogg(&self, ogg_file_path: GString) -> f32 {
        let ogg_path = ogg_file_path.to_string();
        let ogg_file = FileAccess::open(&ogg_path, ModeFlags::READ).unwrap();
        let ogg_bytes = ogg_file.get_buffer(ogg_file.get_length() as i64).to_vec();
        detect_bpm_aubio_ogg(&ogg_bytes)
    }

    #[func]
    pub fn get_midi_note_on_off_event_buffer_ticks(&self, midi_file_path: GString) -> Dictionary {
        make_note_on_off_event_dict_ticks(&midi_file_path)
    }

    #[func]
    pub fn get_midi_note_on_off_event_buffer_seconds(&self, midi_file_path: GString) -> Dictionary {
        make_note_on_off_event_dict_seconds(&midi_file_path)
    }

    #[func]
    pub fn render_midi_to_sound_bytes_constant_time(
        &self,
        sample_rate: i32,
        midi_file_path: GString,
        sf2_file_path: GString,
    ) -> PackedByteArray {
        let sf2_path = sf2_file_path.to_string();
        let sf2_file = FileAccess::open(&sf2_path, ModeFlags::READ).unwrap();
        let sf2_bytes = sf2_file.get_buffer(sf2_file.get_length() as i64).to_vec();
        let mut sf2_cursor = Cursor::new(sf2_bytes);
        let soundfont = Arc::new(SoundFont::new(&mut sf2_cursor).unwrap());
        let mut synth = Synthesizer::new(&soundfont, &SynthesizerSettings::new(sample_rate)).unwrap();
        let midi_path = midi_file_path.to_string();
        let file = FileAccess::open(&midi_path, ModeFlags::READ).unwrap();
        let midi_file_bytes = file.get_buffer(file.get_length() as i64).to_vec();
        let smf = Smf::parse(&midi_file_bytes).unwrap();
        //TODO: make this more about the accordion, "program" is such a shitty name for an instrument in midi
        // i am not a fan of who ever made that naming decision, they better not be japanese
        let mut events = prepare_events(&smf);
        events = inject_program_change(events, TARGET_CHANNEL, PROGRAM);
        let mut audio = Vec::new();
        let mut active_notes = std::collections::HashSet::new();
        let mut time_cursor = 0.0;
        let step_secs = 1.0 / sample_rate as f64;
        process_midi_events_with_timing(events, &smf, |event_time, event, ch| {
            while time_cursor < event_time {
                audio.push(render_sample_frame(&mut synth));
                time_cursor += step_secs;
            }
            if let Some(channel) = ch {
                match event {
                    TrackEventKind::Midi { message, .. } => match message {
                        MidiMessage::NoteOn { key, vel } => {
                            let note = key.as_int() as i32;
                            let velocity = vel.as_int() as i32;
                            if velocity > 0 {
                                synth.note_on(channel as i32, note, velocity);
                                active_notes.insert((channel, note));
                            } else {
                                synth.note_off(channel as i32, note);
                                active_notes.remove(&(channel, note));
                            }
                        },
                        MidiMessage::NoteOff { key, .. } => {
                            let note = key.as_int() as i32;
                            synth.note_off(channel as i32, note);
                            active_notes.remove(&(channel, note));
                        },
                        _ => {},
                    },
                    _ => {},
                }
            }
        });
        while !active_notes.is_empty() {
            audio.push(render_sample_frame(&mut synth));
            time_cursor += step_secs;
        }
        write_samples_to_wav(sample_rate, audio)
        //TODO: look into vorbis later if needed, the rust support is very ugly with C libraries wrapped up and dragged in
    }
}
