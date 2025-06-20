use raylib::callbacks::TraceLogLevel::{LOG_INFO, LOG_WARNING};
use raylib::color::Color;
use raylib::drawing::{RaylibDraw, RaylibShaderModeExt, RaylibTextureModeExt};
use raylib::ffi::rlFramebufferAttachTextureType::{RL_ATTACHMENT_RENDERBUFFER, RL_ATTACHMENT_TEXTURE2D};
use raylib::ffi::rlFramebufferAttachType::{RL_ATTACHMENT_COLOR_CHANNEL0, RL_ATTACHMENT_DEPTH};
use raylib::ffi::{
    rlActiveTextureSlot, rlClearColor, rlClearScreenBuffers, rlDisableFramebuffer, rlEnableFramebuffer,
    rlEnableTexture, rlFramebufferAttach, rlFramebufferComplete, rlLoadFramebuffer, rlLoadTexture, rlLoadTextureDepth,
    rlTextureParameters, LoadImage, LoadTextureFromImage, TraceLog, UnloadImage, RL_TEXTURE_FILTER_NEAREST,
    RL_TEXTURE_MAG_FILTER, RL_TEXTURE_MIN_FILTER, RL_TEXTURE_WRAP_REPEAT, RL_TEXTURE_WRAP_S, RL_TEXTURE_WRAP_T,
};
use raylib::init;
use raylib::math::{Rectangle, Vector2};
use raylib::prelude::RenderTexture2D;
use raylib::shaders::RaylibShader;
use raylib::texture::{RaylibRenderTexture2D, Texture2D};
use std::ffi::{c_char, CString};
use std::fs::read_to_string;
use std::path::Path;

const ORIGIN_X: i32 = 0;
const ORIGIN_Y: i32 = 0;

const ORIGIN: Vector2 = Vector2::zero();
const APPLE_DPI: i32 = 2;
const WINDOW_WIDTH: i32 = 850;
const WINDOW_HEIGHT: i32 = 480;

