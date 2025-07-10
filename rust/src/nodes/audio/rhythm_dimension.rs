use crate::midi::rhythm::RhythmDimension;
use godot::builtin::{PackedVector2Array, Vector2};
use godot::classes::{INode, Node};
use godot::obj::Base;
use godot::prelude::{godot_api, GodotClass};

#[derive(GodotClass)]
#[class(init, base=Node)]
pub struct RhythmDimensionGodot {
    #[base]
    base: Base<Node>,
    inner: RhythmDimension,
    #[export]
    song_time: f32,
}

#[godot_api]
impl INode for RhythmDimensionGodot {
    fn process(&mut self, delta: f64) {
        self.inner.update(delta as f32, &mut self.song_time);
    }

    fn ready(&mut self) {
        self.inner = RhythmDimension::new();
    }
}

#[godot_api]
impl RhythmDimensionGodot {
    #[func]
    pub fn get_bpm(&self) -> f32 {
        self.inner.bpm
    }

    #[func]
    pub fn get_f_onset_count(&self) -> i32 {
        self.inner.f_onset_count as i32
    }

    #[func]
    pub fn get_j_onset_count(&self) -> i32 {
        self.inner.j_onset_count as i32
    }

    #[func]
    pub fn get_f_onsets(&self) -> PackedVector2Array {
        let mut arr = PackedVector2Array::new();
        for [start, end] in &self.inner.f_onsets_flat_buffer {
            arr.push(Vector2::new(*start, *end));
        }
        arr
    }

    #[func]
    pub fn get_j_onsets(&self) -> PackedVector2Array {
        let mut arr = PackedVector2Array::new();
        for [start, end] in &self.inner.j_onsets_flat_buffer {
            arr.push(Vector2::new(*start, *end));
        }
        arr
    }

    #[func]
    pub fn reset_song_time(&mut self) {
        self.song_time = 0.0;
    }
}
