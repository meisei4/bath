use crate::sound_render::godot_util::compute_smooth_energy_for_frequency_range;
use crate::sound_render::sound_renderer::FFTTexture;
use godot::builtin::PackedFloat32Array;
use godot::classes::image::Format;
use godot::classes::{AudioEffectSpectrumAnalyzerInstance, Image, Node};
use godot::obj::{Base, Gd, WithBaseField};
use godot::prelude::{Color, GodotClass};

#[derive(GodotClass)]
#[class(init, base=Node)]
pub struct GodotFFTTexture {
    base: Base<Node>,
}

const TEXTURE_HEIGHT: i32 = 1;
const BUFFER_SIZE: usize = 512;
const MDN_BINS_F: f32 = 1024.0;
const FFT_ROW: i32 = 0;
const DEAD_CHANNEL: f32 = 0.0;
const SAMPLE_RATE: f32 = 44_100.0;

impl FFTTexture for GodotFFTTexture {
    type Image = Gd<Image>;
    type FFTData = PackedFloat32Array;
    type SpectrumAnalyzer = Gd<AudioEffectSpectrumAnalyzerInstance>;

    fn resize_buffer(&mut self, fft_data: &mut PackedFloat32Array) {
        fft_data.resize(BUFFER_SIZE);
    }

    fn init_audio_texture(&mut self) -> Self::Image {
        Image::create_empty(BUFFER_SIZE as i32, TEXTURE_HEIGHT, false, Format::R8).unwrap()
    }

    fn fetch_spectrum_analyzer(&mut self) -> Self::SpectrumAnalyzer {
        let scene_tree = self.base().get_tree().unwrap();
        let root_window = scene_tree.get_root().unwrap();
        let music_dimensions_manager = root_window.get_node_as::<Node>("MusicDimensionsManager");
        let spectrum_analyzer = music_dimensions_manager
            .get("spectrum_analyzer_instance")
            .try_to::<Self::SpectrumAnalyzer>()
            .unwrap();
        spectrum_analyzer
    }

    fn update_audio_texture(
        &mut self,
        spectrum: &Self::SpectrumAnalyzer,
        fft_data: &mut Self::FFTData,
        audio_texture: &mut Self::Image,
    ) {
        let fft_data_slice = fft_data.as_mut_slice();
        for bin_index in 0..BUFFER_SIZE {
            let bin_index_f = bin_index as f32;
            let from_hz = bin_index_f * (SAMPLE_RATE * 0.5) / MDN_BINS_F;
            let to_hz = (bin_index_f + 1.0) * (SAMPLE_RATE * 0.5) / MDN_BINS_F;
            let previous_smooth_energy = fft_data_slice[bin_index];
            let smooth_energy =
                compute_smooth_energy_for_frequency_range(spectrum, from_hz, to_hz, previous_smooth_energy);
            fft_data_slice[bin_index] = smooth_energy;
            let color = Color::from_rgba(smooth_energy, DEAD_CHANNEL, DEAD_CHANNEL, DEAD_CHANNEL);
            audio_texture.set_pixel(bin_index as i32, FFT_ROW, color);
        }
    }
}
