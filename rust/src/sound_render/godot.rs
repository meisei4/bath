use crate::nodes::audio_bus::AudioBus;
use crate::nodes::audio_bus::BUS::MUSIC;
use crate::sound_render::sound_renderer::{
    FFTTexture, WaveformTexture, BUFFER_SIZE, DEAD_CHANNEL, FFT_ROW, HZ_STEP, INVERSE_DECIBEL_RANGE,
    MDN_MIN_AUDIO_DECIBEL, TEXTURE_HEIGHT,
};
use crate::sound_render::util::compute_smooth_energy;
use godot::builtin::PackedFloat32Array;
use godot::classes::audio_effect_spectrum_analyzer_instance::MagnitudeMode;
use godot::classes::image::Format;
use godot::classes::{AudioEffectCapture, AudioEffectSpectrumAnalyzerInstance, AudioServer, Image, Node};
use godot::global::linear_to_db;
use godot::obj::{Base, Gd, NewGd, WithBaseField};
use godot::prelude::{Color, GodotClass};

#[derive(GodotClass)]
#[class(init, base=Node)]
pub struct GodotFFTTexture {
    base: Base<Node>,
}

impl FFTTexture for GodotFFTTexture {
    type Image = Gd<Image>;
    type FFTData = PackedFloat32Array;
    type AudioEffect = Gd<AudioEffectSpectrumAnalyzerInstance>;

    fn resize_buffer(&mut self, fft_data: &mut PackedFloat32Array) {
        fft_data.resize(BUFFER_SIZE);
    }

    fn init_audio_texture(&mut self) -> Self::Image {
        Image::create_empty(BUFFER_SIZE as i32, TEXTURE_HEIGHT, false, Format::RGBA8).unwrap()
    }

    fn fetch_spectrum_analyzer(&mut self) -> Self::AudioEffect {
        let scene_tree = self.base().get_tree().unwrap();
        let root_window = scene_tree.get_root().unwrap();
        let music_dimensions_manager = root_window.get_node_as::<Node>("MusicDimensionsManager");
        let spectrum_analyzer = music_dimensions_manager
            .get("spectrum_analyzer_instance")
            .try_to::<Self::AudioEffect>()
            .unwrap();
        spectrum_analyzer
        //MusicDimensionsManagerRust::singleton().bind_mut().spectrum_instance()
        // Engine::singleton()
        //     .get_singleton(&StringName::from("MusicDimensionsManagerRust"))
        //     .unwrap()
        //     .cast::<MusicDimensionsManagerRust>()
        //     .bind_mut()
        //     .spectrum_instance()
    }

    fn update_audio_texture(&mut self, fft_data: &mut Self::FFTData, audio_texture: &mut Self::Image) {
        let fft_data_slice = fft_data.as_mut_slice();
        for bin_index in 0..BUFFER_SIZE {
            let bin_index_f = bin_index as f32;
            let from_hz = bin_index_f * HZ_STEP;
            let to_hz = (bin_index_f + 1.0) * HZ_STEP;
            // http://github.com/godotengine/godot/blob/master/servers/audio/effects/audio_effect_spectrum_analyzer.cpp
            let spectrum = self.fetch_spectrum_analyzer();
            let stereo_magnitude = spectrum
                .get_magnitude_for_frequency_range_ex(from_hz, to_hz)
                .mode(MagnitudeMode::AVERAGE)
                .done();

            let linear_magnitude = (stereo_magnitude.x + stereo_magnitude.y) / 2_f32;
            let db = linear_to_db(linear_magnitude as f64) as f32;
            let normalized = ((db - MDN_MIN_AUDIO_DECIBEL) * INVERSE_DECIBEL_RANGE).clamp(0_f32, 1_f32);
            let previous_smooth_energy = fft_data_slice[bin_index];
            let smooth_energy = compute_smooth_energy(previous_smooth_energy, normalized);
            fft_data_slice[bin_index] = smooth_energy;
            let color = Color::from_rgba(smooth_energy, DEAD_CHANNEL, DEAD_CHANNEL, DEAD_CHANNEL);
            audio_texture.set_pixel(bin_index as i32, FFT_ROW, color);
        }
    }
}

