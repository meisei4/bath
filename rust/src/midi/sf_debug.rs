#![allow(dead_code)]
//TODO: this is very hard to strucutre becuase it needs to be shared with my main.rs testing and the lib.rs
// but there isa ton of unused code between both of them so you get compiler warnings all over the place

use crate::midi::keys::{key_bindings, note_to_name};

use rustysynth::{Instrument, InstrumentRegion, Preset, SoundFont};
use std::{error::Error, fs::File, io::BufReader};
use terminal_size::{terminal_size, Width};

const L0: &str = "";
const L1: &str = "  ├── ";
const L1_LAST: &str = "  └── ";
const L2: &str = "        ├── ";
const L2_LAST: &str = "        └── ";
const L3: &str = "        │   ├── ";
const L3_LAST: &str = "        │   └── ";
const L4: &str = "        │   │   ├── ";
const L4_LAST: &str = "        │   │   └── ";

pub fn print_full_structure(
    soundfont_file_path: &str,
    bank: i32,
    patch: i32,
) -> Result<(), Box<dyn Error>> {
    let file = File::open(soundfont_file_path)?;
    let mut reader = BufReader::new(file);
    let soundfont = SoundFont::new(&mut reader)?;
    let preset = soundfont
        .get_presets()
        .iter()
        .find(|p| p.get_bank_number() == bank && p.get_patch_number() == patch)
        .ok_or("No matching preset found in SoundFont.")?;
    let mut lines = Vec::new();
    print_preset_info(preset, &mut lines);
    let preset_regions = preset.get_regions();
    lines.push(format!("{L1}Preset Regions: {} bags", preset_regions.len()));
    for (i, preset_region) in preset_regions.iter().enumerate() {
        let is_last = i == preset_regions.len() - 1;
        let region_label = if is_last {
            format!("{L1_LAST}Preset Region index: {}", i)
        } else {
            format!("{L1}Preset Region index: {}", i)
        };
        lines.push(region_label);
        let instrument_index = preset_region.get_instrument_id();
        if let Some(instrument) = soundfont.get_instruments().get(instrument_index) {
            print_instrument_info(instrument, &soundfont, &mut lines);
        } else {
            lines.push(format!(
                "{L2}(Missing instrument at index {})",
                instrument_index
            ));
        }
    }
    print_aligned_right(&lines);
    Ok(())
}

fn print_preset_info(preset: &Preset, lines: &mut Vec<String>) {
    lines.push(format!("{L0}Preset: \"{}\"", preset.get_name()));
    lines.push(format!("{L1}Bank Number: {}", preset.get_bank_number()));
    lines.push(format!(
        "{L1_LAST}Patch Number: {}",
        preset.get_patch_number()
    ));
}

fn print_instrument_info(instrument: &Instrument, soundfont: &SoundFont, lines: &mut Vec<String>) {
    lines.push(format!("{L2}Instrument: \"{}\"", instrument.get_name()));
    let instrument_regions = instrument.get_regions();
    lines.push(format!(
        "{L2}Instrument Regions: {} bags",
        instrument_regions.len()
    ));
    for (i, region) in instrument_regions.iter().enumerate() {
        let is_last = i == instrument_regions.len() - 1;
        let label = if is_last {
            format!("{L2_LAST}Instrument Region index: {}", i)
        } else {
            format!("{L2}Instrument Region index: {}", i)
        };
        lines.push(label);
        print_instrument_region_key_velocity_info(region, lines);
        print_sample_info(region, soundfont, lines);
        print_region_pan_envelope(region, lines);
    }
}

fn print_instrument_region_key_velocity_info(region: &InstrumentRegion, lines: &mut Vec<String>) {
    let low = region.get_key_range_start() as u8;
    let high = region.get_key_range_end() as u8;
    let low_note = note_to_name(low);
    let high_note = note_to_name(high);
    lines.push(format!(
        "{L3}Key Range: {}–{} ({}–{})",
        low, high, low_note, high_note
    ));
    let vel_low = region.get_velocity_range_start();
    let vel_high = region.get_velocity_range_end();
    lines.push(format!(
        "{L3}Velocity Range: {}–{} (0=soft, 127=hard)",
        vel_low, vel_high
    ));
}

fn print_sample_info(region: &InstrumentRegion, soundfont: &SoundFont, lines: &mut Vec<String>) {
    let sample_id = region.get_sample_id() as usize;
    if let Some(sample) = soundfont.get_sample_headers().get(sample_id) {
        lines.push(format!("{L3}Sample: \"{}\"", sample.get_name()));
        lines.push(format!("{L4}Sample Rate: {} Hz", sample.get_sample_rate()));
        let loop_start = sample.get_start_loop();
        let loop_end = sample.get_end_loop();
        lines.push(format!("{L4}Loop: {} → {} ", loop_start, loop_end));
        let pitch = sample.get_original_pitch();
        let note = note_to_name(sample.get_original_pitch() as u8);
        lines.push(format!("{L4}Original Pitch: {} ({})", pitch, note));
        let pitch_correct = sample.get_pitch_correction();
        lines.push(format!("{L4}Pitch Correction: {} cents", pitch_correct));
        lines.push(format!(
            "{L4_LAST}Sample Type: {} (1 = mono)",
            sample.get_sample_type()
        ));
    }
}

fn print_region_pan_envelope(region: &InstrumentRegion, lines: &mut Vec<String>) {
    let pan = region.get_pan();
    let pan_desc = if pan == 0.0 {
        "Center".into()
    } else if pan < 0.0 {
        format!("Left {}% ({} pan)", (pan.abs() * 100.0) as i32, pan)
    } else {
        format!("Right {}% ({} pan)", (pan * 100.0) as i32, pan)
    };
    lines.push(format!("{L3}Pan Position: {} (Stereo balance)", pan_desc));
    let attack = region.get_attack_volume_envelope();
    lines.push(format!(
        "{L3}Volume Envelope - Attack Time: {:.3} sec",
        attack
    ));
    let decay = region.get_decay_volume_envelope();
    lines.push(format!(
        "{L3}Volume Envelope - Decay Time: {:.3} sec",
        decay
    ));
    let sustain = region.get_sustain_volume_envelope();
    lines.push(format!(
        "{L3}Volume Envelope - Sustain Level: {:.1} dB",
        sustain
    ));
    let envelope_vol = region.get_release_volume_envelope();
    lines.push(format!(
        "{L3_LAST}Volume Envelope - Release Time: {:.3} sec)",
        envelope_vol
    ));
}

fn print_aligned_right(lines: &[String]) {
    let max_width = lines.iter().map(|l| l.len()).max().unwrap_or(0);
    let keyboard_width = key_bindings().len() * 6;
    let gap = 1;
    let term_width = terminal_size()
        .map(|(Width(w), _)| w as usize)
        .unwrap_or(80);
    let required = keyboard_width + gap + max_width;
    let pad = if required <= term_width {
        keyboard_width + gap
    } else {
        term_width.saturating_sub(max_width)
    };
    for line in lines {
        println!("{}{}", " ".repeat(pad), line);
    }
}
