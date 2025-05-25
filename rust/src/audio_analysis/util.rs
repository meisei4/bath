use crate::midi::midi::MidiNote;
use aubio_rs::Onset;
use aubio_rs::OnsetMode::SpecFlux;
use aubio_rs::{Smpl, Tempo};
use godot::builtin::{GString, PackedFloat32Array};
use godot::global::godot_print;
use godot::prelude::{Dictionary, PackedVector2Array, Vector2, Vector2i};
use hound::WavReader;
use std::collections::HashMap;
//use rspleeter::{split_pcm_audio, SpleeterModelInfo};

const BUF_SIZE: usize = 1024; // FFT window size
const HOP_SIZE: usize = 512; // analysis hop size
const I16_TO_SMPL: Smpl = 1.0 / (i16::MAX as Smpl);

pub fn detect_bpm(path: GString) -> f32 {
    let mut reader = match WavReader::open(path.to_string()) {
        Ok(r) => r,
        Err(e) => {
            godot_print!(
                "detect_bpm: failed to open WAV '{}': {}",
                path.to_string(),
                e
            );
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

//TODO: need to work on an optimization for the sound envelope shader:
// see https://github.com/meisei4/bath/blob/main/godot/Shaders/Audio/SoundEnvelope.gd's TODO

pub fn extract_onset_times(path: GString) -> PackedFloat32Array {
    let infile = path.to_string();
    let mut reader = match WavReader::open(&infile) {
        Ok(r) => r,
        Err(e) => {
            godot_print!("extract_onset_times: failed to open '{}': {}", infile, e);
            return PackedFloat32Array::new();
        }
    };
    let spec = reader.spec();
    let sr = spec.sample_rate;
    let channels = spec.channels as usize;
    let mut onset =
        Onset::new(SpecFlux, BUF_SIZE, HOP_SIZE, sr).expect("couldn't create aubio Onset");
    let mut in_data = vec![0.0f32; HOP_SIZE];
    let mut out_data = vec![0.0f32; HOP_SIZE];
    let mut samples = reader.samples::<i16>();
    let mut elapsed = 0usize;
    let mut times = Vec::new();
    'outer: loop {
        for i in 0..HOP_SIZE {
            let mut sum = 0i32;
            for _ in 0..channels {
                match samples.next() {
                    Some(Ok(s)) => sum += s as i32,
                    _ => break 'outer,
                }
            }
            in_data[i] = (sum as Smpl / channels as Smpl) * I16_TO_SMPL;
        }
        onset
            .do_(in_data.as_slice(), out_data.as_mut_slice())
            .expect("onset.do failed");
        if out_data[0] > 0.0 {
            times.push(elapsed as f32 / sr as f32);
        }
        elapsed += HOP_SIZE;
    }
    let mut arr = PackedFloat32Array::new();
    arr.resize(times.len());
    for &t in times.iter() {
        arr.push(t);
    }
    arr
}
