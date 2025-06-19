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
            Context::ShaderToy => self.shadertoy,
            Context::Godot => self.godot,
            Context::GLSL => self.glsl,
        };
        let dst = match to {
            Context::ShaderToy => self.shadertoy,
            Context::Godot => self.godot,
            Context::GLSL => self.glsl,
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
        glsl: "#version 330 core",
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
        shadertoy: "fragColor",
        godot: "COLOR",
        glsl: "finalColor",
    },
    Mapping {
        shadertoy: "fragCoord",
        godot: "FRAGCOORD",
        glsl: "gl_FragCoord.xy",
    },
];

pub fn translate_token(token: &str, from: Context, to: Context) -> Option<&'static str> {
    for m in MAPPINGS {
        let src = match from {
            Context::ShaderToy => m.shadertoy,
            Context::Godot => m.godot,
            Context::GLSL => m.glsl,
        };
        if src == token {
            return Some(match to {
                Context::ShaderToy => m.shadertoy,
                Context::Godot => m.godot,
                Context::GLSL => m.glsl,
            });
        }
    }
    None
}

fn main() {
    let src = "fragCoord / iResolution.xy";
    let mapped = translate_token(src, Context::ShaderToy, Context::Godot);
    println!("{} → {:?}", src, mapped);
}
