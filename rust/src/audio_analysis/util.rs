#[cfg(feature = "aubio")]
use aubio_rs::{OnsetMode::SpecFlux, Smpl, Tempo};
#[cfg(feature = "aubio")]
use hound::WavReader;
#[cfg(feature = "aubio")]
use std::io::Cursor;

// Logging macro that uses godot_print when available, eprintln otherwise
#[cfg(all(feature = "aubio", feature = "godot"))]
macro_rules! log_error {
    ($($arg:tt)*) => { godot::global::log_error!($($arg)*) };
}
#[cfg(all(feature = "aubio", not(feature = "godot")))]
macro_rules! log_error {
    ($($arg:tt)*) => { eprintln!($($arg)*) };
}

#[cfg(feature = "aubio")]
const BUF_SIZE: usize = 1024;
#[cfg(feature = "aubio")]
const HOP_SIZE: usize = 512;
#[cfg(feature = "aubio")]
const I16_TO_SMPL: Smpl = 1.0 / (i16::MAX as Smpl);

#[cfg(not(feature = "aubio"))]
pub fn detect_bpm_aubio_wav(_pcm_bytes: &[u8]) -> f32 {
    0.0
}
#[cfg(not(feature = "aubio"))]
pub fn detect_bpm_aubio_ogg(_pcm_bytes: &[u8]) -> f32 {
    0.0
}

#[cfg(feature = "aubio")]
pub fn detect_bpm_aubio_wav(pcm_bytes: &[u8]) -> f32 {
    let mut reader = match WavReader::new(Cursor::new(pcm_bytes)) {
        Ok(r) => r,
        Err(e) => {
            log_error!("detect_bpm_aubio: failed to parse PCM bytes: {}", e);
            return 0.0;
        },
    };
    let spec = reader.spec();
    let channels = spec.channels as usize;
    let mut tempo = Tempo::new(SpecFlux, BUF_SIZE, HOP_SIZE, spec.sample_rate).expect("couldn't create aubio Tempo");
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

#[cfg(feature = "aubio")]
const REFERENCE_SAMPLE_RATE: f32 = 44_100.0;
#[cfg(feature = "aubio")]
use lewton::inside_ogg::OggStreamReader;
#[cfg(feature = "aubio")]
pub fn detect_bpm_aubio_ogg(ogg_bytes: &[u8]) -> f32 {
    let mut ogg = match OggStreamReader::new(Cursor::new(ogg_bytes)) {
        Ok(r) => r,
        Err(e) => {
            log_error!("OGG BPM: failed to parse OGG: {:?}", e);
            return 0.0;
        },
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
            log_error!("OGG BPM: Tempo init failed: {}", e);
            return 0.0;
        },
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
        tempo.do_(in_data.as_slice(), out_data.as_mut_slice()).unwrap();
        bpm = tempo.get_bpm();
    }

    bpm
}
