use asset_payload::BAYER_PNG_PATH;
use bath::fixed_func::ghost::{
    add_dither, load_bayer_png, uv_to_grid_space, UMBRAL_MASK_CENTER, UMBRAL_MASK_FADE_BAND, UMBRAL_MASK_OUTER_RADIUS,
};
use bath::render::raylib::RaylibRenderer;
use bath::render::raylib_util::{flip_framebuffer, N64_HEIGHT, N64_WIDTH, ORIGIN};
use bath::render::renderer::Renderer;
use raylib::color::Color;
use raylib::drawing::RaylibDraw;
use raylib::math::Vector2;
use raylib::prelude::Image;
use raylib::texture::RaylibTexture2D;

fn main() {
    let mut render = RaylibRenderer::init(N64_WIDTH, N64_HEIGHT);
    let screen_w = render.handle.get_screen_width();
    let screen_h = render.handle.get_screen_height();
    let i_resolution = Vector2::new(screen_w as f32, screen_h as f32);
    let blank_img = Image::gen_image_color(screen_w, screen_h, Color::BLANK);
    let mut circle_texture = render
        .handle
        .load_texture_from_image(&render.thread, &blank_img)
        .unwrap();
    let mut circle_img = generate_circle_image(screen_w, screen_h);
    let (bayer_data, bayer_w, bayer_h) = load_bayer_png(BAYER_PNG_PATH);
    let mut colors: Vec<u8> = circle_img.get_image_data_u8(false);
    for y in 0..circle_img.height() {
        for x in 0..circle_img.width() {
            let idx = 4 * (y * circle_img.width() + x) as usize;
            let lum = colors[idx] as f32 / 255.0;
            let dither = add_dither(lum, x, y, &bayer_data, bayer_w, bayer_h);
            let v = (dither * 255.0).round() as u8;
            colors[idx] = v; // R
            colors[idx + 1] = v; // G
            colors[idx + 2] = v; // B
            colors[idx + 3] = 255; // A
        }
    }
    circle_texture.update_texture(&colors).unwrap();
    let mut i_time = 0.0f32;
    while !render.handle.window_should_close() {
        i_time += render.handle.get_frame_time();
        let mut draw_handle = render.handle.begin_drawing(&render.thread);
        draw_handle.clear_background(Color::BLACK);
        draw_handle.draw_texture_rec(
            &circle_texture,
            flip_framebuffer(i_resolution.x, i_resolution.y),
            ORIGIN,
            Color::WHITE,
        );
    }
}

#[inline]
fn generate_circle_image(width: i32, height: i32) -> Image {
    let mut img = Image::gen_image_color(width, height, Color::BLANK);
    let total_bytes = (width * height * 4) as usize;
    //TODO: figure out a better way to do this, the amount of ways to do this is insane there are like 5 different ways lol
    let color: &mut [u8] = unsafe { std::slice::from_raw_parts_mut(img.data as *mut u8, total_bytes) };
    for y in 0..height {
        for x in 0..width {
            let s = (x as f32 + 0.5) / width as f32;
            let t = (y as f32 + 0.5) / height as f32;
            let frag_tex_coord = Vector2::new(s, t);
            let grid_coords = uv_to_grid_space(frag_tex_coord);
            let body_radius = grid_coords.distance(UMBRAL_MASK_CENTER);
            let fade = 1.0 - {
                let outline_fade_radius = UMBRAL_MASK_OUTER_RADIUS - UMBRAL_MASK_FADE_BAND;
                let t = ((body_radius - outline_fade_radius) / (UMBRAL_MASK_OUTER_RADIUS - outline_fade_radius))
                    .clamp(0.0, 1.0);
                t * t * (3.0 - 2.0 * t)
            };
            let lum = (fade.clamp(0.0, 1.0) * 255.0) as u8;
            let idx = 4 * (y as usize * width as usize + x as usize);
            color[idx] = lum; // R
            color[idx + 1] = lum; // G
            color[idx + 2] = lum; // B
            color[idx + 3] = 255; // A
        }
    }
    img
}
