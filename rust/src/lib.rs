mod audio_analysis;
mod collision_mask;
mod midi;

use crate::midi::midi::{
    parse_midi_events_into_note_on_off_event_buffer_SECONDS,
    parse_midi_events_into_note_on_off_event_buffer_TICKS,
};
use audio_analysis::util::{band_pass_filter, detect_bpm, extract_onset_times};
use collision_mask::util::{
    generate_concave_collision_polygons_pixel_perfect,
    generate_convex_collision_polygons_pixel_perfect,
};
use godot::builtin::{PackedByteArray, PackedVector2Array, Vector2};
use godot::classes::{AudioStreamGenerator, Node2D};
use godot::obj::{Gd, NewGd};
use godot::prelude::{
    gdextension, godot_api, godot_print, Array, Base, Dictionary, ExtensionLibrary, GString,
    GodotClass, PackedFloat32Array, Vector2i,
};
use midly::{MetaMessage, MidiMessage, Smf, Timing, TrackEventKind};
use rustysynth::{SoundFont, Synthesizer, SynthesizerSettings};
use std::fs::File;
use std::io::BufReader;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::thread;

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
    pub fn get_midi_note_on_off_event_buffer_TICKS(&self) -> Dictionary {
        const MIDI_FILE_PATH: &str = "/Users/ann/Documents/misc_game/2am.mid";
        let note_on_off_event_buffer =
            parse_midi_events_into_note_on_off_event_buffer_TICKS(MIDI_FILE_PATH);
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
    pub fn get_midi_note_on_off_event_buffer_SECONDS(&self) -> Dictionary {
        const MIDI_FILE_PATH: &str = "/Users/ann/Documents/misc_game/2am.mid";
        let note_on_off_event_buffer =
            parse_midi_events_into_note_on_off_event_buffer_SECONDS(MIDI_FILE_PATH);
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
    pub fn render_midi_to_pcm(&self, sample_rate: i32) -> PackedFloat32Array {
        const MIDI_FILE_PATH: &str = "/Users/ann/Documents/misc_game/2am.mid";
        let buffer = parse_midi_events_into_note_on_off_event_buffer_SECONDS(MIDI_FILE_PATH);
        let mut max_time = 0.0f64;
        for segments in buffer.values() {
            for &(_on, off) in segments {
                if off > max_time {
                    max_time = off;
                }
            }
        }
        let sr = sample_rate as usize;
        let total_samples = (max_time * sr as f64).ceil() as usize;
        let mut pcm = vec![0.0f32; total_samples];
        let freq = 440.0f32;
        let amp = 0.2f32;
        for segments in buffer.values() {
            for &(on_sec, off_sec) in segments {
                let start = (on_sec * sr as f64).floor() as usize;
                let end = (off_sec * sr as f64).ceil() as usize;
                for i in start..end.min(total_samples) {
                    let t = i as f32 / sr as f32;
                    pcm[i] += amp * (2.0 * std::f32::consts::PI * freq * t).sin();
                }
            }
        }
        let mut out = PackedFloat32Array::new();
        for &sample in pcm.iter() {
            out.push(sample);
        }
        out
    }

    #[func]
    pub fn create_midi_audio_generator(&self, sample_rate: i32) -> Gd<AudioStreamGenerator> {
        const MIDI_FILE_PATH: &str = "/Users/ann/Documents/misc_game/2am.mid";
        const SF2_PATH: &str = "/Users/ann/Documents/misc_game/Animal_Crossing_Wild_World.sf2";
        let mut generator = AudioStreamGenerator::new_gd();
        generator.set_mix_rate(sample_rate as f32);
        generator.set_buffer_length(0.1);
        let playback = generator.instantiate_playback();
        let playback = Arc::new(Mutex::new(playback));
        let pb_clone = Arc::clone(&playback);
        thread::spawn(move || {
            let file = File::open(SF2_PATH).expect("Failed to open SF2 file");
            let mut reader = BufReader::new(file);
            let sf = Arc::new(SoundFont::new(&mut reader).expect("Failed to parse SF2"));
            let settings = SynthesizerSettings::new(sample_rate);
            let mut synth = Synthesizer::new(&sf, &settings).expect("Failed to create Synthesizer");
            let data = std::fs::read(MIDI_FILE_PATH).expect("Failed to read MIDI file");
            let smf = Smf::parse(&data).expect("Failed to parse MIDI");
            let tpq = match smf.header.timing {
                Timing::Metrical(t) => t.as_int() as u64,
                _ => panic!("Unsupported timing format"),
            };
            let mut events = Vec::new();
            for track in &smf.tracks {
                let mut abs = 0u64;
                for ev in track {
                    abs += ev.delta.as_int() as u64;
                    events.push((abs, ev.kind.clone()));
                }
            }
            events.sort_by_key(|(t, _)| *t);
            let mut last_tick = 0u64;
            let mut us_per_qn = 500_000u64; // 120 BPM default
            for (tick, kind) in events {
                let delta_ticks = tick - last_tick;
                let delta_secs =
                    (delta_ticks as f64 / tpq as f64) * (us_per_qn as f64 / 1_000_000.0);
                if delta_secs > 0.0 {
                    thread::sleep(Duration::from_secs_f64(delta_secs));
                }
                last_tick = tick;
                match kind {
                    TrackEventKind::Meta(MetaMessage::Tempo(us)) => {
                        us_per_qn = us.as_int() as u64;
                    }
                    TrackEventKind::Midi { channel, message } => {
                        let ch = channel.as_int() as i32;
                        match message {
                            MidiMessage::NoteOn { key, vel } => {
                                let note = key.as_int() as i32;
                                let v = vel.as_int() as i32;
                                if v > 0 {
                                    synth.note_on(ch, note, v);
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
                let frames = (0.1 * sample_rate as f32) as usize;
                let mut left = vec![0.0f32; frames];
                let mut right = vec![0.0f32; frames];
                synth.render(&mut left[..], &mut right[..]);
                if let Ok(playback) = pb_clone.lock() {
                    for i in 0..frames {
                        let l = left[i];
                        let r = right[i];
                        //playback.push_frame(Vector2::new(l, r));
                    }
                }
            }
        });
        generator
    }
}
