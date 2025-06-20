use raylib::callbacks::TraceLogLevel::{LOG_INFO, LOG_WARNING};
use raylib::color::Color;
use raylib::drawing::{RaylibDraw, RaylibShaderModeExt, RaylibTextureModeExt};
use raylib::ffi::rlFramebufferAttachTextureType::{RL_ATTACHMENT_RENDERBUFFER, RL_ATTACHMENT_TEXTURE2D};
use raylib::ffi::rlFramebufferAttachType::{RL_ATTACHMENT_COLOR_CHANNEL0, RL_ATTACHMENT_DEPTH};
use raylib::ffi::{
    rlClearColor, rlClearScreenBuffers, rlDisableColorBlend, rlDisableFramebuffer, rlEnableFramebuffer,
    rlFramebufferAttach, rlFramebufferComplete, rlLoadFramebuffer, rlLoadTexture, rlLoadTextureDepth,
    rlTextureParameters, LoadImage, LoadTextureFromImage, TraceLog, UnloadImage, RL_TEXTURE_FILTER_MIP_NEAREST,
    RL_TEXTURE_MAG_FILTER, RL_TEXTURE_MIN_FILTER, RL_TEXTURE_WRAP_REPEAT, RL_TEXTURE_WRAP_S, RL_TEXTURE_WRAP_T,
};
use raylib::math::{Rectangle, Vector2};
use raylib::prelude::{RenderTexture2D, Shader};
use raylib::shaders::RaylibShader;
use raylib::texture::Texture2D;
use raylib::{init, RaylibHandle, RaylibThread};
use std::ffi::{c_char, CString};
use std::fs::read_to_string;
use std::path::Path;

const ORIGIN: Vector2 = Vector2::zero();
const APPLE_DPI: i32 = 2;
const WINDOW_WIDTH: i32 = 850;
const WINDOW_HEIGHT: i32 = 480;

fn main() {
    let (mut rl, thread) =
        init().size(WINDOW_WIDTH / APPLE_DPI, WINDOW_HEIGHT / APPLE_DPI).title("drekker effect").build();
    rl.set_target_fps(60);
    let screen_w = rl.get_screen_width();
    let screen_h = rl.get_screen_height();
    let project_root = env!("CARGO_MANIFEST_DIR");
    let glsl_dir = format!("{}/resources/glsl", project_root);
    let drekker_src = load_shader_with_includes(&format!("{}/color/drekker_effect.glsl", glsl_dir));
    let mut shader = rl.load_shader_from_memory(&thread, None, Some(&drekker_src));
    let i_res_loc = shader.get_shader_location("iResolution");
    let i_channel_1_loc = shader.get_shader_location("iChannel1");
    shader.set_shader_value(i_channel_1_loc, 0);
    shader.set_shader_value(i_res_loc, [screen_w as f32, screen_h as f32]);
    let image_path = CString::new(format!("{}/../godot/Resources/textures/icebergs.jpg", project_root)).unwrap();
    let image_texture = unsafe {
        let image_raw = LoadImage(image_path.as_ptr());
        let mut image_texture_raw = LoadTextureFromImage(image_raw);
        rlTextureParameters(image_texture_raw.id, RL_TEXTURE_WRAP_S as i32, RL_TEXTURE_WRAP_REPEAT as i32);
        rlTextureParameters(image_texture_raw.id, RL_TEXTURE_WRAP_T as i32, RL_TEXTURE_WRAP_REPEAT as i32);
        rlTextureParameters(image_texture_raw.id, RL_TEXTURE_MIN_FILTER as i32, RL_TEXTURE_FILTER_MIP_NEAREST as i32);
        rlTextureParameters(image_texture_raw.id, RL_TEXTURE_MAG_FILTER as i32, RL_TEXTURE_FILTER_MIP_NEAREST as i32);
        UnloadImage(image_raw);
        Texture2D::from_raw(image_texture_raw)
    };
    shader.set_shader_value_texture(i_channel_1_loc, image_texture);
    let mut rgba16_render_texture = create_rgba16_render_texture(screen_w, screen_h);

    while !rl.window_should_close() {
        {
            let mut texture_mode = rl.begin_texture_mode(&thread, &mut rgba16_render_texture);
            let mut shader_mode = texture_mode.begin_shader_mode(&mut shader);
            shader_mode.draw_rectangle(0, 0, screen_w, screen_h, Color::WHITE);
        }
        {
            let mut draw = rl.begin_drawing(&thread);
            let flipped = Rectangle {
                x: 0.0,
                y: 0.0,
                width: rgba16_render_texture.texture.width as f32,
                height: -1.0 * rgba16_render_texture.texture.height as f32,
            };
            draw.draw_texture_rec(&rgba16_render_texture, flipped, ORIGIN, Color::WHITE);
        }
    }
}

