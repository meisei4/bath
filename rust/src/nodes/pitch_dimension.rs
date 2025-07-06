use crate::midi::pitch::PitchDimension;
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
    pub fn _ready(&mut self) {
        //TODO: figure out file stuff oh my goodness i hate this part
        // notes: web build requires res://,
        // no_std will be literal bytes loaded at compile time for game include_bytes!() or something
        let midi_path = "";
        let sf2_path = "";
        let wav_cache_path = "";

        self.inner.load_midi_to_buffer(midi_path);

        let wav_bytes = if let Ok(bytes) = fs::read(wav_cache_path) {
            bytes
        } else {
            let sample_rate = AudioServer::singleton().get_mix_rate() as i32;
            let bytes = self
                .inner
                .render_midi_to_sound_bytes_constant_time(sample_rate, midi_path, sf2_path);
            let _ = fs::write(wav_cache_path, &bytes);
            bytes
        };

        self.wav_stream = AudioStreamWav::load_from_buffer(&PackedByteArray::from(wav_bytes));
    }

    #[func]
    pub fn _process(&mut self, delta: f32) {
        self.song_time += delta;
        self.inner.update_hsv_buffer(self.song_time);
    }

    #[func]
    pub fn get_hsv_buffer(&self) -> PackedVector3Array {
        let mut array = PackedVector3Array::new();
        for [h, s, v] in self.inner.get_hsv_buffer().iter() {
            array.push(Vector3::new(*h, *s, *v));
        }
        array
    }

    #[func]
    fn debug_print_cwd(&self) {
        let cwd = std::env::current_dir().unwrap();
        godot_print!("Working directory: {:?}", cwd);
    }
}
