use hound::SampleFormat::Int;
use hound::{WavSpec, WavWriter};
use midly::{MetaMessage, MidiMessage, Smf, Timing, TrackEventKind};
use rustysynth::{SoundFont, Synthesizer, SynthesizerSettings};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::io::Cursor;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MidiNote {
    pub midi_note: u8,
    pub instrument_id: u8,
}

pub fn render_midi_to_wav_bytes(
    sample_rate: i32,
    midi_bytes: &[u8],
    sf2_bytes: &[u8],
    target_channel: u8,
    program: u8,
) -> Result<Vec<u8>, Box<dyn Error>> {
    //TODO: the Box dyn is to allow for whatever hound Error or soundfont error or synth error i think
    let mut sf2_cursor = Cursor::new(sf2_bytes.to_vec());
    let sf = SoundFont::new(&mut sf2_cursor)?;
    let soundfont = Arc::new(sf);
    let mut synth = Synthesizer::new(&soundfont, &SynthesizerSettings::new(sample_rate))?;
    let smf = Smf::parse(midi_bytes)?;
    let mut events = prepare_events(&smf);
    events = inject_program_change(events, target_channel, program);
    let mut samples = Vec::new();
    let mut active_notes = HashSet::new();
    let mut time_cursor = 0_f32;
    let step_secs = 1_f32 / (sample_rate as f32);

    process_midi_events_with_timing(events, &smf, |event_time, event, ch| {
        while time_cursor < event_time {
            samples.push(render_one_frame(&mut synth));
            time_cursor += step_secs;
        }
        if let Some(channel) = ch {
            if let TrackEventKind::Midi { message, .. } = event {
                match message {
                    MidiMessage::NoteOn { key, vel } => {
                        let note = key.as_int();
                        let velocity = vel.as_int();
                        if velocity > 0 {
                            synth.note_on(channel as i32, note as i32, velocity as i32);
                            active_notes.insert((channel, note));
                        } else {
                            synth.note_off(channel as i32, note as i32);
                            active_notes.remove(&(channel, note));
                        }
                    },
                    MidiMessage::NoteOff { key, .. } => {
                        let note = key.as_int();
                        synth.note_off(channel as i32, note as i32);
                        active_notes.remove(&(channel, note));
                    },
                    _ => {},
                }
            }
        }
    });
    while !active_notes.is_empty() {
        samples.push(render_one_frame(&mut synth));
        time_cursor += step_secs;
    }
    Ok(write_samples_to_wav_bytes(sample_rate, &samples)?)
}

pub fn prepare_events(smf: &Smf) -> Vec<(u32, TrackEventKind<'static>)> {
    let mut events = Vec::new();
    for track in &smf.tracks {
        let mut abs_tick = 0_u32;
        for e in track {
            abs_tick += e.delta.as_int();
            events.push((abs_tick, e.kind.clone().to_static()));
        }
    }
    events.sort_by_key(|(t, _)| *t);
    events
}

