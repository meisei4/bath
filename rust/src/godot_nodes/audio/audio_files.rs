use crate::audio_analysis::util::{detect_bpm_aubio_ogg, detect_bpm_aubio_wav};
use godot::builtin::GString;
use godot::classes::file_access::ModeFlags;
use godot::classes::{FileAccess, Node};
use godot::obj::Base;
use godot::prelude::{godot_api, GodotClass};

#[derive(GodotClass)]
#[class(init, base=Node)]
pub struct AudioFiles {
    #[base]
    base: Base<Node>,
}

#[godot_api]
impl AudioFiles {
    #[func]
    pub fn detect_bpm_wav(&self, wav_file_path: GString) -> f32 {
        let wav_path = wav_file_path.to_string();
        let wav_file = FileAccess::open(&wav_path, ModeFlags::READ).unwrap();
        let wav_bytes = wav_file.get_buffer(wav_file.get_length() as i64).to_vec();
        detect_bpm_aubio_wav(&wav_bytes)
    }

    #[func]
    pub fn detect_bpm_ogg(&self, ogg_file_path: GString) -> f32 {
        let ogg_path = ogg_file_path.to_string();
        let ogg_file = FileAccess::open(&ogg_path, ModeFlags::READ).unwrap();
        let ogg_bytes = ogg_file.get_buffer(ogg_file.get_length() as i64).to_vec();
        detect_bpm_aubio_ogg(&ogg_bytes)
    }
}
