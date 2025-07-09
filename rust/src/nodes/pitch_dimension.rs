use crate::midi::pitch::PitchDimension;
use crate::nodes::audio_pool_manager::AudioPoolManagerRust;
use asset_loader::runtime_io::{CACHED_WAV, MIDI_FILE_PATH, SOUND_FONT_FILE_PATH};
use godot::builtin::{PackedByteArray, PackedVector3Array, Vector3};
use godot::classes::{AudioServer, AudioStreamWav, Node};
use godot::global::godot_print;
use godot::obj::{Base, Gd};
use godot::register::{godot_api, GodotClass};
use std::fs;

#[derive(GodotClass)]
#[class(init, base=Node)]
pub struct PitchDimensionGodot {
    #[base]
    base: Base<Node>,
    inner: PitchDimension,
    song_time: f32,
    wav_stream: Option<Gd<AudioStreamWav>>,
}

#[godot_api]
impl PitchDimensionGodot {
    #[func]
    pub fn ready(&mut self) {
        let wav_bytes = match fs::read(CACHED_WAV) {
            Ok(bytes) => bytes,
            Err(_) => {
                let sample_rate = AudioServer::singleton().get_mix_rate() as i32;
                let bytes = self.inner.render_midi_to_sound_bytes_constant_time(
                    sample_rate,
                    MIDI_FILE_PATH,
                    SOUND_FONT_FILE_PATH,
                );
                if let Some(dir) = std::path::Path::new(CACHED_WAV).parent() {
                    let _ = fs::create_dir_all(dir);
                }
                fs::write(CACHED_WAV, &bytes).expect("write WAV cache");
                bytes
            },
        };

        let gd_wav_bytes = PackedByteArray::from(wav_bytes);
        self.wav_stream = AudioStreamWav::load_from_buffer(&gd_wav_bytes);
        AudioPoolManagerRust::singleton()
            .bind_mut()
            .play_music(self.wav_stream.clone().unwrap().upcast(), 0.0);
    }

    #[func]
    pub fn process(&mut self, delta: f64) {
        self.song_time += delta as f32;
        self.inner.update_hsv_buffer(self.song_time);
    }

    #[func]
    pub fn get_hsv_buffer(&self) -> PackedVector3Array {
        let mut out = PackedVector3Array::new();
        for [h, s, v] in self.inner.get_hsv_buffer() {
            out.push(Vector3::new(h, s, v));
        }
        out
    }

    #[func]
    fn debug_print_cwd(&self) {
        godot_print!("cwd = {:?}", std::env::current_dir().unwrap());
    }
}
