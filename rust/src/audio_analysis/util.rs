use crate::midi::midi::MidiNote;
use aubio_rs::OnsetMode::SpecFlux;
use aubio_rs::{Onset, Pitch, PitchMode, PitchUnit};
use aubio_rs::{Smpl, Tempo};
use biquad::{Biquad, Coefficients, DirectForm1, ToHertz, Type, Q_BUTTERWORTH_F32};
use godot::builtin::{GString, PackedFloat32Array};
use godot::global::godot_print;
use godot::prelude::{Dictionary, PackedVector2Array, Vector2, Vector2i};
use hound::WavReader;
use hound::{SampleFormat, WavSpec, WavWriter};
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

pub fn _detect_bpm_accurate(path: GString) -> f32 {
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
    let sample_rate = spec.sample_rate as Smpl;
    let mut tempo = Tempo::new(SpecFlux, BUF_SIZE, HOP_SIZE, spec.sample_rate)
        .expect("couldn't create aubio Tempo");
    let mut in_data = vec![0.0 as Smpl; HOP_SIZE];
    let mut out_data = vec![0.0 as Smpl; HOP_SIZE];
    let mut samples = reader.samples::<i16>();
    let mut prev_last: Option<usize> = None;
    let mut bpms: Vec<f32> = Vec::new();
    'outer: loop {
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
        tempo
            .do_(in_data.as_slice(), out_data.as_mut_slice())
            .expect("tempo.do_ failed");
        let last = tempo.get_last();
        if let Some(prev) = prev_last {
            let delta_s = (last - prev) as Smpl / sample_rate;
            if delta_s > 0.0 {
                bpms.push(60.0 / delta_s);
            }
        }
        prev_last = Some(last);
    }
    if bpms.is_empty() {
        let fallback = tempo.get_bpm();
        if fallback > 0.0 {
            fallback
        } else {
            godot_print!("detect_bpm: no beats detected, defaulting to 60 BPM");
            60.0
        }
    } else {
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
    let chunk = (len + seg - 1) / seg;
    let mut out = PackedFloat32Array::new();
    out.resize(seg);
    for i in 0..seg {
        let start = i * chunk;
        let end = ((i + 1) * chunk).min(len);
        if start >= end {
            out.insert(i, 0.0);
            continue;
        }
        let sum: f32 = data[start..end].iter().map(|v| v.abs()).sum();
        let avg = sum / ((end - start) as f32);
        out.insert(i, avg);
    }
    out
}

pub fn band_pass_filter(path: GString, center_hz: f32, out_path: GString) {
    let infile = path.to_string();
    let outfile = out_path.to_string();
    let mut reader = match WavReader::open(&infile) {
        Ok(r) => r,
        Err(e) => {
            godot_print!("band_pass_filter: failed to open '{}': {}", infile, e);
            return;
        }
    };
    let spec = reader.spec();
    let sr = spec.sample_rate;
    let channels = spec.channels as usize;
    let out_spec = WavSpec {
        channels: 1,
        sample_rate: sr,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };
    let mut writer = match WavWriter::create(&outfile, out_spec) {
        Ok(w) => w,
        Err(e) => {
            godot_print!("band_pass_filter: failed to create '{}': {}", outfile, e);
            return;
        }
    };
    let coeffs = Coefficients::<f32>::from_params(
        Type::BandPass,
        sr.hz(),
        center_hz.hz(),
        Q_BUTTERWORTH_F32,
    )
    .expect("Invalid filter params");
    let mut filter = DirectForm1::<f32>::new(coeffs);

    let mut samples = reader.samples::<i16>();
    'proc: loop {
        let mut sum = 0i32;
        for _ in 0..channels {
            match samples.next() {
                Some(Ok(s)) => sum += s as i32,
                _ => break 'proc,
            }
        }
        let mono = (sum as f32 / channels as f32) * I16_TO_SMPL;
        let filtered = filter.run(mono);
        let out_samp = (filtered / I16_TO_SMPL).clamp(i16::MIN as f32, i16::MAX as f32) as i16;
        if let Err(e) = writer.write_sample(out_samp) {
            godot_print!("band_pass_filter: write error: {}", e);
            break;
        }
    }
    writer.finalize().ok();
}

pub fn _extract_pitch_contour(path: GString) -> PackedFloat32Array {
    let infile = path.to_string();
    let mut reader = match WavReader::open(&infile) {
        Ok(r) => r,
        Err(e) => {
            godot_print!("extract_pitch_contour: failed to open '{}': {}", infile, e);
            return PackedFloat32Array::new();
        }
    };
    let spec = reader.spec();
    let sr = spec.sample_rate;
    let channels = spec.channels as usize;
    let mut pitch =
        Pitch::new(PitchMode::Yin, BUF_SIZE, HOP_SIZE, sr).expect("couldn't create aubio Pitch");
    pitch.set_unit(PitchUnit::Hz);
    let mut in_data = vec![0.0f32; HOP_SIZE];
    let mut out_data = vec![0.0f32; HOP_SIZE];
    let mut samples = reader.samples::<i16>();
    let mut freqs = Vec::new();
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
        pitch
            .do_(in_data.as_slice(), out_data.as_mut_slice())
            .expect("pitch.do failed");
        freqs.push(pitch.get_confidence());
    }
    let mut arr = PackedFloat32Array::new();
    arr.resize(freqs.len());
    let mut arr = PackedFloat32Array::new();
    for &f in freqs.iter() {
        arr.push(f);
    }
    arr
}

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
