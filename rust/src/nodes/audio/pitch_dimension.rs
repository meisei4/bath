use crate::midi::pitch::PitchDimension;
use asset_loader::runtime_io::{CACHED_WAV, MIDI_FILE_PATH, SOUND_FONT_FILE_PATH};

use godot::builtin::{PackedByteArray, PackedVector3Array, Vector3};
use godot::classes::{AudioServer, AudioStreamWav, INode, Node};
use godot::global::godot_print;
use godot::obj::{Base, Gd};
use godot::prelude::{godot_api, GodotClass};

use std::fs;
// godot --path . --scene Scenes/Audio/PitchDimension.tscn
#[derive(GodotClass)]
#[class(init, base=Node)]
pub struct PitchDimensionGodot {
    #[base]
    base: Base<Node>,
    inner: PitchDimension,
    wav_stream: Option<Gd<AudioStreamWav>>,
    #[export]
    song_time: f32,
}

#[godot_api]
impl INode for PitchDimensionGodot {
    fn process(&mut self, delta: f64) {
        self.song_time += delta as f32;
        self.inner.update_hsv_buffer(self.song_time);
    }

    fn ready(&mut self) {
        self.inner.load_midi_to_buffer(MIDI_FILE_PATH);
        if self.wav_stream.is_none() {
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
                    fs::write(CACHED_WAV, &bytes).expect("Failed to write WAV cache");
                    bytes
                },
            };

            let buffer = PackedByteArray::from(wav_bytes);
            let stream = AudioStreamWav::load_from_buffer(&buffer).expect("Failed to decode WAV from buffer");
            self.wav_stream = Some(stream);
        }
        // TODO: we need to get away from any custom singletons in the whole project please
        // let audio_pool_manager_obj = Engine::singleton().get_singleton(&StringName::from("AudioPoolManagerRust"));
        // let mut audio_pool_manager = audio_pool_manager_obj.unwrap().cast::<AudioPoolManagerRust>();
        // audio_pool_manager.bind_mut().play_music(self.wav_stream.clone().unwrap().upcast(), 0.0);
        // AudioPoolManagerRust::singleton().bind_mut().play_music(self.wav_stream.clone().unwrap().upcast(), 0.0);
    }
}

#[godot_api]
impl PitchDimensionGodot {
    #[func]
    pub fn get_hsv_buffer(&self) -> PackedVector3Array {
        let mut out = PackedVector3Array::new();
        for [h, s, v] in self.inner.get_hsv_buffer() {
            out.push(Vector3::new(h, s, v));
        }
        out
    }

    #[func]
    pub fn get_wav_stream(&self) -> Gd<AudioStreamWav> {
        self.wav_stream.clone().unwrap()
    }

    #[func]
    fn debug_print_cwd(&self) {
        godot_print!("cwd = {:?}", std::env::current_dir().unwrap());
    }
}
