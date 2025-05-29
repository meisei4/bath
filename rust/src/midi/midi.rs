#![allow(dead_code)]
//TODO: this is very hard to strucutre becuase it needs to be shared with my main.rs testing and the lib.rs
// but there isa ton of unused code between both of them so you get compiler warnings all over the place
use godot::builtin::{Dictionary, PackedVector2Array, Vector2, Vector2i};
use godot::classes::file_access::ModeFlags;
use godot::classes::FileAccess;
use godot::prelude::PackedByteArray;
use hound::{SampleFormat, WavSpec, WavWriter};
use midly::num::{u4, u7};
use midly::{MetaMessage, MidiMessage, Smf, Timing, TrackEventKind};
use rustysynth::{SoundFont, Synthesizer};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Cursor};

pub fn prepare_events(smf: &Smf) -> Vec<(u64, TrackEventKind<'static>)> {
    let mut events = Vec::new();
    for track in &smf.tracks {
        let mut abs_tick = 0u64;
        for e in track {
            abs_tick += e.delta.as_int() as u64;
            events.push((abs_tick, e.kind.clone().to_static()));
        }
    }
    events.sort_by_key(|(t, _)| *t);
    events
}

pub fn inject_program_change(
    mut events: Vec<(u64, TrackEventKind<'static>)>,
    channel: u8,
    program: u8,
) -> Vec<(u64, TrackEventKind<'static>)> {
    let pc = TrackEventKind::Midi {
        channel: u4::from(channel),
        message: MidiMessage::ProgramChange {
            program: u7::from(program),
        },
    };
    events.insert(0, (0, pc));
    events
}

//TODO: the above function is actually how you set the soundfont instrumetn
pub fn assign_midi_instrument_from_soundfont(
    channel: u8,
    preset_name: &str,
    soundfont_path: &str,
    mut send: impl FnMut(&[u8]),
) {
    let file = File::open(soundfont_path).unwrap();
    let mut reader = BufReader::new(file);
    let soundfont = SoundFont::new(&mut reader).unwrap();

    let preset = soundfont
        .get_presets()
        .iter()
        .find(|p| p.get_name() == preset_name)
        .unwrap();

    let full_bank = preset.get_bank_number();
    let bank_msb = (full_bank >> 7) as u8;
    let bank_lsb = (full_bank & 0x7F) as u8;
    let program = preset.get_patch_number() as u8;

    send(&[0xB0 | channel, 0x00, bank_msb]);
    send(&[0xB0 | channel, 0x20, bank_lsb]);
    send(&[0xC0 | channel, program]);
}

pub fn render_sample_frame(synth: &mut Synthesizer) -> (i16, i16) {
    let mut left = [0.0];
    let mut right = [0.0];
    synth.render(&mut left, &mut right);

    let l = (left[0].clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
    let r = (right[0].clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
    (l, r)
}

pub fn write_samples_to_wav(sample_rate: i32, samples: Vec<(i16, i16)>) -> PackedByteArray {
    let mut buffer = Cursor::new(Vec::new());
    let spec = WavSpec {
        channels: 2,
        sample_rate: sample_rate as u32,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };
    let mut writer = WavWriter::new(&mut buffer, spec).unwrap();
    for (l, r) in samples {
        writer.write_sample(l).unwrap();
        writer.write_sample(r).unwrap();
    }
    writer.finalize().unwrap();
    PackedByteArray::from(buffer.into_inner())
}

pub fn process_midi_events_with_timing(
    events: Vec<(u64, TrackEventKind<'static>)>,
    smf: &Smf,
    mut on_event: impl FnMut(f64, &TrackEventKind<'_>, Option<u8>),
) {
    let tpq = match smf.header.timing {
        Timing::Metrical(t) => t.as_int() as f64,
        _ => panic!(),
    };

    let mut us_per_qn = 500_000u64;
    let mut time_sec = 0.0;
    let mut last_tick = 0u64;

    for (tick, event) in events {
        let delta_ticks = tick - last_tick;
        let delta_secs = (delta_ticks as f64 / tpq) * (us_per_qn as f64 / 1_000_000.0);
        time_sec += delta_secs;
        last_tick = tick;

        if let TrackEventKind::Meta(MetaMessage::Tempo(us)) = &event {
            us_per_qn = us.as_int() as u64;
        }
        let channel = match &event {
            TrackEventKind::Midi { channel, .. } => Some(channel.as_int()),
            _ => None,
        };
        on_event(time_sec, &event, channel);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MidiNote {
    pub midi_note: u8,
    pub instrument_id: u8,
}

pub fn parse_midi_events_into_note_on_off_event_buffer_ticks(
    midi_path: &str,
) -> HashMap<MidiNote, Vec<(u64, u64)>> {
    let mut active_note_on_events = HashMap::new();
    let mut final_note_on_off_event_buffer = HashMap::new();
    parse_midi_events_into_note_on_off(
        midi_path,
        |tick, _| tick,
        |channel_index, key, vel, event_tick_position, current_instrument_for_channel| {
            handle_note_message(
                channel_index,
                key,
                vel,
                event_tick_position,
                current_instrument_for_channel,
                &mut active_note_on_events,
                &mut final_note_on_off_event_buffer,
            );
        },
    );
    let file = FileAccess::open(midi_path, ModeFlags::READ).unwrap();
    let midi_file_bytes = file.get_buffer(file.get_length() as i64).to_vec();
    //let midi_file_bytes = fs::read(midi_path).unwrap();
    let smf = Smf::parse(&midi_file_bytes).unwrap();

    let ticks_per_quarter: u64 = match smf.header.timing {
        Timing::Metrical(tpq) => tpq.as_int() as u64,
        _ => panic!("Unsupported MIDI timing format"),
    };
    debug_midi_note_onset_buffer(&final_note_on_off_event_buffer, ticks_per_quarter);
    final_note_on_off_event_buffer
}

pub fn parse_midi_events_into_note_on_off_event_buffer_seconds(
    midi_path: &str,
) -> HashMap<MidiNote, Vec<(f64, f64)>> {
    let file = FileAccess::open(midi_path, ModeFlags::READ).unwrap();
    let midi_file_bytes = file.get_buffer(file.get_length() as i64).to_vec();
    let smf = Smf::parse(&midi_file_bytes).unwrap();
    let tpq = match smf.header.timing {
        Timing::Metrical(tpq) => tpq.as_int() as u64,
        _ => panic!("Unsupported MIDI timing format"),
    };
    let mut current_us_per_qn = 500_000u64;
    let mut last_tick = 0u64;
    let mut elapsed = 0f64;
    let mut active_note_on_events = HashMap::new();
    let mut final_note_on_off_event_buffer = HashMap::new();
    parse_midi_events_into_note_on_off(
        midi_path,
        |tick, kind| {
            let delta_ticks = tick - last_tick;
            let delta_secs =
                (delta_ticks as f64 / tpq as f64) * (current_us_per_qn as f64 / 1_000_000.0);
            elapsed += delta_secs;
            last_tick = tick;

            if let TrackEventKind::Meta(MetaMessage::Tempo(us)) = kind {
                current_us_per_qn = us.as_int() as u64;
            }

            elapsed
        },
        |channel_index, note, vel, elapsed_seconds, current_instrument_for_channel| {
            handle_note_message(
                channel_index,
                note,
                vel,
                elapsed_seconds,
                current_instrument_for_channel,
                &mut active_note_on_events,
                &mut final_note_on_off_event_buffer,
            );
        },
    );
    final_note_on_off_event_buffer
}

fn parse_midi_events_into_note_on_off<T>(
    midi_path: &str,
    mut time_fn: impl FnMut(u64, &TrackEventKind<'_>) -> T,
    mut handle_note_fn: impl FnMut(u8, u8, u8, T, &[u8; 16]),
) {
    let file = FileAccess::open(midi_path, ModeFlags::READ).unwrap();
    let midi_file_bytes = file.get_buffer(file.get_length() as i64).to_vec();
    let smf = Smf::parse(&midi_file_bytes).unwrap();
    let mut current_instrument_for_channel = [0u8; 16];
    let mut events = Vec::new();
    for track in &smf.tracks {
        let mut tick = 0u64;
        for event in track {
            tick += event.delta.as_int() as u64;
            events.push((tick, event.kind.clone()));
        }
    }
    events.sort_unstable_by_key(|(tick, _)| *tick);

    for (tick, kind) in events {
        let time = time_fn(tick, &kind);
        match kind {
            TrackEventKind::Midi { channel, message } => {
                let ch = channel.as_int();
                match message {
                    MidiMessage::ProgramChange { program } => {
                        current_instrument_for_channel[ch as usize] = program.as_int();
                    }
                    MidiMessage::NoteOn { key, vel } => {
                        handle_note_fn(
                            ch,
                            key.as_int(),
                            vel.as_int(),
                            time,
                            &current_instrument_for_channel,
                        );
                    }
                    MidiMessage::NoteOff { key, .. } => {
                        handle_note_fn(ch, key.as_int(), 0, time, &current_instrument_for_channel);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

fn handle_note_message<T: Copy>(
    midi_note_channel: u8,
    midi_note_number: u8,
    event_velocity: u8,
    event_time: T,
    current_instrument_for_channel: &[u8; 16],
    active_note_on_events: &mut HashMap<(u8, u8), T>,
    midi_note_onset_buffer: &mut HashMap<MidiNote, Vec<(T, T)>>,
) {
    if event_velocity > 0 {
        active_note_on_events.insert((midi_note_channel, midi_note_number), event_time);
    } else if let Some(onset_time) =
        active_note_on_events.remove(&(midi_note_channel, midi_note_number))
    {
        let instrument_identifier = current_instrument_for_channel[midi_note_channel as usize];
        let key = MidiNote {
            midi_note: midi_note_number,
            instrument_id: instrument_identifier,
        };
        midi_note_onset_buffer
            .entry(key)
            .or_default()
            .push((onset_time, event_time));
    }
}

pub fn debug_midi_note_onset_buffer(
    buffer: &HashMap<MidiNote, Vec<(u64, u64)>>,
    ticks_per_quarter: u64,
) {
    if buffer.is_empty() {
        println!("-- no note events to display --");
        return;
    }
    let bars_to_display = 8;
    let ticks_per_bar = ticks_per_quarter * 4;
    let max_tick_to_display = ticks_per_bar * bars_to_display as u64;
    let chart_width: usize = 128;
    let scale = max_tick_to_display as f64 / chart_width as f64;
    let label_width = 7;
    let segment = chart_width / bars_to_display;
    print!("{:label_width$}", "");
    for bar in 1..=bars_to_display {
        let bar_str = bar.to_string();
        print!("│{}", bar_str);
        for _ in 0..segment - 1 - bar_str.len() {
            print!("─");
        }
    }
    println!();
    fn note_name(n: u8) -> String {
        let names = [
            "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
        ];
        let octave = n / 12;
        format!("{}{}", names[(n % 12) as usize], octave)
    }
    let mut all_notes: Vec<MidiNote> = buffer.keys().cloned().collect();
    all_notes.sort_by_key(|n| n.midi_note);
    all_notes.reverse();
    let mut pairs = Vec::new();
    let mut i = 0;
    while i < all_notes.len() {
        let top = all_notes[i].clone();
        let bottom = all_notes.get(i + 1).cloned();
        pairs.push((top, bottom));
        i += 2;
    }
    for (top, bottom_opt) in pairs {
        let label = if let Some(bottom) = &bottom_opt {
            format!(
                "{}/{}",
                note_name(top.midi_note),
                note_name(bottom.midi_note)
            )
        } else {
            note_name(top.midi_note)
        };
        print!("{:<label_width$}", label);
        let mut row = vec![' '; chart_width];
        if let Some(segments) = buffer.get(&top) {
            for &(onset, release) in segments {
                if onset >= max_tick_to_display {
                    continue;
                }
                let start = (onset as f64 / scale).floor() as usize;
                let end = ((release.min(max_tick_to_display) as f64) / scale).ceil() as usize;
                for x in start.min(chart_width - 1)..end.min(chart_width) {
                    row[x] = '▀';
                }
            }
        }
        if let Some(bottom) = bottom_opt {
            if let Some(segments) = buffer.get(&bottom) {
                for &(onset, release) in segments {
                    if onset >= max_tick_to_display {
                        continue;
                    }
                    let start = (onset as f64 / scale).floor() as usize;
                    let end = ((release.min(max_tick_to_display) as f64) / scale).ceil() as usize;
                    for x in start.min(chart_width - 1)..end.min(chart_width) {
                        row[x] = match row[x] {
                            '▀' | '█' => '█',
                            _ => '▄',
                        };
                    }
                }
            }
        }
        let line: String = row.into_iter().collect();
        println!("{}", line);
    }
}

pub fn make_note_on_off_event_dict<T>(
    midi_path: &str,
    parser_fn: impl Fn(&str) -> HashMap<MidiNote, Vec<(T, T)>>,
    to_f32: impl Fn(T) -> f32,
) -> Dictionary
where
    T: Copy,
{
    let event_buffer = parser_fn(midi_path);
    let mut dict = Dictionary::new();
    for (key, segments) in event_buffer {
        let dict_key = Vector2i::new(key.midi_note as i32, key.instrument_id as i32);
        let mut arr = PackedVector2Array::new();
        for (onset, release) in segments {
            arr.push(Vector2::new(to_f32(onset), to_f32(release)));
        }
        let _ = dict.insert(dict_key, arr);
    }
    dict
}