fn main() {
    let (mut raylib_handle, raylib_thread) = init()
        .size(WINDOW_WIDTH / APPLE_DPI, WINDOW_HEIGHT / APPLE_DPI)
        .title("drekker effect")
        .build();
    raylib_handle.set_target_fps(60);
    let screen_width = raylib_handle.get_screen_width();
    let screen_height = raylib_handle.get_screen_height();
    let dpi = raylib_handle.get_window_scale_dpi();
    let render_width = raylib_handle.get_render_width();
    let render_height = raylib_handle.get_render_height();
    println!("screen: {}x{}", screen_width, screen_height);
    println!("render:{}x{}", render_width, render_height);
    println!("dpi: {:?}", dpi);
    let project_root_dir = env!("CARGO_MANIFEST_DIR");
    let glsl_dir = format!("{}/resources/glsl", project_root_dir);
    let drekker_src = load_shader_with_includes(&format!("{}/color/drekker_effect.glsl", glsl_dir));
    let mut drekker_shader = raylib_handle.load_shader_from_memory(&raylib_thread, None, Some(&drekker_src));
    let i_resolution_location = drekker_shader.get_shader_location("iResolution");
    let i_channel_1_location = drekker_shader.get_shader_location("iChannel1");
    println!(
        "iChannel1 loc = {}, iResolution loc = {}",
        i_channel_1_location, i_resolution_location
    );
    let i_resolution = Vector2::new(screen_width as f32, screen_height as f32);
    drekker_shader.set_shader_value_v(i_resolution_location, &[i_resolution]);
    let image_path = CString::new(format!("{}/../godot/Resources/textures/icebergs.jpg", project_root_dir)).unwrap();
    let image_texture = unsafe {
        let image_raw = LoadImage(image_path.as_ptr());
        let image_texture_raw = LoadTextureFromImage(image_raw);
        rlTextureParameters(
            image_texture_raw.id,
            RL_TEXTURE_WRAP_S as i32,
            RL_TEXTURE_WRAP_REPEAT as i32,
        );
        rlTextureParameters(
            image_texture_raw.id,
            RL_TEXTURE_WRAP_T as i32,
            RL_TEXTURE_WRAP_REPEAT as i32,
        );
        rlTextureParameters(
            image_texture_raw.id,
            RL_TEXTURE_MAG_FILTER as i32,
            RL_TEXTURE_FILTER_NEAREST as i32,
        );
        rlTextureParameters(
            image_texture_raw.id,
            RL_TEXTURE_MIN_FILTER as i32,
            RL_TEXTURE_FILTER_NEAREST as i32,
        );
        UnloadImage(image_raw);
        Texture2D::from_raw(image_texture_raw)
    };
    let mut buffer_a_texture = create_rgba16_render_texture(render_width, render_height);
    let flipped_rectangle = Rectangle {
        x: 0.0,
        y: 0.0,
        width: render_width as f32,
        height: -1.0 * render_height as f32,
    };
    {
        let mut texture_mode = raylib_handle.begin_texture_mode(&raylib_thread, &mut buffer_a_texture);
        texture_mode.clear_background(Color::BLACK);
        texture_mode.draw_texture_rec(&image_texture, flipped_rectangle, ORIGIN, Color::WHITE);
    }
    drekker_shader.set_shader_value_texture(i_channel_1_location, &buffer_a_texture);
    while !raylib_handle.window_should_close() {
        let buffer_a_texture_id = buffer_a_texture.texture().id;
        let texture_slot: i32 = 7; // LUCKY 7!!! arbitrary see: https://github.com/raysan5/raylib/issues/4568
        unsafe {
            // NOTE: this is the most safest code to run ever in the entire world, rust i will never trust you ever, you wasted 5 hours of my life
            rlActiveTextureSlot(texture_slot);
            rlEnableTexture(buffer_a_texture_id);
        }
        drekker_shader.set_shader_value(i_channel_1_location, texture_slot);
        let mut draw_handle = raylib_handle.begin_drawing(&raylib_thread);
        let mut shader_mode = draw_handle.begin_shader_mode(&mut drekker_shader);
        shader_mode.draw_texture_rec(&buffer_a_texture, flipped_rectangle, Vector2::zero(), Color::WHITE);
        // TODO: somewhere document the hell you just went through:
        // let mut draw_handle = raylib_handle.begin_drawing(&raylib_thread);
        // draw_handle.draw_texture(&buffer_a_texture, ORIGIN_X, ORIGIN_Y, Color::WHITE);
        // TODO: ^^just the fbo drawn, no shaders, good test^^^
        // let mut shader_mode = draw_handle.begin_shader_mode(&mut image_shader);
        // TODO: this is some  issue in low level graphics shit with quad drawing or something i have no idea
        //  https://github.com/raysan5/raylib/issues/4568
        // shader_mode.draw_texture_rec(&buffer_a_texture, flipped_rectangle, ORIGIN, Color::WHITE);
        // drekker_shader.set_shader_value_texture(i_channel_1_location, &buffer_a_texture_clone);
    }
}

// TODO: copy paste from previous raylib test, next time start moving everything into an organized place
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
        rlFramebufferAttach(
            fbo_id,
            texture_id,
            RL_ATTACHMENT_COLOR_CHANNEL0 as i32,
            RL_ATTACHMENT_TEXTURE2D as i32,
            0,
        );
        rlFramebufferAttach(
            fbo_id,
            depth_texture_id,
            RL_ATTACHMENT_DEPTH as i32,
            RL_ATTACHMENT_RENDERBUFFER as i32,
            0,
        );
        if rlFramebufferComplete(fbo_id) {
            TraceLog(
                LOG_INFO as i32,
                b"FBO: [ID %i] Framebuffer object created successfully\0"
                    .as_ptr()
                    .cast::<c_char>(),
                fbo_id,
            );
        } else {
            TraceLog(
                LOG_WARNING as i32,
                b"FBO: [ID %i] Framebuffer object is not complete\0"
                    .as_ptr()
                    .cast::<c_char>(),
                fbo_id,
            );
        }
        //rlDisableColorBlend();
        rlDisableFramebuffer();
        RenderTexture2D::from_raw(raw_render_texture)
    };
    render_texture
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
