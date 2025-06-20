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
    fn get(&self, ctx: Context) -> &'static str {
        match ctx {
            ShaderToy => self.shadertoy,
            Godot => self.godot,
            GLSL => self.glsl,
        }
    }
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
        godot: "FRAGCOORD.xy / iResolution.xy",
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
    Mapping {
        shadertoy: "",
        godot: "uniform vec2 iResolution;",
        glsl: "uniform vec2 iResolution;",
    },
    Mapping {
        shadertoy: "",
        godot: "uniform sampler2D iChannel0 : hint_screen_texture;",
        glsl: "uniform sampler2D iChannel0;",
    },
    Mapping {
        shadertoy: "",
        godot: "uniform float iTime;",
        glsl: "uniform float iTime;",
    },
];

// find . -type f \
// -exec sh -c 'printf "\n== %s ==\n" "$1"; cat "$1"' _ {} \;

fn main() {
    let src = "fragCoord / iResolution.xy";
    let mapped = translate_token(src, ShaderToy, Godot);
    println!("{} → {:?}", src, mapped);
    convert("glsl/image.glsl", GLSL, Godot, "gdshader/image.gdshader").expect("conversion failed");
    convert(
        "glsl/buffer_a.glsl",
        GLSL,
        Godot,
        "gdshader/buffer_a.gdshader",
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
        "glsl/buffer_a.glsl",
        GLSL,
        ShaderToy,
        "shadertoy/buffer_a.shadertoy",
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
    let mut shader_src = input.to_string();
    let has_nl_eof = shader_src.ends_with('\n');
    shader_src = shader_src
        .replace(MAPPINGS[0].get(from), "")
        .trim_start_matches('\n')
        .to_string();
    for m in &MAPPINGS[1..] {
        let src = m.get(from);
        let dst = m.get(to);
        if src != dst {
            shader_src = if dst.is_empty() {
                shader_src
                    .lines()
                    .filter(|l| !l.contains(src))
                    .collect::<Vec<_>>()
                    .join("\n")
            } else {
                shader_src.replace(src, dst)
            }
        }
    }
    let new_hdr = MAPPINGS[0].get(to);
    if !new_hdr.is_empty() {
        shader_src = format!("{}\n{}", new_hdr, shader_src.trim_start_matches('\n'));
    }
    if has_nl_eof && !shader_src.ends_with('\n') {
        shader_src.push('\n')
    }
    if !has_nl_eof && shader_src.ends_with('\n') {
        shader_src.pop();
    }
    shader_src
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
