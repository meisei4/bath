use crate::Context::{Godot, ShaderToy, GLSL};
use std::fs::{create_dir_all, read_to_string, write};
use std::path::PathBuf;
use std::process::Command;

// TODO: next step is reverse this process for a bidirectional godot resources to glsl
//  https://github.com/godotengine/godot/blob/master/servers/rendering/shader_language.cpp
//  https://github.com/godotengine/godot/blob/master/drivers/gles3/shader_gles3.cpp#L156
//  the goal is extreme simplicity, where the gdshader Resources should be optional, and glsl src code becomes the true priority source
//  NOTE: this might go nowhere and be a waste of time, im ok with that. but it could be very helpful
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Context {
    ShaderToy,
    Godot,
    GLSL,
}

pub struct Mapping {
    pub shadertoy: &'static str,
    pub godot: &'static str,
    pub glsl: &'static str,
}

impl Mapping {
    pub fn translate(&self, token: &str, from: Context, to: Context) -> Option<&'static str> {
        let src = match from {
            ShaderToy => self.shadertoy,
            Godot => self.godot,
            GLSL => self.glsl,
        };
        let dst = match to {
            ShaderToy => self.shadertoy,
            Godot => self.godot,
            GLSL => self.glsl,
        };
        if token == src && src != dst {
            Some(dst)
        } else {
            None
        }
    }
}

pub static MAPPINGS: &[Mapping] = &[
    Mapping {
        shadertoy: "",
        godot: "shader_type canvas_item;\
              \nrender_mode blend_disabled;",
        glsl: "#version 330",
    },
    Mapping {
        shadertoy: "void mainImage(out vec4 fragColor, in vec2 fragCoord)",
        godot: "void fragment()",
        glsl: "in vec2 fragTexCoord;\
             \nin vec4 fragColor;\
             \nout vec4 finalColor;\
             \nvoid main()",
    },
    Mapping {
        shadertoy: "fragCoord / iResolution.xy",
        godot: "UV",
        glsl: "fragTexCoord",
    },
    Mapping {
        //TODO: GLSL ALSO HAS fragColor!!! OH NO!!!
        shadertoy: "fragColor",
        godot: "COLOR",
        glsl: "finalColor",
    },
    Mapping {
        shadertoy: "fragCoord",
        godot: "FRAGCOORD",
        glsl: "gl_FragCoord.xy", //???
    },
];

fn main() {
    let src = "fragCoord / iResolution.xy";
    let mapped = translate_token(src, ShaderToy, Godot);
    println!("{} → {:?}", src, mapped);

    convert(
        "glsl/buffer_a.glsl",
        GLSL,
        Godot,
        "gdshader/buffer_a.gdshader",
    )
    .expect("conversion failed");
    convert("glsl/image.glsl", GLSL, Godot, "gdshader/image.gdshader").expect("conversion failed");
    convert(
        "glsl/buffer_a.glsl",
        GLSL,
        ShaderToy,
        "shadertoy/buffer_a.shadertoy",
    )
    .expect("conversion failed");
    convert(
        "glsl/image.glsl",
        GLSL,
        ShaderToy,
        "shadertoy/image.shadertoy",
    )
    .expect("conversion failed");
    convert(
        "shadertoy/buffer_a.shadertoy",
        ShaderToy,
        GLSL,
        "glsl/buffer_a.glsl",
    )
    .expect("conversion failed");
    convert(
        "shadertoy/image.shadertoy",
        ShaderToy,
        GLSL,
        "glsl/image.glsl",
    )
    .expect("conversion failed");
    if let Err(e) = compare_dirs("resources", "test_output") {
        eprintln!("Failed to diff directories: {}", e);
    }
}

fn compare_dirs(dir1: &str, dir2: &str) -> std::io::Result<()> {
    let output = Command::new("diff").args(&["-ru", dir1, dir2]).output()?;

    if !output.stdout.is_empty() {
        print!("{}", String::from_utf8_lossy(&output.stdout));
    }
    if !output.stderr.is_empty() {
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
    }
    Ok(())
}

pub fn translate_token(token: &str, from: Context, to: Context) -> Option<&'static str> {
    for m in MAPPINGS {
        let src = match from {
            ShaderToy => m.shadertoy,
            Godot => m.godot,
            GLSL => m.glsl,
        };
        if src == token {
            return Some(match to {
                ShaderToy => m.shadertoy,
                Godot => m.godot,
                GLSL => m.glsl,
            });
        }
    }
    None
}

pub fn convert_shader(input: &str, from: Context, to: Context) -> String {
    let mut out = input.to_string();
    let header_mapping = &MAPPINGS[0];
    let src_header = match from {
        ShaderToy => header_mapping.shadertoy,
        Godot => header_mapping.godot,
        GLSL => header_mapping.glsl,
    };
    let dst_header = match to {
        ShaderToy => header_mapping.shadertoy,
        Godot => header_mapping.godot,
        GLSL => header_mapping.glsl,
    };
    if !src_header.is_empty() {
        out = out.replace(src_header, "");
    }
    out = out.trim_start_matches('\n').to_string();
    if !dst_header.is_empty() {
        out = format!("{}\n{}", dst_header, out);
    }
    for mapping in &MAPPINGS[1..] {
        let src = match from {
            ShaderToy => mapping.shadertoy,
            Godot => mapping.godot,
            GLSL => mapping.glsl,
        };
        let dst = match to {
            ShaderToy => mapping.shadertoy,
            Godot => mapping.godot,
            GLSL => mapping.glsl,
        };
        if !src.is_empty() && src != dst {
            out = out.replace(src, dst);
        }
    }
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out
}

pub fn convert(
    input_rel: &str,
    from: Context,
    to: Context,
    output_rel: &str,
) -> std::io::Result<()> {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let in_path = base.join("resources").join(input_rel);
    let out_path = base.join("test_output").join(output_rel);
    if let Some(dir) = out_path.parent() {
        create_dir_all(dir)?;
    }
    let src = read_to_string(in_path)?;
    let dst = convert_shader(&src, from, to);
    write(out_path, dst)?;
    Ok(())
}
