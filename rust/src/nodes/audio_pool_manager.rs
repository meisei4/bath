use crate::nodes::audio_bus::BUS::{INPUT, MASTER, MUSIC, SFX};
use crate::nodes::audio_bus::{AudioBus, BUS};
use godot::classes::audio_server::PlaybackType;
use godot::classes::{AudioServer, AudioStream, AudioStreamPlayer, Engine, Node, Os};
use godot::global::godot_warn;
use godot::meta::ToGodot;
use godot::obj::{Base, Gd, GodotClass, NewAlloc, WithBaseField};
use godot::register::{godot_api, GodotClass};

#[derive(GodotClass)]
#[class(init, base=Node)]
pub struct AudioPoolRust {
    base: Base<Node>,
    #[export]
    pub pool_size: i32,
    #[export]
    pub bus: BUS,
    #[export]
    pub loop_on_end: bool,
    players: Vec<Gd<AudioStreamPlayer>>,
    available: Vec<Gd<AudioStreamPlayer>>,
}

#[godot_api]
impl AudioPoolRust {
    #[func]
    pub fn ready(&mut self) {
        for _ in 0..self.pool_size {
            let mut player = AudioStreamPlayer::new_alloc();
            AudioBus::val_rust(self.bus.into());
            player.set_bus(&AudioBus::val_rust(self.bus));
            self.base_mut().add_child(&player.clone());
            self.players.push(player.clone());
            self.available.push(player);
        }
    }

    fn acquire(&mut self) -> Option<Gd<AudioStreamPlayer>> {
        if self.available.is_empty() {
            godot_warn!("Pool exhausted on bus {}", &AudioBus::val_rust(self.bus));
            None
        } else {
            self.available.pop()
        }
    }

    #[func]
    pub fn play(&mut self, resource: Gd<AudioStream>, volume_db: f32) {
        if let Some(mut player) = self.acquire() {
            let os = Os::singleton();
            if os.get_name() == "Web".into() || os.has_feature("wasm32") || os.has_feature("web") {
                player.set_playback_type(PlaybackType::STREAM);
            }
            player.set_stream(&resource);
            player.set_volume_db(volume_db);
            player.play();
        }
    }
}

#[derive(GodotClass)]
#[class(init, base=Node)]
pub struct AudioPoolManagerRust {
    base: Base<Node>,
    sfx_pool: Option<Gd<AudioPoolRust>>,
    music_pool: Option<Gd<AudioPoolRust>>,
    input_pool: Option<Gd<AudioPoolRust>>,
}

#[godot_api]
impl AudioPoolManagerRust {
    #[constant]
    const SFX_POOL_SIZE: i32 = 12;
    #[constant]
    const MUSIC_POOL_SIZE: i32 = 5;
    #[constant]
    const INPUT_POOL_SIZE: i32 = 1;

    pub fn singleton() -> Gd<AudioPoolManagerRust> {
        Engine::singleton()
            .get_singleton(&AudioPoolManagerRust::class_name().to_string_name())
            .unwrap()
            .cast::<AudioPoolManagerRust>()
    }

    #[func]
    pub fn ready(&mut self) {
        self.setup_buses(&[MASTER, SFX, MUSIC, INPUT]);
        self.set_bus_volumes();
        let mut sfx = AudioPoolRust::new_alloc();
        sfx.bind_mut().set_pool_size(Self::SFX_POOL_SIZE);
        sfx.bind_mut().set_bus(SFX.to_godot());
        sfx.bind_mut().set_loop_on_end(false);
        self.base_mut().add_child(&sfx);
        self.sfx_pool = Some(sfx);

        let mut music = AudioPoolRust::new_alloc();
        music.bind_mut().set_pool_size(Self::MUSIC_POOL_SIZE);
        music.bind_mut().set_bus(MUSIC.to_godot());
        music.bind_mut().set_loop_on_end(true);
        self.base_mut().add_child(&music);
        self.music_pool = Some(music);

        let mut input = AudioPoolRust::new_alloc();
        input.bind_mut().set_pool_size(Self::INPUT_POOL_SIZE);
        input.bind_mut().set_bus(INPUT.to_godot());
        input.bind_mut().set_loop_on_end(false);
        self.base_mut().add_child(&input);
        self.input_pool = Some(input);
    }

    fn setup_buses(&self, buses: &[BUS]) {
        let current = AudioServer::singleton().get_bus_count();
        for _ in current..buses.len() as i32 {
            AudioServer::singleton().add_bus();
        }
        for (i, &bus) in buses.iter().enumerate() {
            let name = AudioBus::val_rust(bus);
            AudioServer::singleton().set_bus_name(i as i32, name.arg());
        }
    }

    fn bus_volumes(&self) -> [(BUS, f32); 4] {
        [(MASTER, 0.0), (SFX, 0.0), (MUSIC, 0.0), (INPUT, 0.0)]
    }

    fn set_bus_volumes(&self) {
        for (bus, vol) in &self.bus_volumes() {
            let idx = AudioBus::get_bus_index_rust(*bus);
            AudioServer::singleton().set_bus_volume_db(idx, *vol);
        }
    }
    #[func]
    pub fn play_sfx(&mut self, sound: Gd<AudioStream>, volume_db: f32) {
        let pool = self.sfx_pool.as_mut().unwrap();
        pool.bind_mut().play(sound, volume_db);
    }

    #[func]
    pub fn play_music(&mut self, music: Gd<AudioStream>, volume_db: f32) {
        let pool = self.music_pool.as_mut().unwrap();
        pool.bind_mut().play(music, volume_db);
    }

    #[func]
    pub fn play_input(&mut self, input: Gd<AudioStream>, volume_db: f32) {
        let pool = self.input_pool.as_mut().unwrap();
        pool.bind_mut().play(input, volume_db);
    }
}
