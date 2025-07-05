use crate::audio_analysis::fftw::{fftw_complex, fftw_create_plan, fftw_direction, fftw_one, fftw_plan};
use crate::sound_render::sound_renderer::{
    FFTTexture, BUFFER_SIZE, DEAD_CHANNEL, FFT_ROW, FFT_WINDOW_SIZE, HALF_SAMPLE_RATE, HZ_STEP, INVERSE_DECIBEL_RANGE,
    K, MDN_MIN_AUDIO_DECIBEL, TEXTURE_HEIGHT, WINDOW_TIME,
};
use crate::sound_render::util::compute_smooth_energy;
use raylib::color::Color;
use raylib::math::Vector4;
use raylib::texture::Image;

pub struct RaylibFFTTexture {
    pub plan: Option<fftw_plan>,
    pub spectrum: [fftw_complex; FFT_WINDOW_SIZE],
    pub fft_history: Vec<[f32; BUFFER_SIZE]>,
    pub history_pos: usize,
    pub last_fft_time: f64,
    pub tapback_pos: f32,
}

impl RaylibFFTTexture {
    pub fn capture_frame(&mut self, fft_data: &mut [f32; FFT_WINDOW_SIZE]) {
        let mut input = [fftw_complex { re: 0_f64, im: 0_f64 }; FFT_WINDOW_SIZE];
        let mut output = self.fetch_spectrum_analyzer();
        for i in 0_usize..FFT_WINDOW_SIZE {
            input[i].re = fft_data[i] as f64;
            input[i].im = 0_f64;
        }
        unsafe {
            if self.plan.is_none() {
                let plan = fftw_create_plan(FFT_WINDOW_SIZE as i32, fftw_direction::FFTW_FORWARD, 0);
                self.plan = Some(plan);
            }
            fftw_one(self.plan.unwrap(), input.as_mut_ptr(), output.as_mut_ptr());
        }
        let mut smoothed_spectrum = [0.0f32; BUFFER_SIZE];

        for bin in 0_usize..BUFFER_SIZE {
            let freq_low = bin as f32 * HZ_STEP;
            let freq_high = (bin as f32 + 1.0) * HZ_STEP;
            let mut bin_low = (freq_low * FFT_WINDOW_SIZE as f32 / HALF_SAMPLE_RATE).floor();
            let mut bin_high = (freq_high * FFT_WINDOW_SIZE as f32 / HALF_SAMPLE_RATE).ceil();
            bin_low = bin_low.clamp(0_f32, (FFT_WINDOW_SIZE - 1) as f32);
            bin_high = bin_high.clamp(0_f32, (FFT_WINDOW_SIZE - 1) as f32);
            if bin_low > bin_high {
                std::mem::swap(&mut bin_low, &mut bin_high);
            }
            let lo = bin_low as i32;
            let hi = bin_high as i32;
            let mut magnitude_sum = 0_f64;
            for i in lo..=hi {
                let sample = &output[i as usize];
                let magnitude = (sample.re * sample.re + sample.im * sample.im).sqrt() / (FFT_WINDOW_SIZE as f64);
                magnitude_sum += magnitude;
            }
            let bin_span = (hi - lo + 1_i32) as f64;
            let linear_magnitude = if bin_span > 0_f64 {
                magnitude_sum / bin_span
            } else {
                0_f64
            };
            let db = (linear_magnitude.max(f64::MIN_POSITIVE).ln() * K) as f32;
            let normalized = ((db - MDN_MIN_AUDIO_DECIBEL) * INVERSE_DECIBEL_RANGE).clamp(0_f32, 1_f32);
            let previous_smooth_energy = self.fft_history[self.history_pos][bin];
            let smooth_energy = compute_smooth_energy(previous_smooth_energy, normalized);
            smoothed_spectrum[bin] = smooth_energy;
        }
        let now = std::time::Instant::now().elapsed().as_secs_f64();
        self.last_fft_time = now;
        self.fft_history[self.history_pos] = smoothed_spectrum;
        self.history_pos = (self.history_pos + 1) % self.fft_history.len();
        self.spectrum = output;
    }

    pub fn render_frame(&self, texture: &mut Image) {
        let now = std::time::Instant::now().elapsed().as_secs_f64();
        let tapback_time = now - self.tapback_pos as f64;
        let frames_since_tapback = ((now - tapback_time) / WINDOW_TIME)
            .floor()
            .clamp(0_f64, (self.fft_history.len() - 1) as f64) as isize;
        let history_position =
            (self.history_pos as isize - 1 - frames_since_tapback).rem_euclid(self.fft_history.len() as isize) as usize;
        let spectrum_to_draw = &self.fft_history[history_position];
        for (bin, &amplitude) in spectrum_to_draw.iter().enumerate() {
            let color = Color::color_from_normalized(Vector4::new(amplitude, DEAD_CHANNEL, DEAD_CHANNEL, DEAD_CHANNEL));
            texture.draw_pixel(bin as i32, FFT_ROW, color);
        }
    }
}

impl FFTTexture for RaylibFFTTexture {
    type Image = Image;
    type FFTData = [f32; FFT_WINDOW_SIZE];
    type AudioEffect = [fftw_complex; FFT_WINDOW_SIZE];

    fn resize_buffer(&mut self, _fft_data: &mut Self::FFTData) {
        /* no op */
    }

    fn init_audio_texture(&mut self) -> Self::Image {
        Image::gen_image_color(BUFFER_SIZE as i32, TEXTURE_HEIGHT, Color::WHITE)
    }

    fn fetch_spectrum_analyzer(&mut self) -> Self::AudioEffect {
        self.spectrum
    }

    fn update_audio_texture(&mut self, fft_data: &mut Self::FFTData, audio_texture: &mut Self::Image) {
        self.capture_frame(fft_data);
        self.render_frame(audio_texture);
    }
}
