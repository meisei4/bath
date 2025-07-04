use godot::classes::audio_effect_spectrum_analyzer_instance::MagnitudeMode;
use godot::classes::AudioEffectSpectrumAnalyzerInstance;
use godot::global::linear_to_db;
use godot::obj::Gd;

const MDN_MIN_AUDIO_DECIBEL: f32 = -100.0; //match WebAudio defaults
const MDN_MAX_AUDIO_DECIBEL: f32 = -30.0; //match WebAudio defaults
const MDN_SMOOTHING: f32 = 0.8;

pub fn compute_smooth_energy_for_frequency_range(
    spectrum_analyzer_instance: &Gd<AudioEffectSpectrumAnalyzerInstance>,
    from_hz: f32,
    to_hz: f32,
    _previous_smooth_energy: f32,
) -> f32 {
    let linear_average: f32 = _compute_linear_average_for_frequency_range(spectrum_analyzer_instance, from_hz, to_hz);
    let normalized: f32 = _compute_normalized_energy_from_linear_magnitude(linear_average);
    _compute_smooth_energy(_previous_smooth_energy, normalized)
}

fn _compute_linear_average_for_frequency_range(
    spectrum_analyzer_instance: &Gd<AudioEffectSpectrumAnalyzerInstance>,
    from_hz: f32,
    to_hz: f32,
) -> f32 {
    let stereo_magnitude = spectrum_analyzer_instance
        .get_magnitude_for_frequency_range_ex(from_hz, to_hz)
        .mode(MagnitudeMode::AVERAGE)
        .done();

    (stereo_magnitude.x + stereo_magnitude.y) * 0.5
}

fn _compute_normalized_energy_from_linear_magnitude(linear_magnitude: f32) -> f32 {
    let db: f32 = linear_to_db(linear_magnitude as f64) as f32;
    ((db - MDN_MIN_AUDIO_DECIBEL) / (MDN_MAX_AUDIO_DECIBEL - MDN_MIN_AUDIO_DECIBEL)).clamp(0.0, 1.0)
}

fn _compute_smooth_energy(_previous_smooth_energy: f32, new_normalized_energy: f32) -> f32 {
    MDN_SMOOTHING * _previous_smooth_energy + (1.0 - MDN_SMOOTHING) * new_normalized_energy
}
