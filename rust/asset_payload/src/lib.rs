#![allow(non_snake_case)]

#[macro_export]
macro_rules! define_payloads {
    (binary : { $( $BIN:ident => $bin_path:expr ),* $(,)? },
     string : { $( $STR:ident => $str_path:expr ),* $(,)? }) => {
        #[cfg(feature = "nasa-embed")]
        pub mod payloads {
            $( pub const $BIN: &[u8] = include_bytes!(concat!("../../../assets/", $bin_path)); )*
            $( pub const $STR: &str  = include_str!(concat!("../../../assets/", $str_path)); )*
        }

        #[cfg(not(feature = "nasa-embed"))]
        pub mod payloads {
            use std::{fs, str};
            fn leak(path: &str) -> &'static [u8] {
                Box::leak(fs::read(path).unwrap_or_else(|e| panic!("read payload {path}: {e}")).into_boxed_slice())
            }

            $(
                #[allow(non_upper_case_globals)]
                mod $BIN {
                    use super::leak;
                    use std::sync::OnceLock;
                    static CELL: OnceLock<&'static [u8]> = OnceLock::new();
                    pub fn get() -> &'static [u8] {
                        CELL.get_or_init(|| leak(concat!("../assets/", $bin_path)))
                    }
                }
                pub use $BIN::get as $BIN;
            )*

            $(
                #[allow(non_upper_case_globals)]
                mod $STR {
                    use super::leak;
                    use std::sync::OnceLock;
                    static CELL: OnceLock<&'static str> = OnceLock::new();
                    pub fn get() -> &'static str {
                        CELL.get_or_init(|| {
                            let bytes = leak(concat!("../assets/", $str_path));
                            str::from_utf8(bytes).expect("utf-8 payload")
                        })
                    }
                }
                pub use $STR::get as $STR;
            )*
        }

        pub struct ResourcePaths;
        pub struct LocalCachePaths;

        impl ResourcePaths {
            $( pub const $BIN: &str = concat!("res://assets/", $bin_path); )*
            $( pub const $STR: &str = concat!("res://assets/", $str_path); )*
        }

        impl LocalCachePaths {
            $( pub const $BIN: &str = concat!("../assets/", $bin_path); )*
            $( pub const $STR: &str = concat!("../assets/", $str_path); )*
        }

        pub fn lookup_shader_source(path: &str) -> Option<&'static str> {
            match path {
                $( $str_path => Some($crate::payloads::$STR()), )*
                _ => None,
            }
        }

        pub fn expand_includes(root_src: &str) -> &'static str {
            fn recurse(src: &str, out: &mut Vec<u8>) {
                for line in src.lines() {
                    if let Some(rest) = line.trim_start().strip_prefix("#include") {
                        if let Some(name) = rest.trim().strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
                            let include_src = lookup_shader_source(name).unwrap_or_else(|| panic!("Unresolved include {:?}", name));
                            recurse(include_src, out);
                            continue;
                        }
                    }
                    out.extend_from_slice(line.as_bytes());
                    out.push(b'\n');
                }
            }
            let mut buf: Vec<u8> = Vec::new();
            recurse(root_src, &mut buf);
            //NOTE: this is an intentional leak for developer hot reloading only! (I think....idk if its a leak for includes
            // regardless its also just an explciit example of Box leaks in rust for me to reference
            let leaked: &'static [u8] = Box::leak(buf.into_boxed_slice());
            unsafe { str::from_utf8_unchecked(leaked) }
        }
    }
}

