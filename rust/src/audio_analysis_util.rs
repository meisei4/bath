use aubio_rs::OnsetMode::SpecFlux;
use aubio_rs::{Smpl, Tempo};
use godot::builtin::{GString, PackedFloat32Array};
use godot::global::godot_print;
use hound::WavReader;


const BUF_SIZE: usize = 1024;  // FFT window size
const HOP_SIZE: usize = 512;   // analysis hop size
const I16_TO_SMPL: Smpl    = 1.0 / (i16::MAX as Smpl);

pub fn detect_bpm(path: GString) -> f32 {
    let mut reader = match WavReader::open(path.to_string()) {
        Ok(r) => r,
        Err(e) => {
            godot_print!("detect_bpm: failed to open WAV '{}': {}", path.to_string(), e);
            return 0.0;
        }
    };
    let spec = reader.spec();
    let channels = spec.channels as usize;

    let mut tempo = Tempo::new(SpecFlux, BUF_SIZE, HOP_SIZE, spec.sample_rate)
        .expect("couldn't create aubio Tempo");

    // buffers for one hop
    let mut in_data  = vec![0.0 as Smpl; HOP_SIZE];
    let mut out_data = vec![0.0 as Smpl; HOP_SIZE];
    let mut samples  = reader.samples::<i16>();
    let mut bpm      = 0.0_f32;

    'outer: loop {
        for frame in 0..HOP_SIZE {
            let mut sum = 0i32;
            for _ in 0..channels {
                match samples.next() {
                    Some(Ok(s)) => sum += s as i32,
                    _ => break 'outer,  // EOF
                }
            }
            in_data[frame] = (sum as Smpl / channels as Smpl) * I16_TO_SMPL;
        }

        tempo.do_(in_data.as_slice(), out_data.as_mut_slice())
            .expect("tempo.do_ failed");
        bpm = tempo.get_bpm();
    }

    bpm
}


pub fn _detect_bpm_accurate(path: GString) -> f32 {
    // Open WAV file
    let mut reader = match WavReader::open(path.to_string()) {
        Ok(r) => r,
        Err(e) => {
            godot_print!("detect_bpm: failed to open WAV '{}': {}", path.to_string(), e);
            return 0.0;
        }
    };
    let spec = reader.spec();
    let channels = spec.channels as usize;
    let sample_rate = spec.sample_rate as Smpl;

    // Create aubio tempo tracker
    let mut tempo = Tempo::new(SpecFlux, BUF_SIZE, HOP_SIZE, spec.sample_rate)
        .expect("couldn't create aubio Tempo");

    // Buffers for one hop
    let mut in_data  = vec![0.0 as Smpl; HOP_SIZE];
    let mut out_data = vec![0.0 as Smpl; HOP_SIZE];
    let mut samples  = reader.samples::<i16>();

    // Collect beat‐to‐beat BPMs
    let mut prev_last: Option<usize> = None;
    let mut bpms: Vec<f32> = Vec::new();

    'outer: loop {
        // Read one hop of samples (mono mix)
        for i in 0..HOP_SIZE {
            let mut sum = 0i32;
            for _ in 0..channels {
                match samples.next() {
                    Some(Ok(s)) => sum += s as i32,
                    _ => break 'outer, // EOF
                }
            }
            in_data[i] = (sum as Smpl / channels as Smpl) * I16_TO_SMPL;
        }

        // Run aubio detection
        tempo.do_(in_data.as_slice(), out_data.as_mut_slice())
            .expect("tempo.do_ failed");

        // If a beat was detected, compute the interval BPM
        let last = tempo.get_last(); // sample index of last beat
        if let Some(prev) = prev_last {
            let delta_s = (last - prev) as Smpl / sample_rate;
            if delta_s > 0.0 {
                bpms.push(60.0 / delta_s);
            }
        }
        prev_last = Some(last);
    }

    // Determine final BPM with fallback
    if bpms.is_empty() {
        // No beats? fall back to aubio's internal estimate
        let fallback = tempo.get_bpm();
        if fallback > 0.0 {
            fallback
        } else {
            godot_print!("detect_bpm: no beats detected, defaulting to 60 BPM");
            60.0
        }
    } else {
        // Median of detected BPMs
        bpms.sort_by(|a, b| a.partial_cmp(b).unwrap());
        bpms[bpms.len() / 2]
    }
}


//TODO: this is not effective as an optimization for the sound envelope shader:
// see https://github.com/meisei4/bath/blob/main/godot/Shaders/Audio/SoundEnvelope.gd's TODO

pub fn _compute_envelope_segments(
    waveform_data: PackedFloat32Array,
    segments: i32,
) -> PackedFloat32Array {
    let data: Vec<f32> = waveform_data.to_vec();
    let seg = segments as usize;
    let len = data.len();
    let chunk = (len + seg - 1) / seg; // ceil division

    let mut out = PackedFloat32Array::new();
    out.resize(seg);

    for i in 0..seg {
        let start = i * chunk;
        let end = ((i + 1) * chunk).min(len);
        if start >= end {
            out.insert(i, 0.0);
            continue;
        }
        let sum: f32 = data[start..end]
            .iter()
            .map(|v| v.abs())
            .sum();
        let avg = sum / ((end - start) as f32);
        out.insert(i, avg);
    }

    out
}
