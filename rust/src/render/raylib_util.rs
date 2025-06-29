use raylib::color::Color;
use raylib::drawing::{RaylibDraw, RaylibShaderModeExt, RaylibTextureModeExt};
use raylib::ffi::rlFramebufferAttachTextureType::{RL_ATTACHMENT_RENDERBUFFER, RL_ATTACHMENT_TEXTURE2D};
use raylib::ffi::rlFramebufferAttachType::{RL_ATTACHMENT_COLOR_CHANNEL0, RL_ATTACHMENT_DEPTH};
use raylib::ffi::TraceLogLevel::{LOG_INFO, LOG_WARNING};
use raylib::ffi::{
    rlClearColor, rlClearScreenBuffers, rlDisableColorBlend, rlDisableFramebuffer, rlEnableFramebuffer,
    rlFramebufferAttach, rlFramebufferComplete, rlLoadFramebuffer, rlLoadTexture, rlLoadTextureDepth, Texture2D,
    TraceLog,
};
use raylib::math::{Rectangle, Vector2};
use raylib::prelude::Shader;
use raylib::texture::RenderTexture2D;
use raylib::{RaylibHandle, RaylibThread};
use std::ffi::c_char;
use std::fs::read_to_string;
use std::path::Path;

pub const ORIGIN_X: i32 = 0;
pub const ORIGIN_Y: i32 = 0;
pub const ORIGIN: Vector2 = Vector2::zero();
pub const APPLE_DPI: i32 = 2;
pub const EXPERIMENTAL_WINDOW_WIDTH: i32 = 850;
pub const EXPERIMENTAL_WINDOW_HEIGHT: i32 = 480;

pub const BATH_WIDTH: i32 = 256 * 2;
pub const BATH_HEIGHT: i32 = 384 * 2;

pub fn create_rgba16_render_texture(width: i32, height: i32) -> RenderTexture2D {
    // raylib code: https://github.com/raysan5/raylib/blob/4bc8d3761c48f4dcf56f126640da8f3567dc516b/src/rtextures.c#L4246
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
        // TODO: lmao try setting useRenderBuffer to false and see what happens
        // let depth_texture_id = rlLoadTextureDepth(width, height, false);
        let depth_texture_id = rlLoadTextureDepth(width, height, true);
        let raw_render_texture = raylib::ffi::RenderTexture2D {
            id: fbo_id,
            texture: Texture2D {
                id: texture_id,
                width,
                height,
                mipmaps: 1,
                format: raylib::ffi::PixelFormat::PIXELFORMAT_UNCOMPRESSED_R16G16B16A16 as i32,
            },
            depth: Texture2D {
                id: depth_texture_id,
                width,
                height,
                mipmaps: 1,
                format: 19i32,
            },
        };
        //TODO: just good practice before attaching to FBOs
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
        rlDisableColorBlend();
        rlDisableFramebuffer();
        RenderTexture2D::from_raw(raw_render_texture)
    };
    render_texture
}

pub fn load_shader_with_includes(path: impl AsRef<Path>) -> String {
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

pub fn feedback_buffer_pass(
    raylib_handle: &mut RaylibHandle,
    raylib_thread: &RaylibThread,
    feedback_buffer_shader: &mut Shader,
    buffer_b_texture: &mut RenderTexture2D,
    buffer_a_texture: &RenderTexture2D,
) {
    let mut texture_mode = raylib_handle.begin_texture_mode(raylib_thread, buffer_b_texture);
    let mut shader_mode = texture_mode.begin_shader_mode(feedback_buffer_shader);
    //TODO: raylib_bath has to flip even at the buffer stage? ugh, am i dumb??
    // uncomment the next line, and then comment out the flipping to see behavior
    //shader_mode.draw_texture(&buffer_a_texture, ORIGIN_X, ORIGIN_Y, Color::WHITE);
    let flipped_framebuffer = flip_framebuffer(
        buffer_a_texture.texture.width as f32,
        buffer_a_texture.texture.height as f32,
    );
    shader_mode.draw_texture_rec(&buffer_a_texture, flipped_framebuffer, ORIGIN, Color::WHITE);
}

pub fn image_pass(
    raylib_handle: &mut RaylibHandle,
    raylib_thread: &RaylibThread,
    image_shader: &mut Shader,
    buffer_a_texture: &RenderTexture2D,
) {
    let mut draw_handle = raylib_handle.begin_drawing(raylib_thread);
    let mut shader_mode = draw_handle.begin_shader_mode(image_shader);
    // TODO: classic raster stage y flip because shadertoy, same thing in godot
    //  uncomment the next line, and then comment out the flipping to see behavior
    //shader_mode.draw_texture(&buffer_a_texture, ORIGIN_X, ORIGIN_Y, Color::WHITE);
    let flipped_framebuffer = flip_framebuffer(
        buffer_a_texture.texture.width as f32,
        buffer_a_texture.texture.height as f32,
    );
    shader_mode.draw_texture_rec(&buffer_a_texture, flipped_framebuffer, ORIGIN, Color::WHITE);
}

pub fn flip_framebuffer(width: f32, height: f32) -> Rectangle {
    Rectangle {
        x: 0.0,
        y: 0.0,
        width,
        height: -1.0 * height,
    }
}
