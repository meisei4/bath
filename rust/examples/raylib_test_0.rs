use raylib::color::Color;
use raylib::drawing::{RaylibDraw, RaylibShaderModeExt, RaylibTextureModeExt};
use raylib::ffi::{
    rlClearColor, rlClearScreenBuffers, rlDisableColorBlend, rlDisableFramebuffer, rlEnableFramebuffer,
    rlFramebufferAttach, rlFramebufferComplete, rlLoadFramebuffer, rlLoadTexture, rlLoadTextureDepth, Texture2D,
    TraceLog,
};
use raylib::math::{Rectangle, Vector2};
use raylib::shaders::{RaylibShader, Shader};
use raylib::texture::RenderTexture2D;
use raylib::{init, RaylibHandle, RaylibThread};
use std::ffi::c_char;

use raylib::consts::TraceLogLevel::{LOG_INFO, LOG_WARNING};
use raylib::ffi::rlFramebufferAttachTextureType::{RL_ATTACHMENT_RENDERBUFFER, RL_ATTACHMENT_TEXTURE2D};
use raylib::ffi::rlFramebufferAttachType::{RL_ATTACHMENT_COLOR_CHANNEL0, RL_ATTACHMENT_DEPTH};
use std::fs::read_to_string;
use std::mem::swap;
use std::time::Instant;

const ORIGIN: Vector2 = Vector2::zero();
const APPLE_DPI: i32 = 2;
const WINDOW_WIDTH: i32 = 850;
const WINDOW_HEIGHT: i32 = 480;

fn main() {
    let (mut raylib_handle, raylib_thread) = init()
        .size(WINDOW_WIDTH / APPLE_DPI, WINDOW_HEIGHT / APPLE_DPI)
        .title("raylib-rs hello world feedback buffer test")
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
    let feedback_buffer_src_code = read_to_string(format!("{project_root_dir}/resources/glsl/buffer_a.glsl")).unwrap();
    let image_src_code = read_to_string(format!("{project_root_dir}/resources/glsl/image.glsl")).unwrap();

    let mut feedback_buffer_shader =
        raylib_handle.load_shader_from_memory(&raylib_thread, None, Some(&feedback_buffer_src_code));
    let mut image_shader = raylib_handle.load_shader_from_memory(&raylib_thread, None, Some(&image_src_code));

    let i_time_location = feedback_buffer_shader.get_shader_location("iTime");
    let buffer_i_channel0_location = feedback_buffer_shader.get_shader_location("iChannel0");
    let image_i_channel1_location = image_shader.get_shader_location("iChannel1");

    feedback_buffer_shader.set_shader_value(buffer_i_channel0_location, 0);
    image_shader.set_shader_value(image_i_channel1_location, 0);

    let mut buffer_a_texture = create_rgba16_render_texture(screen_width, screen_height);
    let mut buffer_b_texture = create_rgba16_render_texture(screen_width, screen_height);

    let application_start_time = Instant::now();

    while !raylib_handle.window_should_close() {
        let elapsed_seconds = application_start_time.elapsed().as_secs_f32();
        feedback_buffer_shader.set_shader_value(i_time_location, elapsed_seconds);
        feedback_buffer_pass(
            &mut raylib_handle,
            &raylib_thread,
            &mut feedback_buffer_shader,
            &mut buffer_b_texture,
            &mut buffer_a_texture,
        );
        swap(&mut buffer_a_texture, &mut buffer_b_texture);
        image_pass(&mut raylib_handle, &raylib_thread, &mut image_shader, &mut buffer_a_texture);
    }
}

fn create_rgba16_render_texture(width: i32, height: i32) -> RenderTexture2D {
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
    render_texture //TODO: I like this because it reminds me how returns work
}

fn feedback_buffer_pass(
    raylib_handle: &mut RaylibHandle,
    raylib_thread: &RaylibThread,
    feedback_buffer_shader: &mut Shader,
    buffer_b_texture: &mut RenderTexture2D,
    buffer_a_texture: &RenderTexture2D,
) {
    let mut texture_mode = raylib_handle.begin_texture_mode(raylib_thread, buffer_b_texture);
    let mut shader_mode = texture_mode.begin_shader_mode(feedback_buffer_shader);
    //TODO: raylib has to flip even at the buffer stage? ugh, am i dumb??
    // uncomment the next line, and then comment out the flipping to see behavior
    //shader_mode.draw_texture(&buffer_a_texture, ORIGIN_X, ORIGIN_Y, Color::WHITE);
    let flipped_rectangle = Rectangle {
        x: 0.0,
        y: 0.0,
        width: buffer_a_texture.texture.width as f32,
        height: -1.0 * buffer_a_texture.texture.height as f32,
    };
    shader_mode.draw_texture_rec(&buffer_a_texture, flipped_rectangle, ORIGIN, Color::WHITE);
}

fn image_pass(
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
    let flipped_rectangle = Rectangle {
        x: 0.0,
        y: 0.0,
        width: buffer_a_texture.texture.width as f32,
        height: -1.0 * buffer_a_texture.texture.height as f32,
    };
    shader_mode.draw_texture_rec(&buffer_a_texture, flipped_rectangle, ORIGIN, Color::WHITE);
}
