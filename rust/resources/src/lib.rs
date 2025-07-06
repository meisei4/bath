pub mod glsl {
    pub const RAYLIB_DEFAULT_VERT_PATH: &str = "resources/shaders/glsl/raylib_default_vertex.glsl";
    pub const DEBUG_VERT_PATH: &str = "resources/shaders/glsl/debug_vertex.glsl";
    pub const DEBUG_FRAG_PATH: &str = "resources/shaders/glsl/debug_fragment.glsl";

    pub const DREKKER_PATH: &str = "resources/shaders/glsl/color/drekker_effect.glsl";
    pub const BUFFER_A_PATH: &str = "resources/shaders/glsl/buffer_a.glsl";
    pub const IMAGE_PATH: &str = "resources/shaders/glsl/image.glsl";

    pub const ICESHEETS_FRAG_DRAFT_PATH: &str = "resources/shaders/glsl/ice_sheets/icesheets_fragment_drafting.glsl";
    pub const ICESHEETS_FRAG_PATH: &str = "resources/shaders/glsl/ice_sheets/icesheets_fragment.glsl";
    pub const ICESHEETS_VERT_DRAFT_PATH: &str = "resources/shaders/glsl/ice_sheets/icesheets_vertex_drafting.glsl";
    pub const ICESHEETS_VERT_PATH: &str = "resources/shaders/glsl/ice_sheets/icesheets_vertex.glsl";

    pub const FFT_FRAG_PATH: &str = "resources/shaders/glsl/audio/fft.glsl";

}

pub mod audio {
    pub const WAV_TEST: &'static str = "resources/audio/cached_wav.wav";
    pub const SHADERTOY_WAV: &'static str = "resources/audio/shadertoy.wav";
    pub const SHADERTOY_WHAT_WAV: &'static str = "resources/audio/shadertoy_what.wav";
}

pub mod textures {
    pub const ICEBERGS_JPG: &str = "../godot/Resources/textures/icebergs.jpg";
}

pub mod audio_godot {
    pub const SOUND_FONT_FILE_PATH: &str = "../godot/Resources/audio/dsdnmoy.sf2";
    pub const MIDI_FILE_PATH: &str = "../godot/Resources/audio/fingerbib.mid";
    // pub const MIDI_FILE_BYTES: &[u8] = include_bytes!("fingerbib.mid");
}

pub mod gdshader {
    pub const BUFFER_A: &str = include_str!("../shaders/gdshader/buffer_a.gdshader");
    pub const IMAGE: &str = include_str!("../shaders/gdshader/image.gdshader");
}

pub mod shadertoy {
    pub const BUFFER_A: &str = include_str!("../shaders/shadertoy/buffer_a.shadertoy.glsl");
    pub const IMAGE: &str = include_str!("../shaders/shadertoy/image.shadertoy.glsl");
}