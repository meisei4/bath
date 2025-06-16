use raylib::color::Color;
use raylib::drawing::{RaylibDraw, RaylibShaderModeExt, RaylibTextureModeExt};
use raylib::math::{Rectangle, Vector2};
use raylib::texture::RenderTexture2D;
// Texture2D still needed for the field type
use raylib::{init, RaylibHandle, RaylibThread};

use raylib::shaders::{RaylibShader, Shader};
use std::fs::read_to_string;
use std::mem::swap;
use std::time::Instant;

const WINDOW_WIDTH_IN_PIXELS: i32 = 855;
const WINDOW_HEIGHT_IN_PIXELS: i32 = 481;

fn main() {
    let (mut raylib_handle, raylib_thread) = init()
        .size(WINDOW_WIDTH_IN_PIXELS, WINDOW_HEIGHT_IN_PIXELS)
        .title("raylib-rs Shader Test")
        .build();
    raylib_handle.set_target_fps(60);

    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let buffer_a_path = format!("{manifest_dir}/resources/buffer_a.glsl");
    let image_path = format!("{manifest_dir}/resources/image.glsl");
    let buffer_a_src = read_to_string(&buffer_a_path).unwrap();
    let image_src = read_to_string(&image_path).unwrap();
    let mut buffer_a = raylib_handle.load_shader_from_memory(&raylib_thread, None, Some(&buffer_a_src));
    let mut image = raylib_handle.load_shader_from_memory(&raylib_thread, None, Some(&image_src));

    let i_resolution = buffer_a.get_shader_location("iResolution");
    let i_time = buffer_a.get_shader_location("iTime");
    let i_channel0_buf = buffer_a.get_shader_location("iChannel0");
    let i_channel0_img = image.get_shader_location("iChannel0");
    let i_resolution_img = image.get_shader_location("iResolution");
    image.set_shader_value(
        i_resolution_img,
        Vector2::new(
            WINDOW_WIDTH_IN_PIXELS as f32,
            WINDOW_HEIGHT_IN_PIXELS as f32,
        ),
    );

    let mut feedback_texture_a = raylib_handle
        .load_render_texture(
            &raylib_thread,
            WINDOW_WIDTH_IN_PIXELS as u32,
            WINDOW_HEIGHT_IN_PIXELS as u32,
        )
        .expect("cannot create RT A");
    
    let mut feedback_texture_b = raylib_handle
        .load_render_texture(
            &raylib_thread,
            WINDOW_WIDTH_IN_PIXELS as u32,
            WINDOW_HEIGHT_IN_PIXELS as u32,
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
            &mut buffer_a,
            i_channel0_buf,
            i_resolution,
            i_time,
            elapsed_seconds,
        );

        swap(&mut feedback_texture_source, &mut feedback_texture_target);

        image_pass(
            &mut raylib_handle,
            &raylib_thread,
            feedback_texture_source,
            &mut image,
            i_channel0_img,
        );
    }
}

fn buffer_pass(
    raylib_handle: &mut RaylibHandle,
    raylib_thread: &RaylibThread,
    feedback_texture_target: &mut RenderTexture2D,
    feedback_texture_source: &RenderTexture2D,
    buffer_a: &mut Shader,
    i_channel_0: i32,
    i_resolution: i32,
    i_time: i32,
    elapsed_seconds: f32,
) {
    buffer_a.set_shader_value_texture(i_channel_0, feedback_texture_source);
    buffer_a.set_shader_value(
        i_resolution,
        Vector2::new(
            WINDOW_WIDTH_IN_PIXELS as f32,
            WINDOW_HEIGHT_IN_PIXELS as f32,
        ),
    );
    buffer_a.set_shader_value(i_time, elapsed_seconds);

    let mut texture_mode = raylib_handle.begin_texture_mode(raylib_thread, feedback_texture_target);
    let mut shader_mode = texture_mode.begin_shader_mode(buffer_a);

    let src_rect = Rectangle {
        x: 0.0,
        y: 0.0,
        width:  feedback_texture_source.texture.width  as f32,
        height: -(feedback_texture_source.texture.height as f32),
    };
    let dest_rect = Rectangle {
        x: 0.0,
        y: 0.0,
        width: WINDOW_WIDTH_IN_PIXELS as f32,
        height: WINDOW_HEIGHT_IN_PIXELS as f32,
    };
    shader_mode.draw_texture_pro(
        feedback_texture_source,
        src_rect,
        dest_rect,
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
    i_channel_0: i32,
) {
    image_shader.set_shader_value_texture(i_channel_0, screen_source_texture);

    let mut draw_handle = raylib_handle.begin_drawing(raylib_thread);
    draw_handle.clear_background(Color::BLACK);
    let mut shader_mode = draw_handle.begin_shader_mode(image_shader);

    let src_rect = Rectangle {
        x: 0.0,
        y: 0.0,
        width:  screen_source_texture.texture.width  as f32,
        height: -(screen_source_texture.texture.height as f32),
    };
    let dest_rect = Rectangle {
        x: 0.0,
        y: 0.0,
        width: WINDOW_WIDTH_IN_PIXELS as f32,
        height: WINDOW_HEIGHT_IN_PIXELS as f32,
    };
    shader_mode.draw_texture_pro(
        screen_source_texture,
        src_rect,
        dest_rect,
        Vector2::zero(),
        0.0,
        Color::WHITE,
    );
}
