mod audio_analysis;
mod collision_mask;
mod midi;

use crate::midi::midi::{
    parse_midi_events_into_note_on_off_event_buffer_seconds,
    parse_midi_events_into_note_on_off_event_buffer_ticks,
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
    GodotClass, PackedFloat32Array, Vector2i,
};
use hound::{SampleFormat, WavSpec, WavWriter};
use midly::{MetaMessage, MidiMessage, Smf, Timing, TrackEventKind};
use rustysynth::{SoundFont, Synthesizer, SynthesizerSettings};
use std::fs;
use std::fs::File;
use std::io::{BufReader, Cursor};
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
    // RETURN TYPE:  Dictionary[Vector2i, PackedVector2Array]
    pub fn get_midi_note_on_off_event_buffer_ticks(&self) -> Dictionary {
        const MIDI_FILE_PATH: &str = "/Users/ann/Documents/misc_game/2am.mid";
        let note_on_off_event_buffer =
            parse_midi_events_into_note_on_off_event_buffer_ticks(MIDI_FILE_PATH);
        // 2. Build a Godot Dictionary[Vector2i, PackedVector2Array]
        let mut godot_note_on_off_event_buffer = Dictionary::new();
        for (key, segments) in note_on_off_event_buffer {
            // use Vector2i(note, instrument) as the dictionary key
            let dict_key = Vector2i::new(key.midi_note as i32, key.instrument_id as i32);
            // pack each (onset, release) into a PackedVector2Array of Vector2(on_timestamp, off_timestamp)
            let mut arr = PackedVector2Array::new();
            for (onset, release) in segments {
                arr.push(Vector2::new(onset as f32, release as f32));
            }
            let _ = godot_note_on_off_event_buffer.insert(dict_key, arr);
        }
        godot_note_on_off_event_buffer
    }

    #[func]
    // RETURN TYPE:  Dictionary[Vector2i, PackedVector2Array]
    pub fn get_midi_note_on_off_event_buffer_seconds(&self) -> Dictionary {
        const MIDI_FILE_PATH: &str = "/Users/ann/Documents/misc_game/2am.mid";
        let note_on_off_event_buffer =
            parse_midi_events_into_note_on_off_event_buffer_seconds(MIDI_FILE_PATH);
        // 2. Build a Godot Dictionary[Vector2i, PackedVector2Array]
        let mut godot_note_on_off_event_buffer = Dictionary::new();
        for (key, segments) in note_on_off_event_buffer {
            // use Vector2i(note, instrument) as the dictionary key
            let dict_key = Vector2i::new(key.midi_note as i32, key.instrument_id as i32);
            // pack each (onset, release) into a PackedVector2Array of Vector2(on_timestamp, off_timestamp)
            let mut arr = PackedVector2Array::new();
            for (onset, release) in segments {
                arr.push(Vector2::new(onset as f32, release as f32));
            }
            let _ = godot_note_on_off_event_buffer.insert(dict_key, arr);
        }
        godot_note_on_off_event_buffer
    }

    #[func]
    pub fn render_midi_to_wav_bytes(&self, sample_rate: i32) -> PackedByteArray {
        const MIDI_PATH: &str = "/Users/ann/Documents/misc_game/2am.mid";
        const SF2_PATH: &str = "/Users/ann/Documents/misc_game/Animal_Crossing_Wild_World.sf2";
        let mut reader = BufReader::new(File::open(SF2_PATH).unwrap());
        let soundfont = Arc::new(SoundFont::new(&mut reader).expect("Failed to parse SF2"));
        let mut synth =
            Synthesizer::new(&soundfont, &SynthesizerSettings::new(sample_rate)).unwrap();
        let data = fs::read(MIDI_PATH).unwrap();
        let smf = Smf::parse(&data).unwrap();
        let tpq = match smf.header.timing {
            Timing::Metrical(t) => t.as_int() as f64,
            _ => panic!("Unsupported MIDI timing format"),
        };
        let mut events = Vec::new();
        for track in &smf.tracks {
            let mut abs_tick = 0u64;
            for e in track {
                abs_tick += e.delta.as_int() as u64;
                events.push((abs_tick, e.kind.clone()));
            }
        }
        events.sort_by_key(|(t, _)| *t);
        let buffer = Vec::new();
        let mut wav_buffer = Cursor::new(buffer);
        let spec = WavSpec {
            channels: 2,
            sample_rate: sample_rate as u32,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::new(&mut wav_buffer, spec).unwrap();
        let mut last_tick = 0u64;
        let mut us_per_qn = 500_000u64;
        for (tick, event) in events {
            let dt_ticks = tick - last_tick;
            let dt_secs = (dt_ticks as f64 / tpq) * (us_per_qn as f64 / 1_000_000.0);
            last_tick = tick;
            let frames = (dt_secs * sample_rate as f64).ceil() as usize;
            if frames > 0 {
                let mut left = vec![0.0f32; frames];
                let mut right = vec![0.0f32; frames];
                synth.render(&mut left, &mut right);
                for i in 0..frames {
                    let l = (left[i].clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
                    let r = (right[i].clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
                    writer.write_sample(l).unwrap();
                    writer.write_sample(r).unwrap();
                }
            }
            match event {
                TrackEventKind::Meta(MetaMessage::Tempo(us)) => {
                    us_per_qn = us.as_int() as u64;
                }
                TrackEventKind::Midi { channel, message } => {
                    let ch = channel.as_int() as i32;
                    match message {
                        MidiMessage::NoteOn { key, vel } => {
                            let note = key.as_int() as i32;
                            let velocity = vel.as_int() as i32;
                            if velocity > 0 {
                                synth.note_on(ch, note, velocity);
                            } else {
                                synth.note_off(ch, note);
                            }
                        }
                        MidiMessage::NoteOff { key, .. } => {
                            let note = key.as_int() as i32;
                            synth.note_off(ch, note);
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        writer.finalize().unwrap();
        let buf: Vec<u8> = wav_buffer.into_inner();
        let out = PackedByteArray::from(buf);
        out
    }

    #[func]
    pub fn render_midi_to_wav_bytes_constant_time(&self, sample_rate: i32) -> PackedByteArray {
        const MIDI_PATH: &str = "/Users/ann/Documents/misc_game/2am.mid";
        const SF2_PATH: &str = "/Users/ann/Documents/misc_game/Animal_Crossing_Wild_World.sf2";
        let mut reader = BufReader::new(File::open(SF2_PATH).unwrap());
        let soundfont = Arc::new(SoundFont::new(&mut reader).expect("Failed to parse SF2"));
        let mut synth =
            Synthesizer::new(&soundfont, &SynthesizerSettings::new(sample_rate)).unwrap();
        let data = fs::read(MIDI_PATH).unwrap();
        let smf = Smf::parse(&data).unwrap();
        let tpq = match smf.header.timing {
            Timing::Metrical(t) => t.as_int() as f64,
            _ => panic!("Unsupported MIDI timing format"),
        };
        let mut events = Vec::new();
        for track in &smf.tracks {
            let mut abs_tick = 0u64;
            for e in track {
                abs_tick += e.delta.as_int() as u64;
                events.push((abs_tick, e.kind.clone()));
            }
        }
        events.sort_by_key(|(t, _)| *t);
        let mut wav_buffer = Cursor::new(Vec::new());
        let spec = WavSpec {
            channels: 2,
            sample_rate: sample_rate as u32,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };
        let mut writer = WavWriter::new(&mut wav_buffer, spec).unwrap();
        let mut us_per_qn = 500_000u64;
        let mut ticks_per_second = (1_000_000.0 / us_per_qn as f64) * tpq; //TODO:this fixes the tick speed issue line??????
        let mut time_sec = 0.0;
        let step_secs = 1.0 / sample_rate as f64;
        let mut tick_position = 0u64;
        let mut event_index = 0;
        let mut active_notes = std::collections::HashSet::new();
        loop {
            while event_index < events.len() && events[event_index].0 <= tick_position {
                let (_, event) = &events[event_index];
                match event {
                    TrackEventKind::Meta(MetaMessage::Tempo(us)) => {
                        us_per_qn = us.as_int() as u64;
                        ticks_per_second = (1_000_000.0 / us_per_qn as f64) * tpq;
                        // <- recalc here
                    }
                    TrackEventKind::Midi { channel, message } => {
                        let ch = channel.as_int() as i32;
                        match message {
                            MidiMessage::NoteOn { key, vel } => {
                                let note = key.as_int() as i32;
                                let velocity = vel.as_int() as i32;
                                if velocity > 0 {
                                    synth.note_on(ch, note, velocity);
                                    active_notes.insert((ch, note));
                                } else {
                                    synth.note_off(ch, note);
                                    active_notes.remove(&(ch, note));
                                }
                            }
                            MidiMessage::NoteOff { key, .. } => {
                                let note = key.as_int() as i32;
                                synth.note_off(ch, note);
                                active_notes.remove(&(ch, note));
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
                event_index += 1;
            }

            let mut left = [0.0f32];
            let mut right = [0.0f32];
            synth.render(&mut left, &mut right);
            let l = (left[0].clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
            let r = (right[0].clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
            writer.write_sample(l).unwrap();
            writer.write_sample(r).unwrap();

            // time_sec += step_secs;
            // tick_position = (time_sec * tpq * 1_000_000.0 / us_per_qn as f64) as u64;
            //^wrong
            time_sec += step_secs;
            tick_position = (time_sec * ticks_per_second) as u64;

            if event_index >= events.len() && active_notes.is_empty() {
                break;
            }
        }

        writer.finalize().unwrap();
        let buf = wav_buffer.into_inner();
        PackedByteArray::from(buf)
    }
}
