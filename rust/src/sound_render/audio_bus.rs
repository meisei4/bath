use godot::builtin::StringName;
use godot::classes::AudioServer;
use godot::global::godot_error;

#[derive(Copy, Clone, Debug)]
pub enum AudioBus {
    Master,
    Sfx,
    Music,
    Input,
}

impl AudioBus {
    pub fn name(self) -> StringName {
        match self {
            AudioBus::Master => "Master".into(),
            AudioBus::Sfx => "SFX".into(),
            AudioBus::Music => "Music".into(),
            AudioBus::Input => "Input".into(),
        }
    }

    pub fn index(self) -> i32 {
        let bus_name = self.name();
        let index = AudioServer::singleton().get_bus_index(&bus_name);
        if index == -1 {
            godot_error!("Audio bus not found: {}", bus_name.to_string());
        }
        index
    }
}
