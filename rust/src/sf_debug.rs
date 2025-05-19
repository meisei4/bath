use crate::keys::{key_bindings, note_to_name};
use crate::tsf_bindings::{tsf_close, tsf_get_presetcount, tsf_get_presetname, tsf_load_filename};
use rustysynth::SoundFont;
use std::ffi::{CStr, CString};
use std::{error::Error, fs::File, io::BufReader};
use terminal_size::{terminal_size, Width};

pub fn print_metadata(sf2_path: &str) {
    let mut lines = Vec::new();
    let c_path = CString::new(sf2_path).unwrap();

    unsafe {
        let handle = tsf_load_filename(c_path.as_ptr());
        if handle.is_null() {
            lines.push(format!("Could not load SF2 metadata: {}", sf2_path));
        } else {
            let count = tsf_get_presetcount(handle);
            lines.push(format!("SoundFont Metadata — {} presets found:", count));

            for idx in 0..count {
                let ptr = tsf_get_presetname(handle, idx);
                if !ptr.is_null() {
                    let name = CStr::from_ptr(ptr).to_string_lossy();
                    lines.push(format!("  [{}] {}", idx, name));
                }
            }

            tsf_close(handle);
        }
    }

    let term_width = terminal_size()
        .map(|(Width(w), _)| w as usize)
        .unwrap_or(80);

    for line in lines {
        let pad = term_width.saturating_sub(line.len());
        println!("{}{}", " ".repeat(pad), line);
    }
}

pub fn print_full_structure(soundfont_file_path: &str) -> Result<(), Box<dyn Error>> {
    let file = File::open(soundfont_file_path)?;
    let mut reader = BufReader::new(file);
    let sf2 = SoundFont::new(&mut reader)?;

    let preset = sf2
        .get_presets()
        .iter()
        .min_by_key(|p| (p.get_bank_number(), p.get_patch_number()))
        .ok_or("No presets in SoundFont")?;

    let mut lines = Vec::new();

    lines.push(format!("Preset: \"{}\"", preset.get_name()));

    lines.push(format!(
        "  ├── Bank / Patch: {} / {}",
        preset.get_bank_number(),
        preset.get_patch_number()
    ));

    let preset_zones = preset.get_regions();
    lines.push(format!("  ├── Preset Zones: {} total", preset_zones.len()));

    let mut sorted_zones: Vec<_> = preset_zones.iter().collect();
    sorted_zones.sort_by_key(|z| z.get_key_range_start());

    if let Some(zone) = sorted_zones.first() {
        // Last item at this level
        lines.push(String::from("  └── Preset Zone #0"));

        let inst_idx = zone.get_instrument_id() as usize;
        if let Some(inst) = sf2.get_instruments().get(inst_idx) {
            // Level 2 children
            lines.push(format!("      ├── Instrument: \"{}\"", inst.get_name()));

            let regions = inst.get_regions();
            lines.push(format!("      ├── Regions: {} total", regions.len()));

            if let Some(region) = regions.first() {
                // Last item at level 2
                lines.push(String::from("      └── Region #0"));

                // Level 3 children
                let kr0 = region.get_key_range_start();
                let kr1 = region.get_key_range_end();
                lines.push(format!(
                    "          ├── Key Range: {}–{} ({}–{})",
                    kr0,
                    kr1,
                    note_to_name(kr0 as u8),
                    note_to_name(kr1 as u8)
                ));

                let vr0 = region.get_velocity_range_start();
                let vr1 = region.get_velocity_range_end();
                lines.push(format!(
                    "          ├── Velocity Range: {}–{} (MIDI)",
                    vr0, vr1
                ));

                let sid = region.get_sample_id() as usize;
                if let Some(sh) = sf2.get_sample_headers().get(sid) {
                    lines.push(format!("          ├── Sample: \"{}\"", sh.get_name()));
                    lines.push(format!(
                        "          │   ├── Sample Rate: {} Hz",
                        sh.get_sample_rate()
                    ));
                    lines.push(format!(
                        "          │   ├── Loop Points: {}→{} frames",
                        sh.get_start_loop(),
                        sh.get_end_loop()
                    ));
                    lines.push(format!(
                        "          │   ├── Orig Pitch: {} ({})",
                        sh.get_original_pitch(),
                        note_to_name(sh.get_original_pitch() as u8)
                    ));
                    lines.push(format!(
                        "          │   ├── Pitch Corr: {} cents",
                        sh.get_pitch_correction()
                    ));
                    lines.push(format!(
                        "          │   └── Sample Type: {}",
                        sh.get_sample_type()
                    ));
                }

                let pan = region.get_pan();
                let pan_desc = if pan == 0.0 {
                    "Center".into()
                } else if pan < 0.0 {
                    format!("Left {}%", (pan.abs() * 100.0) as i32)
                } else {
                    format!("Right {}%", (pan * 100.0) as i32)
                };
                lines.push(format!("          ├── Pan: {}", pan_desc));

                lines.push(format!(
                    "          ├── Attack Time: {:.3}s",
                    region.get_attack_volume_envelope()
                ));
                lines.push(format!(
                    "          ├── Decay Time: {:.3}s",
                    region.get_decay_volume_envelope()
                ));
                lines.push(format!(
                    "          ├── Sustain Level: {:.1}dB",
                    region.get_sustain_volume_envelope()
                ));
                lines.push(format!(
                    "          └── Release Time: {:.3}s",
                    region.get_release_volume_envelope()
                ));
            }
        }
    }
    print_aligned_right(&lines);
    Ok(())
}

pub fn print_aligned_right(lines: &[String]) {
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
