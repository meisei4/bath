#[cfg(not(target_arch = "wasm32"))]
use aubio_rs::{OnsetMode::SpecFlux, Smpl, Tempo};
#[cfg(not(target_arch = "wasm32"))]
use godot::global::godot_print;
#[cfg(not(target_arch = "wasm32"))]
use hound::WavReader;
#[cfg(not(target_arch = "wasm32"))]
use std::io::Cursor;
#[cfg(not(target_arch = "wasm32"))]
const BUF_SIZE: usize = 1024;
#[cfg(not(target_arch = "wasm32"))]
const HOP_SIZE: usize = 512;
#[cfg(not(target_arch = "wasm32"))]
const I16_TO_SMPL: Smpl = 1.0 / (i16::MAX as Smpl);

#[cfg(target_arch = "wasm32")]
pub fn detect_bpm_aubio(_wav_bytes: &[u8]) -> f32 {
    0.0
}
#[cfg(target_arch = "wasm32")]
pub fn detect_bpm_from_beat_detector(_wav_bytes: &[u8]) -> f32 {
    0.0
}

#[cfg(not(target_arch = "wasm32"))]
pub fn detect_bpm_aubio(wav_bytes: &[u8]) -> f32 {
    let mut reader = match WavReader::new(Cursor::new(wav_bytes)) {
        Ok(r) => r,
        Err(e) => {
            godot_print!("detect_bpm_from_bytes: failed to parse WAV bytes: {}", e);
            return 0.0;
        }
    };
    let spec = reader.spec();
    let channels = spec.channels as usize;
    let mut tempo = Tempo::new(SpecFlux, BUF_SIZE, HOP_SIZE, spec.sample_rate)
        .expect("couldn't create aubio Tempo");
    let mut in_data = vec![0.0 as Smpl; HOP_SIZE];
    let mut out_data = vec![0.0 as Smpl; HOP_SIZE];
    let mut samples = reader.samples::<i16>();
    let mut bpm = 0.0_f32;
    'outer: loop {
        for frame in 0..HOP_SIZE {
            let mut sum = 0i32;
            for _ in 0..channels {
                match samples.next() {
                    Some(Ok(s)) => sum += s as i32,
                    _ => break 'outer, // EOF
                }
            }
            in_data[frame] = (sum as Smpl / channels as Smpl) * I16_TO_SMPL;
        }
        tempo
            .do_(in_data.as_slice(), out_data.as_mut_slice())
            .expect("tempo.do_ failed");
        bpm = tempo.get_bpm();
    }
    bpm
}

#[cfg(not(target_arch = "wasm32"))]
pub fn detect_bpm_from_beat_detector(wav_bytes: &[u8]) -> f32 {
    let mut reader = match WavReader::new(Cursor::new(wav_bytes)) {
        Ok(r) => r,
        Err(e) => {
            godot_print!("detect_bpm_from_bytes: failed to parse WAV bytes: {}", e);
            return 0.0;
        }
    };
    let spec = reader.spec();
    let sample_rate = spec.sample_rate as f32;
    let channels = spec.channels as usize;
    let mut mono_samples: Vec<i16> = Vec::new();
    let mut samples_iter = reader.samples::<i16>();
    loop {
        let mut sum: i32 = 0;
        for _ in 0..channels {
            match samples_iter.next() {
                Some(Ok(s)) => sum += s as i32,
                _ => {
                    break;
                }
            }
        }
        if sum == 0 && samples_iter.len() < channels {
            break;
        }
        let mono_i16 = (sum / (channels as i32)) as i16;
        mono_samples.push(mono_i16);
    }
    let rem = mono_samples.len() % BUF_SIZE;
    if rem != 0 {
        let pad = BUF_SIZE - rem;
        mono_samples.extend(std::iter::repeat(0).take(pad));
    }
    //let mut beat_sample_indices: Vec<u64> = Vec::new();
    let beat_sample_indices: Vec<u64> = Vec::new();
    let total_windows = mono_samples.len() / BUF_SIZE;
    //TODO: beat-detector doesnt expose detector lol, i dont want to patch it either
    //let mut detector: Box<dyn Strategy + Send> = StrategyKind::Spectrum.detector(spec.sample_rate);
    for window_idx in 0..total_windows {
        let start = window_idx * BUF_SIZE;
        let end = start + BUF_SIZE;
        let _window: &[i16] = &mono_samples[start..end];
        // if let Some(beat_info) = detector.is_beat(window) {
        //     let rel_ms = beat_info.relative_ms() as f32;
        //     let offset_samples = ((rel_ms / 1000.0) * sample_rate).round() as u64;
        //     let absolute_sample = (start as u64).saturating_add(offset_samples);
        //     beat_sample_indices.push(absolute_sample);
        // }
    }
    if beat_sample_indices.len() < 2 {
        return 0.0;
    }
    let mut total_delta: u64 = 0;
    for i in 1..beat_sample_indices.len() {
        total_delta += beat_sample_indices[i] - beat_sample_indices[i - 1];
    }
    let count = (beat_sample_indices.len() - 1) as f32;
    let avg_delta_samples = total_delta as f32 / count;
    let bpm = (sample_rate * 60.0) / avg_delta_samples;
    bpm
}
