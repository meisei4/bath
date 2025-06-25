use bath::render::raylib_util::{
    create_rgba16_render_texture, feedback_buffer_pass, image_pass, APPLE_DPI, EXPERIMENTAL_WINDOW_HEIGHT,
    EXPERIMENTAL_WINDOW_WIDTH,
};
use bath_resources::glsl;
use raylib::init;
use raylib::shaders::RaylibShader;
use std::fs::read_to_string;
use std::mem::swap;
use std::time::Instant;

fn main() {
    let (mut raylib_handle, raylib_thread) = init()
        .size(
            EXPERIMENTAL_WINDOW_WIDTH / APPLE_DPI,
            EXPERIMENTAL_WINDOW_HEIGHT / APPLE_DPI,
        )
        .title("raylib_bath-rs hello world feedback buffer test")
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

    let feedback_buffer_src_code = read_to_string(glsl::BUFFER_A_PATH).unwrap();
    let image_src_code = read_to_string(glsl::IMAGE_PATH).unwrap();

    let mut feedback_buffer_shader =
        raylib_handle.load_shader_from_memory(&raylib_thread, None, Some(&feedback_buffer_src_code));
    let mut image_shader = raylib_handle.load_shader_from_memory(&raylib_thread, None, Some(&image_src_code));

    let i_time_location = feedback_buffer_shader.get_shader_location("iTime");
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
        image_pass(
            &mut raylib_handle,
            &raylib_thread,
            &mut image_shader,
            &mut buffer_a_texture,
        );
    }
}
