use crate::nodes::audio_bus::{AudioBus, BUS};
use godot::classes::audio_effect_spectrum_analyzer::FftSize;
use godot::classes::{
    AudioEffectSpectrumAnalyzer, AudioEffectSpectrumAnalyzerInstance, AudioServer, AudioStream, Engine, Node,
};
use godot::obj::{Base, Gd, GodotClass, NewGd};
use godot::register::{godot_api, GodotClass};

#[derive(GodotClass)]
#[class(init, base=Node)]
pub struct MusicDimensionsManagerRust {
    #[base]
    base: Base<Node>,
    spectrum_analyzer_instance: Option<Gd<AudioEffectSpectrumAnalyzerInstance>>,
    _audio_stream: Gd<AudioStream>,
    #[export]
    song_time: f32,
}

#[godot_api]
impl MusicDimensionsManagerRust {
    pub fn singleton() -> Gd<MusicDimensionsManagerRust> {
        Engine::singleton()
            .get_singleton(&MusicDimensionsManagerRust::class_name().to_string_name())
            .unwrap()
            .cast::<MusicDimensionsManagerRust>()
    }

    pub fn spectrum_instance(&self) -> Gd<AudioEffectSpectrumAnalyzerInstance> {
        self.spectrum_analyzer_instance
            .clone()
            .expect("Spectrum analyzer not yet initialized")
    }

    #[func]
    pub fn ready(&mut self) {
        let bus_index = AudioBus::singleton().bind().get_bus_index(BUS::MUSIC);
        let mut analyzer: Gd<AudioEffectSpectrumAnalyzer> = AudioEffectSpectrumAnalyzer::new_gd();
        analyzer.set_fft_size(FftSize::SIZE_1024);
        AudioServer::singleton().add_bus_effect(bus_index, &analyzer);
        let effect_count = AudioServer::singleton().get_bus_effect_count(bus_index);
        let audio_effect_instance = AudioServer::singleton()
            .get_bus_effect_instance(bus_index, effect_count - 1)
            .unwrap()
            .cast::<AudioEffectSpectrumAnalyzerInstance>();
        self.spectrum_analyzer_instance = Some(audio_effect_instance);

        //AudioPoolManager::singleton().bind_mut().play_music(self.audio_stream.clone(), 0.0);
    }

    #[func]
    pub fn process(&mut self, delta: f64) {
        self.song_time += delta as f32;
    }
}
