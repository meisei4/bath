use crate::midi::pitch::PitchDimension;
use asset_payload::payloads::{MIDI_FILE_PATH, SOUND_FONT_FILE_PATH};
use asset_payload::ResourcePaths;

use godot::builtin::{PackedByteArray, PackedVector3Array, Vector3};
use godot::classes::{AudioServer, AudioStreamWav, INode, Node};
use godot::global::godot_print;
use godot::obj::{Base, Gd};
use godot::prelude::{godot_api, GodotClass};
// godot --path . --scene Scenes/Shaders/Audio/GhostShape.tscn
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
        let sample_rate = AudioServer::singleton().get_mix_rate() as i32;
        self.inner.resolve_payload_to_midi_buffer(MIDI_FILE_PATH());
        let wav_bytes = self.inner.resolve_payload_to_pcm_buffer_cache(
            sample_rate,
            MIDI_FILE_PATH(),
            SOUND_FONT_FILE_PATH(),
            ResourcePaths::CACHED_WAV_PATH,
        );
        let buffer = PackedByteArray::from(wav_bytes);
        let stream = AudioStreamWav::load_from_buffer(&buffer).expect("Failed to decode WAV from buffer");
        self.wav_stream = Some(stream);
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
