use bath::render::raylib::RaylibRenderer;
use bath::render::raylib_util::{N64_HEIGHT, N64_WIDTH};
use bath::render::renderer::Renderer;
use raylib::color::Color;
use raylib::texture::Image;

fn main() {
    let mut render = RaylibRenderer::init(N64_WIDTH, N64_HEIGHT);
    let size = 64;
    let mut img = Image::gen_image_color(size, size, Color::WHITE);
    let half = size as f32 * 0.5;
    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 - half;
            let dy = y as f32 - half;
            let dist = ((dx * dx + dy * dy).sqrt()) / half;
            let alpha = (1.0 - dist.clamp(0.0, 1.0)).max(0.0);
            let v = (alpha * 255.0) as u8;
            img.draw_pixel(x, y, Color::new(v, v, v, v));
        }
    }
    let mut texture = render
        .handle
        .load_texture_from_image(&render.thread, &img)
        .expect("upload radial texture");
    render.tweak_texture_parameters(&mut texture, true, true);
    while !render.handle.window_should_close() {
        render.draw_fixedfunc_screen_pseudo_ortho_geom(&texture);
    }
}
