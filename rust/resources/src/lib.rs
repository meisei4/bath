pub mod glsl {
    pub const DREKKER_PATH: &str = "resources/glsl/color/drekker_effect.glsl";
    pub const BUFFER_A_PATH: &str = "resources/glsl/buffer_a.glsl";
    pub const IMAGE_PATH: &str = "resources/glsl/image.glsl";
    pub const ICE_SHEETS_PATH: &str = "resources/glsl/ice_sheets/ice_sheets.glsl";
    pub const ICE_FRAG_PATH: &str = "resources/glsl/ice_sheets/ice_frag.glsl";
    pub const ICE_FRAG_2_PATH: &str = "resources/glsl/ice_sheets/ice_frag_2.glsl";
    pub const ICE_VERT_PATH: &str = "resources/glsl/ice_sheets/ice_vertex.glsl";


    pub const BUFFER_A_CONTENTS: &str = include_str!("../glsl/buffer_a.glsl");
    pub const IMAGE_CONTENTS: &str = include_str!("../glsl/image.glsl");
}

pub mod gdshader {
    pub const BUFFER_A: &str = include_str!("../gdshader/buffer_a.gdshader");
    pub const IMAGE: &str = include_str!("../gdshader/image.gdshader");
}

pub mod shadertoy {
    pub const BUFFER_A: &str = include_str!("../shadertoy/buffer_a.shadertoy.glsl");
    pub const IMAGE: &str = include_str!("../shadertoy/image.shadertoy.glsl");
}

pub mod audio_godot {
    pub const SOUND_FONT_FILE_PATH: &str = "../godot/Resources/audio/dsdnmoy.sf2";
    pub const MIDI_FILE_PATH: &str = "../godot/Resources/audio/fingerbib.mid";
    pub const SOUND_FONT_FILE_BYTES: &[u8] = include_bytes!("../../../godot/Resources/audio/dsdnmoy.sf2");
    pub const MIDI_FILE_BYTES: &[u8] = include_bytes!("../../../godot/Resources/audio/fingerbib.mid");
}

pub mod textures {
    pub const ICEBERGS_JPG: &str = "../godot/Resources/textures/icebergs.jpg";
}
