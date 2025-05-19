use crate::keys::note_to_name;
use crate::tsf_bindings;
use soundfont::data::{GeneratorType, Info as SoundFontInfo, PresetHeader, SampleHeader};
use soundfont::SoundFont2;
use std::{
    error::Error,
    ffi::{CStr, CString},
    fs::File,
    io::BufReader,
};
use terminal_size::{terminal_size, Width};

pub fn print_metadata(sf2_path: &str) {
    let c_path = CString::new(sf2_path).unwrap();
    unsafe {
        let handle = tsf_bindings::tsf_load_filename(c_path.as_ptr());
        if handle.is_null() {
            eprintln!("Could not load SF2 metadata: {}", sf2_path);
        } else {
            let count = tsf_bindings::tsf_get_presetcount(handle);
            println!("SoundFont Metadata — {} presets found:", count);
            for idx in 0..count {
                let ptr = tsf_bindings::tsf_get_presetname(handle, idx);
                if !ptr.is_null() {
                    let name = CStr::from_ptr(ptr).to_string_lossy();
                    println!("  [{}] {}", idx, name);
                }
            }
            tsf_bindings::tsf_close(handle);
        }
    }
}

pub fn print_full_structure(sf2_path: &str) -> Result<(), Box<dyn Error>> {
    let file = File::open(sf2_path)?;
    let mut reader = BufReader::new(file);
    let mut sf2 = SoundFont2::load(&mut reader)?;
    sf2 = sf2.sort_presets();

    let mut lines = Vec::new();
    lines.extend(assemble_info_lines(
        &sf2.info,
        sf2.presets.len(),
        sf2.sample_headers.len(),
    ));
    lines.push(String::new());
    if let Some(preset) = sf2.presets.first() {
        lines.extend(assemble_preset_lines(
            &preset.header,
            &preset.zones[0],
            &sf2.sample_headers,
        ));
    }

    print_right_aligned(&lines);
    Ok(())
}

fn assemble_info_lines(
    info: &SoundFontInfo,
    total_presets: usize,
    total_samples: usize,
) -> Vec<String> {
    vec![
        format!("┌── SoundFont: \"{}\"", info.bank_name),
        format!("│   ├── Engine:          {:?}", info.sound_engine),
        format!(
            "│   ├── Version:         {}.{}",
            info.version.major, info.version.minor
        ),
        format!("│   ├── Presets:         {} total", total_presets),
        format!("│   └── Samples:         {} total", total_samples),
    ]
}

fn assemble_preset_lines(
    header: &PresetHeader,
    zone: &soundfont::Zone,
    sample_headers: &[SampleHeader],
) -> Vec<String> {
    let mut out = Vec::new();
    out.push(format!("┌── Preset: \"{}\"", header.name));
    out.push(format!(
        "│   ├── Bank / Program:   {} / {}",
        header.bank, header.preset
    ));
    // preserve original behavior: always “1” zone for this debug view
    out.push("│   ├── Zones:            1 total".into());
    out.push("│   └── Zone #0".into());

    if let Some(gen) = zone
        .gen_list
        .iter()
        .find(|g| g.ty.into_result().ok() == Some(GeneratorType::KeyRange))
    {
        if let Some(r) = gen.amount.as_range() {
            out.push(format!(
                "│       ├── Key Range:      {} – {} ({} – {})",
                r.low,
                r.high,
                note_to_name(r.low as u8),
                note_to_name(r.high as u8)
            ));
        } else {
            out.push("│       ├── Key Range:      N/A".into());
        }
    } else {
        out.push("│       ├── Key Range:      N/A".into());
    }

    if let Some(gen) = zone
        .gen_list
        .iter()
        .find(|g| g.ty.into_result().ok() == Some(GeneratorType::VelRange))
    {
        if let Some(r) = gen.amount.as_range() {
            out.push(format!(
                "│       ├── Velocity Range: {} – {} (MIDI)",
                r.low, r.high
            ));
        } else {
            out.push("│       ├── Velocity Range: N/A".into());
        }
    } else {
        out.push("│       ├── Velocity Range: N/A".into());
    }

    if let Some(gen) = zone
        .gen_list
        .iter()
        .find(|g| g.ty.into_result().ok() == Some(GeneratorType::SampleID))
    {
        if let Some(&sid) = gen.amount.as_u16() {
            out.extend(format_sample_info(sid as usize, sample_headers));
        } else {
            out.push("│       ├── Sample:         N/A".into());
        }
    } else {
        out.push("│       ├── Sample:         N/A".into());
    }

    let mut env_lines = Vec::new();
    for g in &zone.gen_list {
        if let Ok(gt) = g.ty.into_result() {
            if let Some(&v) = g.amount.as_i16() {
                if let Some(line) = format_env_pan(gt, v) {
                    env_lines.push(line);
                }
            }
        }
    }
    if env_lines.is_empty() {
        out.push("│       ├── Generators:     N/A".into());
    } else {
        out.push("│       ├── Generators:".into());
        out.extend(env_lines);
    }

    out
}

fn format_sample_info(index: usize, headers: &[SampleHeader]) -> Vec<String> {
    let sh = &headers[index];
    let mut lines = Vec::new();
    lines.push(format!("│       ├── Sample:         \"{}\"", sh.name));
    lines.push(format!(
        "│       │   ├── Rate:       {:>7} Hz",
        sh.sample_rate
    ));
    lines.push(format!(
        "│       │   ├── Loop:       {} → {} frames",
        sh.loop_start, sh.loop_end
    ));
    lines.push(format!(
        "│       │   ├── Orig Pitch: {} ({})",
        sh.origpitch,
        note_to_name(sh.origpitch)
    ));
    lines.push(format!("│       │   ├── Pitch Adj:  {} cents", sh.pitchadj));
    let ty = if sh.sample_type.is_mono() {
        "Mono"
    } else {
        "Stereo"
    };
    lines.push(format!("│       │   └── Type:       {}", ty));
    lines
}

fn format_env_pan(gt: GeneratorType, value: i16) -> Option<String> {
    use soundfont::raw::GeneratorType::*;
    match gt {
        AttackVolEnv => Some(format!("│       │   ├── Attack Time:  {}", value)),
        DecayVolEnv => Some(format!("│       │   ├── Decay Time:   {}", value)),
        ReleaseVolEnv => Some(format!("│       │   ├── Release Time: {}", value)),
        SustainVolEnv => Some(format!("│       │   ├── Sustain:      {} dB", value)),
        Pan => {
            let desc = if value == 0 {
                "Center".to_string()
            } else if value < 0 {
                format!("Left {}%", -value)
            } else {
                format!("Right {}%", value)
            };
            Some(format!("│       │   └── Pan:          {}", desc))
        }
        _ => None,
    }
}

fn print_right_aligned(lines: &[String]) {
    let max_width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
    let term_width = terminal_size()
        .map(|(Width(w), _)| w as usize)
        .unwrap_or(80);
    let pad = term_width.saturating_sub(max_width);
    for line in lines {
        println!("{}{}", " ".repeat(pad), line);
    }
}
