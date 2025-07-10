use bath::render::raylib_util::{
    create_rgba16_render_texture, flip_framebuffer, load_shader_with_includes, APPLE_DPI, EXPERIMENTAL_WINDOW_HEIGHT,
    EXPERIMENTAL_WINDOW_WIDTH, ORIGIN,
};

use asset_payload::runtime_io::{DREKKER_PATH, ICEBERGS_JPG_PATH, RAYLIB_DEFAULT_VERT_PATH};
use raylib::color::Color;
use raylib::drawing::{RaylibDraw, RaylibShaderModeExt, RaylibTextureModeExt};
use raylib::ffi::{
    rlTextureParameters, LoadImage, LoadTextureFromImage, UnloadImage, RL_TEXTURE_FILTER_NEAREST,
    RL_TEXTURE_MAG_FILTER, RL_TEXTURE_MIN_FILTER, RL_TEXTURE_WRAP_REPEAT, RL_TEXTURE_WRAP_S, RL_TEXTURE_WRAP_T,
};
use raylib::init;
use raylib::math::Vector2;
use raylib::shaders::RaylibShader;
use raylib::texture::Texture2D;
use std::ffi::CString;
use std::fs::read_to_string;

fn main() {
    let (mut raylib_handle, raylib_thread) = init()
        .size(
            EXPERIMENTAL_WINDOW_WIDTH / APPLE_DPI,
            EXPERIMENTAL_WINDOW_HEIGHT / APPLE_DPI,
        )
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
    let raylib_vertex_shader_src_code = read_to_string(RAYLIB_DEFAULT_VERT_PATH).unwrap();

    let drekker_src = load_shader_with_includes(DREKKER_PATH);
    let mut shader =
        raylib_handle.load_shader_from_memory(&raylib_thread, Some(&raylib_vertex_shader_src_code), Some(&drekker_src));
    let i_resolution_location = shader.get_shader_location("iResolution");
    let i_channel_1_location = shader.get_shader_location("iChannel1");
    println!(
        "iChannel1 loc = {}, iResolution loc = {}",
        i_channel_1_location, i_resolution_location
    );
    let i_resolution = Vector2::new(screen_width as f32, screen_height as f32);
    shader.set_shader_value_v(i_resolution_location, &[i_resolution]);
    let image_path = CString::new(ICEBERGS_JPG_PATH).unwrap();
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
    let flipped_framebuffer = flip_framebuffer(
        buffer_a_texture.texture.width as f32,
        buffer_a_texture.texture.height as f32,
    );
    {
        let mut texture_mode = raylib_handle.begin_texture_mode(&raylib_thread, &mut buffer_a_texture);
        texture_mode.clear_background(Color::BLACK);
        texture_mode.draw_texture_rec(&image_texture, flipped_framebuffer, ORIGIN, Color::WHITE);
    }
    // TODO: never forget about this please never, you spent hours on this issue
    // shader.set_shader_value_texture(i_channel_1_location, &buffer_a_texture);
    while !raylib_handle.window_should_close() {
        // let buffer_a_texture_id = buffer_a_texture.texture().id;
        // TODO: lucky 7.... arbitrary out of 0-15 depending on GL version see: https://github.com/raysan5/raylib/issues/4568
        // let texture_slot: i32 = 7;
        // unsafe {
        //     rlActiveTextureSlot(texture_slot);
        //     rlEnableTexture(buffer_a_texture_id);
        // }
        // shader.set_shader_value(i_channel_1_location, texture_slot);
        let mut draw_handle = raylib_handle.begin_drawing(&raylib_thread);
        let mut shader_mode = draw_handle.begin_shader_mode(&mut shader);
        shader_mode.draw_texture_rec(&buffer_a_texture, flipped_framebuffer, ORIGIN, Color::WHITE);
    }
}
