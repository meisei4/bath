use asset_payload::payloads::BAYER_PNG;
use asset_payload::BAYER_PNG_PATH;
use bath::fixed_func::silhouette::{load_bayer_png, shade};
use bath::render::raylib::RaylibRenderer;
use bath::render::raylib_util::{flip_framebuffer, N64_WIDTH, ORIGIN};
use bath::render::renderer::Renderer;
use raylib::math::Vector2;
use raylib::texture::RaylibTexture2D;
use raylib::{color::Color, drawing::RaylibDraw, prelude::Image};

fn main() {
    let mut render = RaylibRenderer::init(N64_WIDTH, N64_WIDTH);
    let width = render.handle.get_screen_width() as f32;
    let height = render.handle.get_screen_height() as f32;
    let i_resolution = Vector2::new(width, height);
    let img = Image::gen_image_color(i_resolution.x as i32, i_resolution.y as i32, Color::BLANK);
    let mut texture = render.handle.load_texture_from_image(&render.thread, &img).unwrap();
    let mut pixels = vec![0u8; (i_resolution.x * i_resolution.y * 4.0) as usize];
    let mut i_time = 0.0f32;
    let i_channel0 = render.load_texture(BAYER_PNG(), ".png");
    let (bayer_data, bayer_w, bayer_h) = load_bayer_png(BAYER_PNG_PATH);
    while !render.handle.window_should_close() {
        i_time += render.handle.get_frame_time();
        for y in 0..i_resolution.y as i32 {
            for x in 0..i_resolution.x as i32 {
                let lum = shade(x, y, i_resolution, i_time, &bayer_data, bayer_w, bayer_h);
                let idx = 4 * (y as usize * i_resolution.x as usize + x as usize);
                pixels[idx] = lum;
                pixels[idx + 1] = lum;
                pixels[idx + 2] = lum;
                pixels[idx + 3] = 255;
            }
        }
        texture.update_texture(&pixels).unwrap();
        let mut draw_handle = render.handle.begin_drawing(&render.thread);
        draw_handle.clear_background(Color::BLACK);
        draw_handle.draw_texture_rec(&texture, flip_framebuffer(width, height), ORIGIN, Color::WHITE);
    }
}
