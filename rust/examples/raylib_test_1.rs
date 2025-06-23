use bath::raylib_bath::util::{
    create_rgba16_render_texture, load_shader_with_includes, APPLE_DPI, ORIGIN, WINDOW_HEIGHT, WINDOW_WIDTH,
};
use raylib::color::Color;
use raylib::drawing::{RaylibDraw, RaylibShaderModeExt, RaylibTextureModeExt};
use raylib::ffi::{
    rlTextureParameters, LoadImage, LoadTextureFromImage, UnloadImage, RL_TEXTURE_FILTER_NEAREST,
    RL_TEXTURE_MAG_FILTER, RL_TEXTURE_MIN_FILTER, RL_TEXTURE_WRAP_REPEAT, RL_TEXTURE_WRAP_S, RL_TEXTURE_WRAP_T,
};
use raylib::init;
use raylib::math::{Rectangle, Vector2};
use raylib::shaders::RaylibShader;
use raylib::texture::Texture2D;
use std::ffi::CString;

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
    //drekker_shader.set_shader_value_texture(i_channel_1_location, &buffer_a_texture);
    while !raylib_handle.window_should_close() {
        // let buffer_a_texture_id = buffer_a_texture.texture().id;
        // let texture_slot: i32 = 7; // LUCKY 7!!! arbitrary see: https://github.com/raysan5/raylib/issues/4568
        // unsafe {
        //     // NOTE: this is the most safest code to run ever in the entire world, rust i will never trust you ever, you wasted 5 hours of my life
        //     rlActiveTextureSlot(texture_slot);
        //     rlEnableTexture(buffer_a_texture_id);
        // }
        //drekker_shader.set_shader_value(i_channel_1_location, texture_slot);
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
