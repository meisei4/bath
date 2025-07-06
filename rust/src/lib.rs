use crate::nodes::audio_files::AudioFiles;
use crate::nodes::collision::Collision;
use crate::sound_render::audio_bus::AudioBus;
use godot::classes::Engine;
use godot::init::{gdextension, ExtensionLibrary, InitLevel};
use godot::obj::NewAlloc;

pub mod audio_analysis;
pub mod collision_mask;
pub mod midi;
pub mod nodes;
pub mod render;
pub mod resource_paths;
pub mod sound_render;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {
    fn on_level_init(level: InitLevel) {
        if level == InitLevel::Scene {
            Engine::singleton().register_singleton("AudioBus", &AudioBus::new_alloc());
            Engine::singleton().register_singleton("Collision", &Collision::new_alloc());
            Engine::singleton().register_singleton("AudioFiles", &AudioFiles::new_alloc());
        }
    }

    fn on_level_deinit(level: InitLevel) {
        if level == InitLevel::Scene {
            let mut engine = Engine::singleton();
            let name = "AudioBus";
            if let Some(singleton) = engine.get_singleton(name) {
                engine.unregister_singleton(name);
                singleton.free();
            }

            let name = "Collision";
            if let Some(singleton) = engine.get_singleton(name) {
                engine.unregister_singleton(name);
                singleton.free();
            }

            let name = "AudioFiles";
            if let Some(singleton) = engine.get_singleton(name) {
                engine.unregister_singleton(name);
                singleton.free();
            }
        }
    }
}