fn load_shader_with_includes(path: impl AsRef<Path>) -> String {
    let path = path.as_ref().canonicalize().expect("bad shader path");
    let parent = path.parent().unwrap();
    let src = read_to_string(&path).unwrap_or_else(|e| panic!("Failed to read {:?}: {}", path, e));
    let mut out = String::new();
    for line in src.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("#include") {
            if let Some(name) = rest.trim().strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
                let incl = parent.join(name);
                out.push_str(&load_shader_with_includes(incl));
                continue;
            }
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

fn create_rgba16_render_texture(width: i32, height: i32) -> RenderTexture2D {
    let render_texture = unsafe {
        let fbo_id = rlLoadFramebuffer();
        rlEnableFramebuffer(fbo_id);
        let texture_id = rlLoadTexture(
            std::ptr::null(),
            width,
            height,
            raylib::ffi::PixelFormat::PIXELFORMAT_UNCOMPRESSED_R16G16B16A16 as i32,
            1,
        );
        let depth_texture_id = rlLoadTextureDepth(width, height, true);
        let raw_render_texture = raylib::ffi::RenderTexture2D {
            id: fbo_id,
            texture: raylib::ffi::Texture2D {
                id: texture_id,
                width,
                height,
                mipmaps: 1,
                format: raylib::ffi::PixelFormat::PIXELFORMAT_UNCOMPRESSED_R16G16B16A16 as i32,
            },
            depth: raylib::ffi::Texture2D {
                id: depth_texture_id,
                width,
                height,
                mipmaps: 1,
                format: 19i32,
            },
        };
        rlClearColor(0, 0, 0, 0);
        rlClearScreenBuffers();
        rlFramebufferAttach(fbo_id, texture_id, RL_ATTACHMENT_COLOR_CHANNEL0 as i32, RL_ATTACHMENT_TEXTURE2D as i32, 0);
        rlFramebufferAttach(fbo_id, depth_texture_id, RL_ATTACHMENT_DEPTH as i32, RL_ATTACHMENT_RENDERBUFFER as i32, 0);
        if rlFramebufferComplete(fbo_id) {
            TraceLog(
                LOG_INFO as i32,
                b"FBO: [ID %i] Framebuffer object created successfully\0".as_ptr().cast::<c_char>(),
                fbo_id,
            );
        } else {
            TraceLog(
                LOG_WARNING as i32,
                b"FBO: [ID %i] Framebuffer object is not complete\0".as_ptr().cast::<c_char>(),
                fbo_id,
            );
        }
        rlDisableColorBlend();
        rlDisableFramebuffer();
        RenderTexture2D::from_raw(raw_render_texture)
    };
    render_texture
}

fn image_pass(
    raylib_handle: &mut RaylibHandle,
    raylib_thread: &RaylibThread,
    image_shader: &mut Shader,
    buffer_a_texture: &RenderTexture2D,
) {
    let mut draw_handle = raylib_handle.begin_drawing(raylib_thread);
    let mut shader_mode = draw_handle.begin_shader_mode(image_shader);
    let flipped_rectangle = Rectangle {
        x: 0.0,
        y: 0.0,
        width: buffer_a_texture.texture.width as f32,
        height: -1.0 * buffer_a_texture.texture.height as f32,
    };
    shader_mode.draw_texture_rec(&buffer_a_texture, flipped_rectangle, ORIGIN, Color::WHITE);
}
