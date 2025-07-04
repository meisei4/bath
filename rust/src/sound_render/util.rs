pub const MDN_SMOOTHING: f32 = 0.8;
pub const MDN_MIN_AUDIO_DECIBEL: f32 = -100.0; //match WebAudio defaults
pub const MDN_MAX_AUDIO_DECIBEL: f32 = -30.0; //match WebAudio defaults

pub fn compute_smooth_energy(previous_smooth_energy: f32, new_normalized_energy: f32) -> f32 {
    MDN_SMOOTHING * previous_smooth_energy + (1.0 - MDN_SMOOTHING) * new_normalized_energy
}

