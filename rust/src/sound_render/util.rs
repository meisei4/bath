pub const MDN_SMOOTHING: f32 = 0.8;

pub fn compute_smooth_energy(previous_smooth_energy: f32, new_normalized_energy: f32) -> f32 {
    MDN_SMOOTHING * previous_smooth_energy + (1.0 - MDN_SMOOTHING) * new_normalized_energy
}
