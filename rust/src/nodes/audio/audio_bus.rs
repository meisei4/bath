use godot::builtin::StringName;
use godot::classes::{AudioServer, Engine, Node};
use godot::global::godot_warn;
use godot::obj::{Base, Gd, GodotClass};
use godot::register::{godot_api, Export, GodotClass, GodotConvert, Var};

#[derive(GodotConvert, Var, Copy, Clone, Debug, Export, Default)]
#[repr(u8)]
#[godot(via = u8)]
pub enum BUS {
    #[default]
    MASTER = 0,
    SFX = 1,
    MUSIC = 2,
    INPUT = 3,
}

#[derive(GodotClass)]
#[class(init, base=Node)]
pub struct AudioBus {
    base: Base<Node>,
    #[export(enum = (MASTER = 0, SFX = 1, MUSIC = 2, INPUT = 3))]
    pub bus: BUS,
}

impl AudioBus {
    //TODO: eh, keep it for reference
    pub fn singleton() -> Gd<AudioBus> {
        Engine::singleton()
            .get_singleton(&AudioBus::class_name().to_string_name())
            .unwrap()
            .cast::<AudioBus>()
    }

    pub fn get_bus_index_rust(bus: BUS) -> i32 {
        let name = Self::val_rust(bus).to_string();
        let index = AudioServer::singleton().get_bus_index(&name);
        if index == -1 {
            godot_warn!("Bus not found: {}", name);
        }
        index
    }

    pub fn val_rust(bus: BUS) -> StringName {
        match bus {
            BUS::MASTER => "Master".into(),
            BUS::SFX => "SFX".into(),
            BUS::MUSIC => "Music".into(),
            BUS::INPUT => "Input".into(),
        }
    }
}

#[godot_api]
impl AudioBus {
    #[constant]
    const MASTER: u8 = BUS::MASTER as u8;
    #[constant]
    const SFX: u8 = BUS::SFX as u8;
    #[constant]
    const MUSIC: u8 = BUS::MUSIC as u8;
    #[constant]
    const INPUT: u8 = BUS::INPUT as u8;

    #[func]
    pub fn val(&self, bus: BUS) -> StringName {
        Self::val_rust(bus)
    }

    #[func]
    pub fn get_bus_index(&self, bus: BUS) -> i32 {
        Self::get_bus_index_rust(bus)
    }
}
