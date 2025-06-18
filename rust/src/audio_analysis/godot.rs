#[cfg(all(not(target_arch = "wasm32"), not(target_os = "linux")))]
use aubio_rs::{OnsetMode::SpecFlux, Smpl, Tempo};
#[cfg(all(not(target_arch = "wasm32"), not(target_os = "linux")))]
use godot::global::godot_print;
#[cfg(all(not(target_arch = "wasm32"), not(target_os = "linux")))]
use hound::WavReader;
#[cfg(all(not(target_arch = "wasm32"), not(target_os = "linux")))]
use std::io::Cursor;

#[cfg(all(not(target_arch = "wasm32"), not(target_os = "linux")))]
const BUF_SIZE: usize = 1024;
#[cfg(all(not(target_arch = "wasm32"), not(target_os = "linux")))]
const HOP_SIZE: usize = 512;
#[cfg(all(not(target_arch = "wasm32"), not(target_os = "linux")))]
const I16_TO_SMPL: Smpl = 1.0 / (i16::MAX as Smpl);

#[cfg(any(target_arch = "wasm32", target_os = "linux"))]
pub fn detect_bpm_aubio_wav(_pcm_bytes: &[u8]) -> f32 {
    0.0
}
#[cfg(any(target_arch = "wasm32", target_os = "linux"))]
pub fn detect_bpm_aubio_ogg(_pcm_bytes: &[u8]) -> f32 {
    0.0
}
#[cfg(any(target_arch = "wasm32", target_os = "linux"))]
pub fn detect_bpm_from_beat_detector(_pcm_bytes: &[u8]) -> f32 {
    0.0
}

#[cfg(all(not(target_arch = "wasm32"), not(target_os = "linux")))]
pub fn detect_bpm_aubio_wav(pcm_bytes: &[u8]) -> f32 {
    let mut reader = match WavReader::new(Cursor::new(pcm_bytes)) {
        Ok(r) => r,
        Err(e) => {
            godot_print!("detect_bpm_aubio: failed to parse PCM bytes: {}", e);
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

#[cfg(all(not(target_arch = "wasm32"), not(target_os = "linux")))]
const REFERENCE_SAMPLE_RATE: f32 = 44_100.0;
#[cfg(all(not(target_arch = "wasm32"), not(target_os = "linux")))]
use lewton::inside_ogg::OggStreamReader;
#[cfg(all(not(target_arch = "wasm32"), not(target_os = "linux")))]
pub fn detect_bpm_aubio_ogg(ogg_bytes: &[u8]) -> f32 {
    let mut ogg = match OggStreamReader::new(Cursor::new(ogg_bytes)) {
        Ok(r) => r,
        Err(e) => {
            godot_print!("OGG BPM: failed to parse OGG: {:?}", e);
            return 0.0;
        }
    };
    // TODO: This is the WAV -> OGG compression details
    //  ffmpeg -i in.wav -c:a libvorbis -qscale:a 0.1 -ar 12000 -ac 1 -compression_level 10 out.ogg
    let channels = ogg.ident_hdr.audio_channels as usize;
    let ogg_sample_rate_f32 = ogg.ident_hdr.audio_sample_rate as f32;
    let ogg_sample_rate_u32 = ogg.ident_hdr.audio_sample_rate;
    let mut all_samples = Vec::new();
    while let Ok(Some(pkt)) = ogg.read_dec_packet_itl() {
        all_samples.extend(pkt);
    }
    let mut iter = all_samples.into_iter();
    let ratio = ogg_sample_rate_f32 / REFERENCE_SAMPLE_RATE;
    let ogg_hop_size = ((HOP_SIZE as f32) * ratio).round() as usize;
    let desired_buf = ((BUF_SIZE as f32) * ratio).round() as usize;
    let mut ogg_buffer_size = desired_buf.next_power_of_two();
    if ogg_buffer_size < ogg_hop_size {
        ogg_buffer_size = ogg_hop_size.next_power_of_two();
    }
    let mut tempo = match Tempo::new(SpecFlux, ogg_buffer_size, ogg_hop_size, ogg_sample_rate_u32) {
        Ok(t) => t,
        Err(e) => {
            godot_print!("OGG BPM: Tempo init failed: {}", e);
            return 0.0;
        }
    };
    let mut in_data = vec![0.0 as Smpl; ogg_hop_size];
    let mut out_data = vec![0.0 as Smpl; ogg_hop_size];
    let mut bpm = 0.0_f32;
    'outer: loop {
        for i in 0..ogg_hop_size {
            let mut sum = 0_i32;
            for _ in 0..channels {
                match iter.next() {
                    Some(s) => sum += s as i32,
                    None => break 'outer,
                }
            }
            in_data[i] = (sum as Smpl / channels as Smpl) * I16_TO_SMPL;
        }
        tempo
            .do_(in_data.as_slice(), out_data.as_mut_slice())
            .unwrap();
        bpm = tempo.get_bpm();
    }

    bpm
}

#[cfg(all(not(target_arch = "wasm32"), not(target_os = "linux")))]
pub fn detect_bpm_from_beat_detector(pcm_bytes: &[u8]) -> f32 {
    let mut reader = match WavReader::new(Cursor::new(pcm_bytes)) {
        Ok(r) => r,
        Err(e) => {
            godot_print!("detect_bpm_from_bytes: failed to parse PCM bytes: {}", e);
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
