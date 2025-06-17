use raylib::color::Color;
use raylib::drawing::{RaylibDraw, RaylibShaderModeExt, RaylibTextureModeExt};
use raylib::math::{Rectangle, Vector2};
use raylib::shaders::{RaylibShader, Shader};
use raylib::texture::RenderTexture2D;
use raylib::{init, RaylibHandle, RaylibThread};

use std::fs::read_to_string;
use std::mem::swap;
use std::time::Instant;

const APPLE_DPI: u32 = 2;
const WINDOW_WIDTH_IN_PIXELS: i32 = (850 / APPLE_DPI) as i32;
const WINDOW_HEIGHT_IN_PIXELS: i32 = (480 / APPLE_DPI) as i32;

fn main() {
    let (mut raylib_handle, raylib_thread) = init()
        .size(WINDOW_WIDTH_IN_PIXELS, WINDOW_HEIGHT_IN_PIXELS)
        .title("raylib-rs hello world feedback buffer test")
        .build();
    raylib_handle.set_target_fps(60);

    let render_w = raylib_handle.get_render_width();
    let render_h = raylib_handle.get_render_height();
    println!(
        "screen size:  {}x{}",
        raylib_handle.get_screen_width(),
        raylib_handle.get_screen_height()
    );
    println!("render size:  {}x{}", render_w, render_h);
    println!("DPI scale:    {:?}", raylib_handle.get_window_scale_dpi());

    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let buffer_a_src = read_to_string(format!("{manifest_dir}/resources/buffer_a.glsl")).unwrap();
    let image_src = read_to_string(format!("{manifest_dir}/resources/image.glsl")).unwrap();

    let mut buffer_a =
        raylib_handle.load_shader_from_memory(&raylib_thread, None, Some(&buffer_a_src));
    let mut image = raylib_handle.load_shader_from_memory(&raylib_thread, None, Some(&image_src));

    let i_time = buffer_a.get_shader_location("iTime");
    let i_channel0_buf = buffer_a.get_shader_location("iChannel0");
    let i_channel0_img = image.get_shader_location("iChannel0");

    buffer_a.set_shader_value(i_channel0_buf, 0); // sampler = texture unit 0
    image.set_shader_value(i_channel0_img, 0);
    let mut feedback_texture_a = raylib_handle
        .load_render_texture(
            &raylib_thread,
            render_w as u32 / APPLE_DPI,
            render_h as u32 / APPLE_DPI,
        )
        .expect("cannot create RT A");
    let mut feedback_texture_b = raylib_handle
        .load_render_texture(
            &raylib_thread,
            render_w as u32 / APPLE_DPI,
            render_h as u32 / APPLE_DPI,
        )
        .expect("cannot create RT B");

    let mut feedback_texture_source: &mut RenderTexture2D = &mut feedback_texture_a;
    let mut feedback_texture_target: &mut RenderTexture2D = &mut feedback_texture_b;

    let application_start_time = Instant::now();

    while !raylib_handle.window_should_close() {
        let elapsed_seconds = application_start_time.elapsed().as_secs_f32();

        buffer_pass(
            &mut raylib_handle,
            &raylib_thread,
            feedback_texture_target,
            feedback_texture_source,
            elapsed_seconds,
            &mut buffer_a,
            i_time,
        );

        swap(&mut feedback_texture_source, &mut feedback_texture_target);

        image_pass(
            &mut raylib_handle,
            &raylib_thread,
            feedback_texture_source,
            &mut image,
        );
    }
}

fn buffer_pass(
    raylib_handle: &mut RaylibHandle,
    raylib_thread: &RaylibThread,
    feedback_texture_target: &mut RenderTexture2D,
    feedback_texture_source: &RenderTexture2D,
    elapsed_seconds: f32,
    buffer_a: &mut Shader,
    i_time: i32,
) {
    let src_w = feedback_texture_source.texture.width as f32; // / APPLE_DPI as f32;
    let src_h = feedback_texture_source.texture.height as f32;
    let dst_w = feedback_texture_target.texture.width as f32;
    let dst_h = feedback_texture_target.texture.height as f32;

    buffer_a.set_shader_value(i_time, elapsed_seconds);

    let mut texture_mode = raylib_handle.begin_texture_mode(raylib_thread, feedback_texture_target);
    texture_mode.clear_background(Color::BLACK);
    let mut shader_mode = texture_mode.begin_shader_mode(buffer_a);

    let src_rect = Rectangle {
        x: 0.0,
        y: 0.0,
        width: src_w,
        height: -src_h,
    };
    let dst_rect = Rectangle {
        x: 0.0,
        y: 0.0,
        width: dst_w,
        height: dst_h,
    };
    shader_mode.draw_texture_pro(
        feedback_texture_source,
        src_rect,
        dst_rect,
        Vector2::zero(),
        0.0,
        Color::WHITE,
    );
}

fn image_pass(
    raylib_handle: &mut RaylibHandle,
    raylib_thread: &RaylibThread,
    screen_source_texture: &RenderTexture2D,
    image_shader: &mut Shader,
) {
    let w = screen_source_texture.texture.width;
    let h = screen_source_texture.texture.height;
    let mut draw_handle = raylib_handle.begin_drawing(raylib_thread);
    let mut shader_mode = draw_handle.begin_shader_mode(image_shader);

    let src_rect = Rectangle {
        x: 0.0,
        y: 0.0,
        width: w as f32,
        height: -(h as f32),
    };
    let dst_rect = Rectangle {
        x: 0.0,
        y: 0.0,
        width: w as f32,
        height: h as f32,
    };
    shader_mode.draw_texture_pro(
        screen_source_texture,
        src_rect,
        dst_rect,
        Vector2::zero(),
        0.0,
        Color::WHITE,
    );
}
