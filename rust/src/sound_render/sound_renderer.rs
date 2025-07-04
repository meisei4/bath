use crate::sound_render::util::{MDN_MAX_AUDIO_DECIBEL, MDN_MIN_AUDIO_DECIBEL};

pub const TEXTURE_HEIGHT: i32 = 1_i32;
pub const BUFFER_SIZE: usize = 512_usize;
pub const MDN_BINS_F: f32 = 1024_f32;
pub const FFT_ROW: i32 = 0_i32;
pub const DEAD_CHANNEL: f32 = 0_f32;
pub const SAMPLE_RATE: f32 = 44_100_f32;

pub const HALF_SAMPLE_RATE: f32 = SAMPLE_RATE / 2_f32;
pub const HZ_STEP: f32 = HALF_SAMPLE_RATE / MDN_BINS_F;
pub const INVERSE_DECIBEL_RANGE: f32 = 1_f32 / (MDN_MAX_AUDIO_DECIBEL - MDN_MIN_AUDIO_DECIBEL);

//TODO: dafuq
// https://github.com/godotengine/godot/blob/master/core/math/math_funcs.h#L611
pub const K: f64 = 20_f64 / std::f64::consts::LN_10;

pub trait FFTTexture {
    type Image;
    type FFTData;
    type AudioEffect;
    fn resize_buffer(&mut self, fft_data: &mut Self::FFTData);
    fn init_audio_texture(&mut self) -> Self::Image;
    fn fetch_spectrum_analyzer(&mut self) -> Self::AudioEffect;
    fn update_audio_texture(&mut self, fft_data: &mut Self::FFTData, audio_texture: &mut Self::Image);
}

pub trait WaveformTexture {
    type Image;
    type WaveformData;
    type AudioEffect;
    fn resize_buffer(&mut self, waveform_data: &mut Self::WaveformData);
    fn init_audio_texture(&mut self) -> Self::Image;
    fn fetch_waveform_capture(&mut self) -> Self::AudioEffect;
    fn update_audio_texture(
        &mut self,
        waveform_capture: &mut Self::AudioEffect,
        waveform_data: &mut Self::WaveformData,
        audio_texture: &mut Self::Image,
    );
}
