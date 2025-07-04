pub mod glsl {
    pub const RAYLIB_DEFAULT_VERT_PATH: &str = "resources/glsl/raylib_default_vertex.glsl";
    pub const DEBUG_VERT_PATH: &str = "resources/glsl/debug_vertex.glsl";
    pub const DEBUG_FRAG_PATH: &str = "resources/glsl/debug_fragment.glsl";

    pub const DREKKER_PATH: &str = "resources/glsl/color/drekker_effect.glsl";
    pub const BUFFER_A_PATH: &str = "resources/glsl/buffer_a.glsl";
    pub const IMAGE_PATH: &str = "resources/glsl/image.glsl";
    pub const ICESHEETS_FRAG_DRAFT_PATH: &str = "resources/glsl/ice_sheets/icesheets_fragment_drafting.glsl";
    pub const ICESHEETS_FRAG_PATH: &str = "resources/glsl/ice_sheets/icesheets_fragment.glsl";
    pub const ICESHEETS_VERT_DRAFT_PATH: &str = "resources/glsl/ice_sheets/icesheets_vertex_drafting.glsl";
    pub const ICESHEETS_VERT_PATH: &str = "resources/glsl/ice_sheets/icesheets_vertex.glsl";

    pub const FFT_FRAG_PATH: &str = "resources/glsl/audio/fft.glsl";
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
    pub const SHADERTOY_MUSIC_EXPERIMENT_OGG: &'static str = "../../../godot/Resources/audio/shadertoy_music_experiment_min_bitrate.ogg";
    pub const WAV_TEST: &'static str = "resources/glsl/audio/cached_wav.wav";

}

pub mod textures {
    pub const ICEBERGS_JPG: &str = "../godot/Resources/textures/icebergs.jpg";
}