define_payloads! {
    binary: {
        // https://shadertoyunofficial.wordpress.com/2019/07/23/shadertoy-media-files
        // https://www.shadertoy.com/media/a/29de534ed5e4a6a224d2dfffab240f2e19a9d95f5e39de8898e850efdb2a99de.mp3
        SHADERTOY_WAV_PATH            => "audio/shadertoy.wav",
        SHADERTOY_WHAT_PATH           => "audio/shadertoy_what.wav",
        // ffmpeg -i shadertoy_music_experiment.wav -c:a libvorbis -qscale:a 0.1 -ar 12000 -ac 1 -compression_level 10 shadertoy_music_experiment_min_bitrate.ogg
        SHADERTOY_EXPERIMENT_OGG_PATH => "audio/shadertoy_music_experiment_min_bitrate.ogg",
        SOUND_FONT_FILE_PATH          => "audio/dsdnmoy.sf2",
        MIDI_FILE_PATH                => "audio/fingerbib.mid",
        CACHED_WAV_PATH               => "audio/cache/cached_wav.wav",
        CACHED_RHYTHM_DATA            => "audio/cache/RhythmData.tres",
        BAYER_PNG_PATH                => "textures/bayer.png",
        GRAY_NOISE_SMALL_PNG_PATH     => "textures/gray_noise_small.png",
        ICEBERGS_JPG_PATH             => "textures/icebergs.jpg",
        MOON_WATER_PNG_PATH           => "textures/moon_water.png",
        PEBBLES_PNG_PATH              => "textures/pebbles.png",
        ROCKS_JPG_PATH                => "textures/rocks.jpg"
    },
    string: {
        RAYLIB_DEFAULT_VERT_PATH      => "shaders/glsl/raylib_default_vertex.glsl",
        DEBUG_VERT_PATH               => "shaders/glsl/debug_vertex.glsl",
        DEBUG_FRAG_PATH               => "shaders/glsl/debug_fragment.glsl",
        DREKKER_PATH                  => "shaders/glsl/color/drekker_effect.glsl",
        SUPERSAMPLING                 => "shaders/glsl/color/supersampling.glsl",
        BUFFER_A_PATH                 => "shaders/glsl/buffer_a.glsl",
        IMAGE_PATH                    => "shaders/glsl/image.glsl",
        ICESHEETS_FRAG_DRAFT_PATH     => "shaders/glsl/ice_sheets/icesheets_fragment_drafting.glsl",
        ICESHEETS_FRAG_PATH           => "shaders/glsl/ice_sheets/icesheets_fragment.glsl",
        ICESHEETS_VERT_DRAFT_PATH     => "shaders/glsl/ice_sheets/icesheets_vertex_drafting.glsl",
        ICESHEETS_VERT_PATH           => "shaders/glsl/ice_sheets/icesheets_vertex.glsl",
        FFT_FRAG_PATH                 => "shaders/glsl/audio/fft.glsl",

        BUFFER_A_GDSHADER                          => "shaders/gdshader/buffer_a.gdshader",
        MAIN_GDSHADER                              => "shaders/gdshader/main.gdshader",
        DREKKER_GDSHADER                           => "shaders/gdshader/color/drekker_effect.gdshader",
        SUPERSAMPLING_GDSHADERINC                  => "shaders/gdshader/color/supersampling.gdshaderinc",
        FFT_GDSHADER                               => "shaders/gdshader/audio/fft.gdshader",
        GHOST_GDSHADER                             => "shaders/gdshader/audio/ghost.gdshader",
        IOI_GDSHADER                               => "shaders/gdshader/audio/ioi.gdshader",
        MUSIC_BALL_GDSHADER                        => "shaders/gdshader/audio/music_ball.gdshader",
        WAVEFORM_GDSHADER                          => "shaders/gdshader/audio/waveform.gdshader",
        BUFFER_A_SOUND_ENVELOPE_GDSHADER           => "shaders/gdshader/audio/sound_envelope_wip/buffer_a_sound_envelope.gdshader",
        IMAGE_SOUND_ENVELOPE_GDSHADER              => "shaders/gdshader/audio/sound_envelope_wip/image_sound_envelope.gdshader",
        OPTIMIZED_ENVELOPE_BUFFER_A_GDSHADER       => "shaders/gdshader/audio/sound_envelope_wip/optimized_envelope_buffer_a.gdshader",
        OPTIMIZED_ENVELOPE_BUFFER_B_GDSHADER       => "shaders/gdshader/audio/sound_envelope_wip/optimized_envelope_buffer_b.gdshader",
        UTILS_SOUND_ENVELOPE_GDSHADERINC           => "shaders/gdshader/audio/sound_envelope_wip/utils.gdshaderinc",
        ALL_SPRITE_MASK_GDSHADER                   => "shaders/gdshader/masks/all_sprite_mask.gdshader",
        COLLISION_MASK_FRAGMENT_GDSHADER           => "shaders/gdshader/masks/collision_mask_fragment.gdshader",
        DITHER_ZONE_GDSHADER                       => "shaders/gdshader/masks/dither_zone.gdshader",
        FREE_ALPHA_CHANNEL_GDSHADER                => "shaders/gdshader/masks/free_alpha_channel.gdshader",
        PERSPECTIVE_TILT_MASK_GDSHADER             => "shaders/gdshader/masks/perspective_tilt_mask.gdshader",
        SCANLINE_GDSHADER                          => "shaders/gdshader/masks/scanline.gdshader",
        UMBRAL_ZONE_GDSHADER                       => "shaders/gdshader/masks/umbral_zone.gdshader",
        DIVE_GDSHADER                              => "shaders/gdshader/mechanics/dive.gdshader",
        JUMP_TRIG_GDSHADER                         => "shaders/gdshader/mechanics/jump_trig.gdshader",
        PERSPECTIVE_TILT_GDSHADERINC               => "shaders/gdshader/mechanics/perspective_tilt.gdshaderinc",
        CONSTANTS_GDSHADERINC                      => "shaders/gdshader/water/constants.gdshaderinc",
        FINITE_APPROX_RIPPLE_GDSHADER              => "shaders/gdshader/water/finite_approx_ripple.gdshader",
        WATER_GDSHADER                             => "shaders/gdshader/water/water.gdshader",
        WATER_PROJECTED_GDSHADER                   => "shaders/gdshader/water/water_projected.gdshader",
        ICE_SHEETS_SHADER_FULL_GDSHADER      => "shaders/gdshader/ice_sheets/icesheet_full.gdshader",
        ICE_SHEETS_GDSHADER                  => "shaders/gdshader/ice_sheets/ice_sheets.gdshader",
        NOISE_GDSHADERINC                    => "shaders/gdshader/ice_sheets/noise.gdshaderinc",
        PROJECTIONS_GDSHADERINC              => "shaders/gdshader/ice_sheets/projections.gdshaderinc",
        COLOR_GDSHADERINC                    => "shaders/gdshader/ice_sheets/color.gdshaderinc",
        SNOW_PARTICLE_GDSHADER               => "shaders/gdshader/particles/snow_particle_shader.gdshader"}
}
