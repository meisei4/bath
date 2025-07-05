use crate::audio_analysis::fftw::{fftw_complex, fftw_create_plan, fftw_direction, fftw_one, fftw_plan};
use crate::sound_render::sound_renderer::{
    FFTTexture, BUFFER_SIZE, DEAD_CHANNEL, FFT_ROW, HALF_SAMPLE_RATE, HZ_STEP, INVERSE_DECIBEL_RANGE, K, TEXTURE_HEIGHT,
};
use crate::sound_render::util::{compute_smooth_energy, MDN_MIN_AUDIO_DECIBEL};
use raylib::color::Color;
use raylib::math::Vector4;
use raylib::texture::Image;

pub struct RaylibFFTTexture {
    pub plan: Option<fftw_plan>,
    pub spectrum: Vec<fftw_complex>,
}

impl FFTTexture for RaylibFFTTexture {
    type Image = Image;
    type FFTData = Vec<f32>; // TODO later use [f32] but figure out the Sized constraint
    type AudioEffect = Vec<fftw_complex>;

    fn resize_buffer(&mut self, fft_data: &mut Self::FFTData) {
        fft_data.resize(BUFFER_SIZE, 0_f32);
    }

    fn init_audio_texture(&mut self) -> Self::Image {
        Image::gen_image_color(BUFFER_SIZE as i32, TEXTURE_HEIGHT, Color::WHITE)
    }

    fn fetch_spectrum_analyzer(&mut self) -> Self::AudioEffect {
        self.spectrum.clone()
    }

    fn update_audio_texture(&mut self, fft_data: &mut Self::FFTData, audio_texture: &mut Self::Image) {
        let mut input = vec![fftw_complex { re: 0_f64, im: 0_f64 }; BUFFER_SIZE];
        let mut output = self.fetch_spectrum_analyzer();
        for i in 0_usize..BUFFER_SIZE {
            input[i].re = fft_data[i] as f64;
            input[i].im = 0_f64;
        }
        unsafe {
            if self.plan.is_none() {
                let plan = fftw_create_plan(BUFFER_SIZE as i32, fftw_direction::FFTW_FORWARD, 0);
                self.plan = Some(plan);
            }
            fftw_one(self.plan.unwrap(), input.as_mut_ptr(), output.as_mut_ptr());
        }
        //let fft_size = (output.len() * 2_usize) as f32; // number of real bins
        let fft_size = (BUFFER_SIZE * 2_usize) as f32;
        for bin_index in 0_usize..BUFFER_SIZE {
            let bin_index_f = bin_index as f32;
            let freq_low_hz = bin_index_f * HZ_STEP;
            let freq_high_hz = (bin_index_f + 1_f32) * HZ_STEP;
            let mut bin_low = (freq_low_hz * fft_size / HALF_SAMPLE_RATE).floor();
            let mut bin_high = (freq_high_hz * fft_size / HALF_SAMPLE_RATE).ceil();
            bin_low = bin_low.clamp(0_f32, fft_size - 1_f32);
            bin_high = bin_high.clamp(0_f32, fft_size - 1_f32);
            if bin_low > bin_high {
                std::mem::swap(&mut bin_low, &mut bin_high);
            }
            let bin_low_i = bin_low as i32;
            let bin_high_i = bin_high as i32;
            let mut magnitude_sum = 0_f64;
            for i in bin_low_i..=bin_high_i {
                let sample = &output[i as usize];
                let magnitude = (sample.re * sample.re + sample.im * sample.im).sqrt() / fft_size as f64;
                magnitude_sum += magnitude;
            }
            let bin_span = (bin_high_i - bin_low_i + 1_i32) as f64;
            let linear_magnitude = if bin_span > 0_f64 {
                magnitude_sum / bin_span
            } else {
                0_f64
            };
            let db = (linear_magnitude.max(f64::MIN_POSITIVE).ln() * K) as f32;
            let normalized = ((db - MDN_MIN_AUDIO_DECIBEL) * INVERSE_DECIBEL_RANGE).clamp(0_f32, 1_f32);
            let previous_smooth_energy = fft_data[bin_index];
            let smooth_energy = compute_smooth_energy(previous_smooth_energy, normalized);
            fft_data[bin_index] = smooth_energy;
            let color =
                Color::color_from_normalized(Vector4::new(smooth_energy, DEAD_CHANNEL, DEAD_CHANNEL, DEAD_CHANNEL));
            audio_texture.draw_pixel(bin_index as i32, FFT_ROW, color);
        }
        self.spectrum = output;
    }
}
