use crate::godot_nodes::audio::audio_files::AudioFiles;
use crate::godot_nodes::collision::Collision;
use godot::classes::Engine;
use godot::init::{gdextension, ExtensionLibrary, InitLevel};
use godot::obj::NewAlloc;
use godot_nodes::audio::audio_bus::AudioBus;

pub mod audio_analysis;
pub mod collision_mask;
pub mod godot_nodes;
pub mod midi;
pub mod render;
pub mod sound_render;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {
    fn on_level_init(level: InitLevel) {
        if level == InitLevel::Scene {
            Engine::singleton().register_singleton("AudioBus", &AudioBus::new_alloc());
            Engine::singleton().register_singleton("AudioFiles", &AudioFiles::new_alloc());
            Engine::singleton().register_singleton("Collision", &Collision::new_alloc());
            //TODO: this all just sucks, i hate this so much, its bad design on my part idk why im doing it
            // this shouldnt be some instance singleton you should get the fucking spectrum instance everytime you make a ffttexture
            // dont try to make a singleton of it for no reason absolutely stupid wasted like 4 hours abosltu
            // Engine::singleton()
            //     .register_singleton("MusicDimensionsManagerRust", &MusicDimensionsManagerRust::new_alloc());
            // Engine::singleton().register_singleton("AudioPoolManagerRust", &AudioPoolManagerRust::new_alloc());
        }
    }

    fn on_level_deinit(level: InitLevel) {
        if level == InitLevel::Scene {
            let mut engine = Engine::singleton();

            // let name = "MusicDimensionsManagerRust";
            // if let Some(singleton) = engine.get_singleton(name) {
            //     engine.unregister_singleton(name);
            //     singleton.free();
            // }
            //
            // let name = "AudioPoolManagerRust";
            // if let Some(singleton) = engine.get_singleton(name) {
            //     engine.unregister_singleton(name);
            //     singleton.free();
            // }

            let name = "AudioBus";
            if let Some(singleton) = engine.get_singleton(name) {
                engine.unregister_singleton(name);
                singleton.free();
            }

            let name = "AudioFiles";
            if let Some(singleton) = engine.get_singleton(name) {
                engine.unregister_singleton(name);
                singleton.free();
            }

            let name = "Collision";
            if let Some(singleton) = engine.get_singleton(name) {
                engine.unregister_singleton(name);
                singleton.free();
            }
        }
    }
}
