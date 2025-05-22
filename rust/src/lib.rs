mod audio_analysis;
mod collision_mask;
mod midi;

use crate::audio_analysis::util::make_note_on_off_event_dict;
use crate::midi::midi::{
    inject_program_change, parse_midi_events_into_note_on_off_event_buffer_seconds,
    parse_midi_events_into_note_on_off_event_buffer_ticks, prepare_events,
    process_midi_events_with_timing, render_sample_frame, write_samples_to_wav,
};
use audio_analysis::util::{band_pass_filter, detect_bpm, extract_onset_times};
use collision_mask::util::{
    generate_concave_collision_polygons_pixel_perfect,
    generate_convex_collision_polygons_pixel_perfect,
};
use godot::builtin::{PackedByteArray, PackedVector2Array, Vector2};
use godot::classes::Node2D;
use godot::prelude::{
    gdextension, godot_api, godot_print, Array, Base, Dictionary, ExtensionLibrary, GString,
    GodotClass, PackedFloat32Array,
};
use midly::{MidiMessage, Smf, TrackEventKind};
use rustysynth::{SoundFont, Synthesizer, SynthesizerSettings};
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}

#[derive(GodotClass)]
#[class(init, base=Node2D)]
struct RustUtil {
    #[base]
    base: Base<Node2D>,
}

#[godot_api]
impl RustUtil {
    #[func]
    pub fn compute_concave_collision_polygons(
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
        let concave_polygons: Vec<Vec<Vector2>> = generate_concave_collision_polygons_pixel_perfect(
            &pixel_data,
            (width, height),
            tile_size,
        );
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
        let convex_polygons: Vec<Vec<Vector2>> = generate_convex_collision_polygons_pixel_perfect(
            &pixel_data,
            (width, height),
            tile_size,
        );

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
    pub fn detect_bpm(&self, path: GString) -> f32 {
        detect_bpm(path)
        //TODO: this is not actually accurate just an alternative, look at offline vs realtime later
        //_detect_bpm_accurate(path)
    }

    #[func]
    pub fn isolate_melody(path: GString, center_hz: f32) -> PackedFloat32Array {
        let infile = path.to_string();
        let out_str = if infile.to_lowercase().ends_with(".wav") {
            format!("{}__isolated.wav", &infile[..infile.len() - 4])
        } else {
            format!("{}__isolated.wav", infile)
        };
        let out_g = GString::from(out_str.clone());
        band_pass_filter(path.clone(), center_hz, out_g.clone());
        godot_print!(
            "test_melody_extraction: wrote isolated stem to '{}'",
            out_str
        );
        let onsets = extract_onset_times(out_g.clone());
        godot_print!("test_melody_extraction: detected {} onsets", onsets.len());
        onsets
    }

    #[func]
    pub fn get_midi_note_on_off_event_buffer_ticks(&self) -> Dictionary {
        const MIDI_FILE_PATH: &str = "/Users/ann/Documents/misc_game/2am.mid";
        make_note_on_off_event_dict(
            MIDI_FILE_PATH,
            parse_midi_events_into_note_on_off_event_buffer_ticks,
            |x| x as f32, // u64 → f32
        )
    }

    #[func]
    pub fn get_midi_note_on_off_event_buffer_seconds(&self) -> Dictionary {
        const MIDI_FILE_PATH: &str = "/Users/ann/Documents/misc_game/4.mid";
        // const MIDI_FILE_PATH: &str = "/Users/ann/Documents/misc_game/2am.mid";
        make_note_on_off_event_dict(
            MIDI_FILE_PATH,
            parse_midi_events_into_note_on_off_event_buffer_seconds,
            |x| x as f32, // f64 → f32
        )
    }

    #[func]
    pub fn render_midi_to_wav_bytes_constant_time(&self, sample_rate: i32) -> PackedByteArray {
        //const MIDI_FILE_PATH: &str = "/Users/ann/Documents/misc_game/2am.mid";
        const MIDI_FILE_PATH: &str = "/Users/ann/Documents/misc_game/4.mid";
        const SF2_PATH: &str = "/Users/ann/Documents/misc_game/Animal_Crossing_Wild_World.sf2";
        const PRESET_NAME: &str = "Accordion";
        const TARGET_CHANNEL: u8 = 0;
        const PROGRAM: u8 = 0;
        let mut reader = BufReader::new(File::open(SF2_PATH).unwrap());
        let soundfont = Arc::new(SoundFont::new(&mut reader).unwrap());
        let mut synth =
            Synthesizer::new(&soundfont, &SynthesizerSettings::new(sample_rate)).unwrap();
        let data = fs::read(MIDI_FILE_PATH).unwrap();
        let smf = Smf::parse(&data).unwrap();
        //TODO: make this more about the acccordian, "program" is such a shitty name for an instrument in midi
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
                        }
                        MidiMessage::NoteOff { key, .. } => {
                            let note = key.as_int() as i32;
                            synth.note_off(channel as i32, note);
                            active_notes.remove(&(channel, note));
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
        });
        while !active_notes.is_empty() {
            audio.push(render_sample_frame(&mut synth));
            time_cursor += step_secs;
        }
        write_samples_to_wav(sample_rate, audio)
    }
}
