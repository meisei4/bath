extern crate alloc;
use crate::audio_analysis::util::detect_bpm_aubio_ogg;
use alloc::vec::Vec;
use asset_payload::runtime_io::{CACHED_RHYTHM_DATA, SHADERTOY_EXPERIMENT_OGG_PATH};
use std::fs;
use std::fs::File;
use std::io::Write;

#[derive(Default)]
pub struct RhythmData {
    pub bpm: f32,
    pub uki: Vec<f32>,
    pub shizumi: Vec<f32>,
}

impl RhythmData {
    pub fn load_from_file(path: &str) -> Option<Self> {
        let bytes = fs::read(path).ok()?;
        RhythmData::deserialize(&bytes)
    }

    pub fn save_rhythm_data(&self, path: &str) {
        let bytes = self.serialize();
        let mut file = File::create(path).expect("Failed to create file");
        file.write_all(&bytes).expect("Failed to write");
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.bpm.to_le_bytes());
        let uki_len = self.uki.len() as u32;
        bytes.extend_from_slice(&uki_len.to_le_bytes());
        for val in &self.uki {
            bytes.extend_from_slice(&val.to_le_bytes());
        }
        let shizumi_len = self.shizumi.len() as u32;
        bytes.extend_from_slice(&shizumi_len.to_le_bytes());
        for val in &self.shizumi {
            bytes.extend_from_slice(&val.to_le_bytes());
        }
        bytes
    }

    pub fn deserialize(bytes: &[u8]) -> Option<Self> {
        let mut offset = 0;
        fn read_f32(bytes: &[u8], offset: &mut usize) -> Option<f32> {
            if *offset + 4 > bytes.len() {
                return None;
            }
            let val = f32::from_le_bytes(bytes[*offset..*offset + 4].try_into().unwrap());
            *offset += 4;
            Some(val)
        }

        fn read_u32(bytes: &[u8], offset: &mut usize) -> Option<u32> {
            if *offset + 4 > bytes.len() {
                return None;
            }
            let val = u32::from_le_bytes(bytes[*offset..*offset + 4].try_into().unwrap());
            *offset += 4;
            Some(val)
        }
        let bpm = read_f32(bytes, &mut offset)?;
        let uki_len = read_u32(bytes, &mut offset)? as usize;
        let mut uki = Vec::with_capacity(uki_len);
        for _ in 0..uki_len {
            uki.push(read_f32(bytes, &mut offset)?);
        }
        let shizumi_len = read_u32(bytes, &mut offset)? as usize;
        let mut shizumi = Vec::with_capacity(shizumi_len);
        for _ in 0..shizumi_len {
            shizumi.push(read_f32(bytes, &mut offset)?);
        }
        Some(Self { bpm, uki, shizumi })
    }
}

#[derive(Default)]
pub struct RhythmDimension {
    pub rhythm_data: RhythmData,
    pub bpm: f32,
    pub f_onsets_flat_buffer: Vec<[f32; 2]>,
    pub j_onsets_flat_buffer: Vec<[f32; 2]>,
    pub f_onset_count: usize,
    pub j_onset_count: usize,
    pub time_of_next_click: f32,
}

impl RhythmDimension {
    pub fn new() -> Self {
        let mut rhythm = Self::default();
        rhythm.rhythm_data = if std::path::Path::new(CACHED_RHYTHM_DATA).exists() {
            RhythmData::load_from_file(CACHED_RHYTHM_DATA).expect("Failed to load cached RhythmData")
        } else {
            RhythmData::default()
        };

        if rhythm.rhythm_data.bpm <= 0.0 {
            let audio_bytes = SHADERTOY_EXPERIMENT_OGG_PATH.as_bytes();
            rhythm.bpm = detect_bpm_aubio_ogg(audio_bytes);
            println!("Offline BPM detection → {}", rhythm.bpm);
            rhythm.rhythm_data.bpm = rhythm.bpm;
            rhythm.rhythm_data.save_rhythm_data(CACHED_RHYTHM_DATA);
        } else {
            rhythm.bpm = rhythm.rhythm_data.bpm;
            println!("Using cached BPM → {}", rhythm.bpm);
        }

        rhythm.load_custom_onsets();
        rhythm
    }

    pub fn load_custom_onsets(&mut self) {
        self.f_onsets_flat_buffer.clear();
        self.j_onsets_flat_buffer.clear();

        let uki = &self.rhythm_data.uki;
        let shizumi = &self.rhythm_data.shizumi;

        for chunk in uki.chunks(2) {
            if let [press, release] = *chunk {
                self.f_onsets_flat_buffer.push([press, release]);
            }
        }
        self.f_onset_count = self.f_onsets_flat_buffer.len();

        for chunk in shizumi.chunks(2) {
            if let [press, release] = *chunk {
                self.j_onsets_flat_buffer.push([press, release]);
            }
        }
        self.j_onset_count = self.j_onsets_flat_buffer.len();
    }

    pub fn update(&self, delta: f32, song_time: &mut f32) {
        self.debug_custom_onsets_ascii(delta, song_time);
    }

    fn debug_custom_onsets_ascii(&self, delta: f32, song_time: &mut f32) {
        let prev_time = *song_time;
        *song_time += delta;

        let f_char = ' ';
        let mut j_char = ' ';
        let f_press_fmt = String::new();
        let f_rel_fmt = String::new();
        let mut j_press_fmt = String::new();
        let mut j_rel_fmt = String::new();

        for v in &self.f_onsets_flat_buffer {
            let [start, end] = *v;
            if prev_time < start && *song_time >= start {
                j_char = 'J';
                j_press_fmt = format!("J_PRS:[{:.3},      ]", start);
            }
            if prev_time < end && *song_time >= end {
                j_rel_fmt = format!("J_REL:[{:.3}, {:.3}]", start, end);
            }
        }

        for v in &self.j_onsets_flat_buffer {
            let [start, end] = *v;
            if prev_time < start && *song_time >= start {
                j_char = 'J';
                j_press_fmt = format!("J_PRS:[{:.3},      ]", start);
            }
            if prev_time < end && *song_time >= end {
                j_rel_fmt = format!("J_REL:[{:.3}, {:.3}]", start, end);
            }
        }

        let event_body = format!("{}{}{}{}", f_press_fmt, f_rel_fmt, j_press_fmt, j_rel_fmt);
        if !event_body.is_empty() {
            let status_body = format!("[{}] [{}]   {}", f_char, j_char, event_body);
            println!("{}", status_body);
        }
    }
}
