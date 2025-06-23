use bath::raylib_bath::util::{
    create_rgba16_render_texture, load_shader_with_includes, APPLE_DPI, ORIGIN, WINDOW_HEIGHT, WINDOW_WIDTH,
};
use raylib::color::Color;
use raylib::drawing::{RaylibDraw, RaylibShaderModeExt, RaylibTextureModeExt};
use raylib::ffi::{
    rlTextureParameters, LoadImage, LoadTextureFromImage, UnloadImage, RL_TEXTURE_FILTER_NEAREST,
    RL_TEXTURE_MAG_FILTER, RL_TEXTURE_MIN_FILTER, RL_TEXTURE_WRAP_REPEAT, RL_TEXTURE_WRAP_S, RL_TEXTURE_WRAP_T,
};
use raylib::math::{Rectangle, Vector2};
use raylib::shaders::{RaylibShader, Shader};
use raylib::texture::{RenderTexture2D, Texture2D};
use raylib::{init, RaylibHandle, RaylibThread};
use std::ffi::CString;
use std::fs::read_to_string;
use std::mem::swap;
use std::time::Instant;

fn main() {
    let (mut raylib_handle, raylib_thread) = init()
        .size(WINDOW_WIDTH / APPLE_DPI, WINDOW_HEIGHT / APPLE_DPI)
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

    let project_root_dir = env!("CARGO_MANIFEST_DIR");
    let feedback_buffer_src_code = read_to_string(format!("{project_root_dir}/resources/glsl/buffer_a.glsl")).unwrap();
    let image_src_code = read_to_string(format!("{project_root_dir}/resources/glsl/image.glsl")).unwrap();
    let drekker_src =
        load_shader_with_includes(&format!("{project_root_dir}/resources/glsl/color/drekker_effect.glsl"));
    let mut feedback_buffer_shader =
        raylib_handle.load_shader_from_memory(&raylib_thread, None, Some(&feedback_buffer_src_code));
    let mut image_shader = raylib_handle.load_shader_from_memory(&raylib_thread, None, Some(&image_src_code));
    let mut drekker_shader = raylib_handle.load_shader_from_memory(&raylib_thread, None, Some(&drekker_src));
    let i_time_location = feedback_buffer_shader.get_shader_location("iTime");
    let i_resolution_location = drekker_shader.get_shader_location("iResolution");
    let i_channel_1_location = drekker_shader.get_shader_location("iChannel1");
    println!(
        "iChannel1 loc = {}, iResolution loc = {}",
        i_channel_1_location, i_resolution_location
    );
    let i_resolution = Vector2::new(screen_width as f32, screen_height as f32);
    drekker_shader.set_shader_value_v(i_resolution_location, &[i_resolution]);
    let mut buffer_a_texture = create_rgba16_render_texture(screen_width, screen_height);
    let mut buffer_b_texture = create_rgba16_render_texture(screen_width, screen_height);
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
    //TODO: how does this differ from the feeback, do we need three?
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
        let mut draw_handle = raylib_handle.begin_drawing(&raylib_thread);
        let mut shader_mode = draw_handle.begin_shader_mode(&mut drekker_shader);
        shader_mode.draw_texture_rec(&buffer_a_texture, flipped_rectangle, Vector2::zero(), Color::WHITE);
    }
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
    let flipped_rectangle = Rectangle {
        x: 0.0,
        y: 0.0,
        width: buffer_a_texture.texture.width as f32,
        height: -1.0 * buffer_a_texture.texture.height as f32,
    };
    shader_mode.draw_texture_rec(&buffer_a_texture, flipped_rectangle, ORIGIN, Color::WHITE);
}
