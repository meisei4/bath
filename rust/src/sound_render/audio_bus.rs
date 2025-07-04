use crate::sound_render::audio_bus::BUS::{INPUT, MASTER, MUSIC, SFX};
use godot::builtin::StringName;
use godot::classes::{AudioServer, Node};
use godot::obj::Base;
use godot::register::{godot_api, GodotClass, GodotConvert, Var};

#[derive(GodotConvert, Var, Copy, Clone, Debug)]
#[godot(via = i64)]
pub enum BUS {
    MASTER = 0,
    SFX,
    MUSIC,
    INPUT,
}

impl BUS {
    pub fn as_name(self) -> StringName {
        match self {
            MASTER => StringName::from("Master"),
            SFX => StringName::from("SFX"),
            MUSIC => StringName::from("Music"),
            INPUT => StringName::from("Input"),
        }
    }

    pub fn get_bus_index(self) -> i32 {
        AudioServer::singleton().get_bus_index(&self.as_name().to_string())
    }
}

#[derive(GodotClass)]
#[class(init, base=Node)]
pub struct AudioBusRust {
    base: Base<Node>,
}

// TODO: this is psycho https://godot-rust.github.io/book/register/properties.html?highlight=enum#enums
//  https://godot-rust.github.io/book/register/constants.html
//  https://godot-rust.github.io/book/recipes/engine-singleton.html
#[godot_api]
impl AudioBusRust {
    #[func]
    pub fn get_bus_index(&self, bus: BUS) -> i32 {
        bus.get_bus_index()
    }

    #[func]
    pub fn val(&self, bus: BUS) -> StringName {
        bus.as_name()
    }

    #[constant]
    const MASTER: i32 = MASTER as i32;
    #[constant]
    const SFX: i32 = SFX as i32;
    #[constant]
    const MUSIC: i32 = MUSIC as i32;
    #[constant]
    const INPUT: i32 = INPUT as i32;
}