const WAVEFORM_ROW: i32 = 0;

#[derive(GodotClass)]
#[class(init, base = Node)]
pub struct GodotWaveformTexture {
    base: Base<Node>,
}

impl WaveformTexture for GodotWaveformTexture {
    type Image = Gd<Image>;
    type WaveformData = PackedFloat32Array;
    type AudioEffect = Gd<AudioEffectCapture>;
    fn resize_buffer(&mut self, waveform_data: &mut Self::WaveformData) {
        waveform_data.resize(BUFFER_SIZE);
    }

    fn init_audio_texture(&mut self) -> Self::Image {
        Image::create_empty(BUFFER_SIZE as i32, TEXTURE_HEIGHT, false, Format::R8).unwrap()
    }

    fn fetch_waveform_capture(&mut self) -> Self::AudioEffect {
        let waveform_audio_effect_capture = AudioEffectCapture::new_gd();
        let mut audio_server: Gd<AudioServer> = AudioServer::singleton();
        //let bus_index = AudioBus::singleton().bind_mut().get_bus_index(MUSIC);
        //audio_server.add_bus_effect(bus_index, &waveform_audio_effect_capture);
        audio_server.add_bus_effect(AudioBus::get_bus_index_rust(MUSIC), &waveform_audio_effect_capture);
        waveform_audio_effect_capture
    }

    fn update_audio_texture(
        &mut self,
        waveform_capture: &mut Self::AudioEffect,
        waveform_data: &mut Self::WaveformData,
        audio_texture: &mut Self::Image,
    ) {
        let waveform_data_slice = waveform_data.as_mut_slice();
        let waveform_audio_effect_capture = waveform_capture;
        if waveform_audio_effect_capture.can_get_buffer(BUFFER_SIZE as i32) {
            let captured_frames_from_current_waveform_buffer =
                waveform_audio_effect_capture.get_buffer(BUFFER_SIZE as i32);
            let waveform_buffer_slice = captured_frames_from_current_waveform_buffer.as_slice();
            let frame_count = captured_frames_from_current_waveform_buffer.len();
            let frames_per_pixel = frame_count / BUFFER_SIZE;
            for x in 0..BUFFER_SIZE {
                let start_frame_index = x * frames_per_pixel;
                let mut end_frame_index = (x + 1) * frames_per_pixel;
                if end_frame_index > frame_count {
                    end_frame_index = frame_count;
                }

                let mut accumulated_amplitudes: f32 = 0.0;
                let mut number_of_amplitude_frames_to_average: usize = 0;
                for i in start_frame_index..end_frame_index {
                    accumulated_amplitudes += waveform_buffer_slice[i].x;
                    number_of_amplitude_frames_to_average += 1;
                }

                let mut average_amplitude: f32 = if number_of_amplitude_frames_to_average > 0 {
                    accumulated_amplitudes / number_of_amplitude_frames_to_average as f32
                } else {
                    0.0
                };

                average_amplitude = average_amplitude / 2.0 + 0.5;
                waveform_data_slice[x] = average_amplitude;

                let audio_texture_value: Color =
                    Color::from_rgba(average_amplitude, DEAD_CHANNEL, DEAD_CHANNEL, DEAD_CHANNEL);
                audio_texture.set_pixel(x as i32, WAVEFORM_ROW, audio_texture_value);
            }
        } else {
            for x in 0..BUFFER_SIZE {
                let audio_texture_value: Color =
                    Color::from_rgba(waveform_data_slice[x], DEAD_CHANNEL, DEAD_CHANNEL, DEAD_CHANNEL);
                audio_texture.set_pixel(x as i32, WAVEFORM_ROW, audio_texture_value);
            }
        }
    }
}