pub fn inject_program_change(
    mut events: Vec<(u32, TrackEventKind<'static>)>,
    channel: u8,
    program: u8,
) -> Vec<(u32, TrackEventKind<'static>)> {
    let pc = TrackEventKind::Midi {
        channel: midly::num::u4::from(channel),
        message: MidiMessage::ProgramChange {
            program: midly::num::u7::from(program),
        },
    };
    events.insert(0, (0, pc));
    events
}

pub fn render_one_frame(synth: &mut Synthesizer) -> (i16, i16) {
    let mut left = [0_f32; 1];
    let mut right = [0_f32; 1];
    synth.render(&mut left, &mut right);
    let l_i = (left[0].clamp(-1_f32, 1_f32) * i16::MAX as f32) as i16;
    let r_i = (left[0].clamp(-1_f32, 1_f32) * i16::MAX as f32) as i16;
    (l_i, r_i)
}

pub fn write_samples_to_wav_bytes(sample_rate: i32, samples: &[(i16, i16)]) -> Result<Vec<u8>, hound::Error> {
    let spec = WavSpec {
        channels: 2_u16,
        sample_rate: sample_rate as u32, //TODO: i3 u32 choose whose in charge, rustysynth? SynthesizerSettings, or hound WavSpec?
        bits_per_sample: 16_u16,
        sample_format: Int,
    };
    let mut cursor = Cursor::new(Vec::new());
    let mut writer = WavWriter::new(&mut cursor, spec)?;
    for &(left, right) in samples {
        writer.write_sample(left)?;
        writer.write_sample(right)?;
    }
    writer.finalize()?;
    Ok(cursor.into_inner())
}

pub fn process_midi_events_with_timing(
    events: Vec<(u32, TrackEventKind<'static>)>,
    smf: &Smf,
    mut on_event: impl FnMut(f32, &TrackEventKind<'_>, Option<u8>),
) {
    let tpq = match smf.header.timing {
        Timing::Metrical(t) => t.as_int(),
        _ => panic!(),
    };
    let tpq_arithmetic = tpq as f32;
    let mut us_per_qn = 500_000_f32;
    let mut time_sec = 0_f32;
    let mut last_tick = 0_u32;
    for (tick, event) in events {
        let delta_ticks = tick - last_tick;
        let delta_ticks_arithmetic = delta_ticks as f32;
        let delta_secs = (delta_ticks_arithmetic / tpq_arithmetic) * (us_per_qn / 1_000_000_f32);
        time_sec += delta_secs;
        last_tick = tick;
        if let TrackEventKind::Meta(MetaMessage::Tempo(us)) = &event {
            us_per_qn = us.as_int() as f32; //TODO: idk what is best idiomatic to make these type casts clearer and intuitve
        }
        let channel = if let TrackEventKind::Midi { channel, .. } = event {
            Some(channel.as_int())
        } else {
            None
        };
        on_event(time_sec, &event, channel);
    }
}

fn inner_parse_note_on_off<T>(
    midi_bytes: &[u8],
    mut time_fn: impl FnMut(u32, &TrackEventKind<'_>) -> T,
    mut handle_note_fn: impl FnMut(u8, u8, u8, T, &[u8; 16]),
) {
    let smf = Smf::parse(midi_bytes).unwrap_or_else(|e| panic!("Failed to parse SMF from bytes: {}", e));
    let mut current_instrument_for_channel = [0u8; 16];
    let mut events: Vec<(u32, TrackEventKind<'static>)> = Vec::new();
    for track in &smf.tracks {
        let mut abs_tick = 0_u32;
        for e in track {
            abs_tick += e.delta.as_int();
            events.push((abs_tick, e.kind.clone().to_static()));
        }
    }
    events.sort_unstable_by_key(|(tick, _)| *tick);
    for (tick, kind) in events {
        let time_value = time_fn(tick, &kind);
        if let TrackEventKind::Midi { channel, message } = kind {
            let ch = channel.as_int();
            match message {
                MidiMessage::ProgramChange { program } => {
                    current_instrument_for_channel[ch as usize] = program.as_int();
                },
                MidiMessage::NoteOn { key, vel } => {
                    handle_note_fn(
                        ch,
                        key.as_int(),
                        vel.as_int(),
                        time_value,
                        &current_instrument_for_channel,
                    );
                },
                MidiMessage::NoteOff { key, .. } => {
                    handle_note_fn(ch, key.as_int(), 0, time_value, &current_instrument_for_channel);
                },
                _ => {},
            }
        }
    }
}

pub fn parse_midi_events_into_note_on_off_event_buffer_ticks_from_bytes(
    midi_bytes: &[u8],
) -> HashMap<MidiNote, Vec<(u32, u32)>> {
    let mut active_note_on: HashMap<(u8, u8), u32> = HashMap::new();
    let mut final_buffer: HashMap<MidiNote, Vec<(u32, u32)>> = HashMap::new();
    let smf = Smf::parse(midi_bytes).unwrap();
    let ticks_per_quarter = match smf.header.timing {
        Timing::Metrical(tpq) => tpq.as_int(),
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
) -> HashMap<MidiNote, Vec<(f32, f32)>> {
    let mut active_note_on: HashMap<(u8, u8), f32> = HashMap::new();
    let mut final_buffer: HashMap<MidiNote, Vec<(f32, f32)>> = HashMap::new();
    let smf = Smf::parse(midi_bytes).unwrap();
    let tpq = match smf.header.timing {
        Timing::Metrical(tpq) => tpq.as_int() as f32,
        _ => panic!("Unsupported MIDI timing format"),
    };
    inner_parse_note_on_off(
        midi_bytes,
        {
            let mut current_us_per_qn = 500_000_f32; // initial default microseconds per quarter note
            let mut last_tick = 0_u32;
            let mut elapsed_secs = 0_f32;
            move |tick, kind| {
                let delta_ticks = (tick - last_tick) as f32;
                let delta_secs = (delta_ticks / tpq) * (current_us_per_qn / 1_000_000_f32);
                elapsed_secs += delta_secs;
                last_tick = tick;
                if let TrackEventKind::Meta(MetaMessage::Tempo(us)) = kind {
                    current_us_per_qn = us.as_int() as f32;
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

pub fn debug_midi_note_onset_buffer(buffer: &HashMap<MidiNote, Vec<(u32, u32)>>, ticks_per_quarter: u16) {
    if buffer.is_empty() {
        println!("-- no note events to display --");
        return;
    }
    let bars_to_display = 8;
    let ticks_per_bar = ticks_per_quarter as usize * 4;
    let max_tick_to_display = (ticks_per_bar * bars_to_display) as u32;
    let chart_width = 128;
    let scale = max_tick_to_display as f32 / chart_width as f32;
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
                let start = (onset as f32 / scale).floor() as usize;
                let end = ((release.min(max_tick_to_display)) as f32 / scale).ceil() as usize;
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
                    let start = (onset as f32 / scale).floor() as usize;
                    let end = ((release.min(max_tick_to_display) as f32) / scale).ceil() as usize;
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
