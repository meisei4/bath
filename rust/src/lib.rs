extern crate core;

pub mod audio_analysis;
pub mod collision_mask;
pub mod fixed_func;
pub mod fu4seoi3;
pub mod midi;
pub mod render;
pub mod sound_render;

#[cfg(feature = "godot")]
pub mod godot_nodes;

#[cfg(feature = "godot")]
mod godot_extension {
    use crate::godot_nodes::audio::audio_bus::AudioBus;
    use crate::godot_nodes::audio::audio_files::AudioFiles;
    use crate::godot_nodes::collision::Collision;
    use godot::classes::Engine;
    use godot::init::{gdextension, ExtensionLibrary, InitLevel};
    use godot::obj::NewAlloc;

    struct MyExtension;

    #[gdextension]
    unsafe impl ExtensionLibrary for MyExtension {
        fn on_level_init(level: InitLevel) {
            if level == InitLevel::Scene {
                Engine::singleton().register_singleton("AudioBus", &AudioBus::new_alloc());
                Engine::singleton().register_singleton("AudioFiles", &AudioFiles::new_alloc());
                Engine::singleton().register_singleton("Collision", &Collision::new_alloc());
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
}
