use hound::{SampleFormat, WavSpec, WavWriter};
use midly::{MetaMessage, MidiMessage, Smf, Timing, TrackEventKind};
use rustysynth::Synthesizer;
use std::collections::HashMap;
use std::io::Cursor;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MidiNote {
    pub midi_note: u8,
    pub instrument_id: u8,
}

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
        channel: midly::num::u4::from(channel),
        message: MidiMessage::ProgramChange {
            program: midly::num::u7::from(program),
        },
    };
    events.insert(0, (0, pc));
    events
}

pub fn render_sample_frame(synth: &mut Synthesizer) -> (i16, i16) {
    let mut left = [0.0];
    let mut right = [0.0];
    synth.render(&mut left, &mut right);
    let l = (left[0].clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
    let r = (right[0].clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
    (l, r)
}

pub fn write_samples_to_wav_bytes(sample_rate: i32, samples: Vec<(i16, i16)>) -> Vec<u8> {
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
    buffer.into_inner()
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
        let channel = if let TrackEventKind::Midi {
            channel, ..
        } = event
        {
            Some(channel.as_int())
        } else {
            None
        };
        on_event(time_sec, &event, channel);
    }
}

fn inner_parse_note_on_off<T>(
    midi_bytes: &[u8],
    mut time_fn: impl FnMut(u64, &TrackEventKind<'_>) -> T,
    mut handle_note_fn: impl FnMut(u8, u8, u8, T, &[u8; 16]),
) {
    let smf = Smf::parse(midi_bytes).unwrap_or_else(|e| panic!("Failed to parse SMF from bytes: {}", e));

    let mut current_instrument_for_channel = [0u8; 16];
    let mut events: Vec<(u64, TrackEventKind<'static>)> = Vec::new();
    for track in &smf.tracks {
        let mut abs_tick = 0u64;
        for e in track {
            abs_tick += e.delta.as_int() as u64;
            events.push((abs_tick, e.kind.clone().to_static()));
        }
    }
    events.sort_unstable_by_key(|(tick, _)| *tick);
    for (tick, kind) in events {
        let time_value = time_fn(tick, &kind);
        if let TrackEventKind::Midi {
            channel,
            message,
        } = kind
        {
            let ch = channel.as_int();
            match message {
                MidiMessage::ProgramChange {
                    program,
                } => {
                    current_instrument_for_channel[ch as usize] = program.as_int();
                },
                MidiMessage::NoteOn {
                    key,
                    vel,
                } => {
                    handle_note_fn(
                        ch,
                        key.as_int(),
                        vel.as_int(),
                        time_value,
                        &current_instrument_for_channel,
                    );
                },
                MidiMessage::NoteOff {
                    key, ..
                } => {
                    handle_note_fn(ch, key.as_int(), 0, time_value, &current_instrument_for_channel);
                },
                _ => {},
            }
        }
    }
}

pub fn parse_midi_events_into_note_on_off_event_buffer_ticks_from_bytes(
    midi_bytes: &[u8],
) -> HashMap<MidiNote, Vec<(u64, u64)>> {
    let mut active_note_on: HashMap<(u8, u8), u64> = HashMap::new();
    let mut final_buffer: HashMap<MidiNote, Vec<(u64, u64)>> = HashMap::new();
    let smf = Smf::parse(midi_bytes).unwrap();
    let ticks_per_quarter: u64 = match smf.header.timing {
        Timing::Metrical(tpq) => tpq.as_int() as u64,
        _ => panic!("Unsupported MIDI timing format"),
    };
    inner_parse_note_on_off(
        midi_bytes,
        |tick, _kind| tick,
        |ch, note, vel, tick_value, current_instr_table| {
            let key = (ch, note);
            if vel > 0 {
                active_note_on.insert(key, tick_value);
            } else if let Some(onset_tick) = active_note_on.remove(&key) {
                let instrument_id = current_instr_table[ch as usize];
                let midi_note = MidiNote {
                    midi_note: note,
                    instrument_id,
                };
                final_buffer
                    .entry(midi_note)
                    .or_default()
                    .push((onset_tick, tick_value));
            }
        },
    );
    debug_midi_note_onset_buffer(&final_buffer, ticks_per_quarter);
    final_buffer
}

pub fn parse_midi_events_into_note_on_off_event_buffer_seconds_from_bytes(
    midi_bytes: &[u8],
) -> HashMap<MidiNote, Vec<(f64, f64)>> {
    let mut active_note_on: HashMap<(u8, u8), f64> = HashMap::new();
    let mut final_buffer: HashMap<MidiNote, Vec<(f64, f64)>> = HashMap::new();
    let smf = Smf::parse(midi_bytes).unwrap();
    let tpq: u64 = match smf.header.timing {
        Timing::Metrical(tpq) => tpq.as_int() as u64,
        _ => panic!("Unsupported MIDI timing format"),
    };
    inner_parse_note_on_off(
        midi_bytes,
        {
            let mut current_us_per_qn = 500_000u64; // initial default microseconds per quarter note
            let mut last_tick = 0u64;
            let mut elapsed_secs = 0f64;
            move |tick, kind| {
                let delta_ticks = tick - last_tick;
                let delta_secs = (delta_ticks as f64 / tpq as f64) * (current_us_per_qn as f64 / 1_000_000.0);
                elapsed_secs += delta_secs;
                last_tick = tick;
                if let TrackEventKind::Meta(MetaMessage::Tempo(us)) = kind {
                    current_us_per_qn = us.as_int() as u64;
                }
                elapsed_secs
            }
        },
        |ch, note, vel, time_value, current_instr_table| {
            let key = (ch, note);
            if vel > 0 {
                active_note_on.insert(key, time_value);
            } else if let Some(onset_sec) = active_note_on.remove(&key) {
                let instrument_id = current_instr_table[ch as usize];
                let midi_note = MidiNote {
                    midi_note: note,
                    instrument_id,
                };
                final_buffer.entry(midi_note).or_default().push((onset_sec, time_value));
            }
        },
    );

    final_buffer
}

pub fn debug_midi_note_onset_buffer(buffer: &HashMap<MidiNote, Vec<(u64, u64)>>, ticks_per_quarter: u64) {
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
        let names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
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
            format!("{}/{}", note_name(top.midi_note), note_name(bottom.midi_note))
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
