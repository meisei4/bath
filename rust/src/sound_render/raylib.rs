use crate::audio_analysis::fftw::{fftw_complex, fftw_create_plan, fftw_direction, fftw_one, fftw_plan};
use crate::sound_render::sound_renderer::{FFTTexture, BUFFER_SIZE, DEAD_CHANNEL, FFT_ROW, TEXTURE_HEIGHT};
use crate::sound_render::util::{compute_smooth_energy, MDN_MAX_AUDIO_DECIBEL, MDN_MIN_AUDIO_DECIBEL};
use raylib::color::Color;
use raylib::texture::Image;

pub struct RaylibFFTTexture {
    plan: Option<fftw_plan>,
    pub spectrum: Vec<fftw_complex>,
}

impl FFTTexture for RaylibFFTTexture {
    type Image = Image;
    type FFTData = Vec<f32>;
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

    fn update_audio_texture(
        &mut self,
        spectrum: &Self::AudioEffect,
        fft_data: &mut Self::FFTData,
        audio_texture: &mut Self::Image,
    ) {
        let mut input = vec![fftw_complex { re: 0.0, im: 0.0 }; BUFFER_SIZE];
        let mut output = spectrum.clone();
        for i in 0..BUFFER_SIZE {
            input[i].re = fft_data[i] as f64;
            input[i].im = 0.0;
        }
        unsafe {
            let plan = fftw_create_plan(BUFFER_SIZE as i32, fftw_direction::FFTW_FORWARD, 0);
            self.plan = Some(plan.clone()); //TODO: WHY?
            fftw_one(plan, input.as_mut_ptr(), output.as_mut_ptr());
        }

        for bin_index in 0..BUFFER_SIZE {
            let re = output[bin_index].re;
            let im = output[bin_index].im;
            let linear_magnitude = (re * re + im * im).sqrt() as f32;
            let db = 10.0 * linear_magnitude.max(f32::EPSILON).log10();
            let normalized =
                ((db - MDN_MIN_AUDIO_DECIBEL) / (MDN_MAX_AUDIO_DECIBEL - MDN_MIN_AUDIO_DECIBEL)).clamp(0.0, 1.0);
            let previous_smooth_energy = fft_data[bin_index];
            let smooth_energy = compute_smooth_energy(previous_smooth_energy, normalized);
            fft_data[bin_index] = smooth_energy;
            let smooth_energy = (smooth_energy * 255.0) as u8;
            let color = Color::new(smooth_energy, DEAD_CHANNEL, DEAD_CHANNEL, DEAD_CHANNEL);
            audio_texture.draw_pixel(bin_index as i32, FFT_ROW, color);
        }
        self.spectrum = output;
    }
}
