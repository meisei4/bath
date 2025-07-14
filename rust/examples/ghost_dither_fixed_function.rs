use asset_payload::payloads::BAYER_PNG;
use asset_payload::BAYER_PNG_PATH;
use bath::fixed_func::ghost::{load_bayer_png, shade};
use bath::render::raylib::RaylibRenderer;
use bath::render::raylib_util::{flip_framebuffer, N64_HEIGHT, N64_WIDTH, ORIGIN};
use bath::render::renderer::Renderer;
use raylib::math::Vector2;
use raylib::texture::RaylibTexture2D;
use raylib::{color::Color, drawing::RaylibDraw, prelude::Image};

fn main() {
    let mut render = RaylibRenderer::init(N64_WIDTH, N64_HEIGHT);
    let width = render.handle.get_screen_width() as f32;
    let height = render.handle.get_screen_height() as f32;
    let i_resolution = Vector2::new(width, height);
    let img = Image::gen_image_color(i_resolution.x as i32, i_resolution.y as i32, Color::BLANK);
    let mut texture = render.handle.load_texture_from_image(&render.thread, &img).unwrap();
    let mut pixels = vec![0u8; (i_resolution.x * i_resolution.y * 4.0) as usize];
    let mut i_time = 0.0f32;

    let _i_channel0 = render.load_texture(BAYER_PNG(), ".png");
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

// pub const HALF: f32 = 0.5;
// pub const GRID_SCALE: f32 = 4.0;
// pub const GRID_CELL_SIZE: f32 = 1.0 / GRID_SCALE;
// pub const GRID_ORIGIN_INDEX: Vector2 = Vector2::new(0.0, 0.0);
// pub const GRID_ORIGIN_OFFSET_CELLS: Vector2 = Vector2::new(2.0, 2.0);
// pub const GRID_ORIGIN_UV_OFFSET: Vector2 = Vector2::new(
//     (GRID_ORIGIN_INDEX.x + GRID_ORIGIN_OFFSET_CELLS.x) * GRID_CELL_SIZE,
//     (GRID_ORIGIN_INDEX.y + GRID_ORIGIN_OFFSET_CELLS.y) * GRID_CELL_SIZE,
// );
// pub const CELL_DRIFT_AMPLITUDE: f32 = 0.2;
// pub const LIGHT_WAVE_SPATIAL_FREQ_X: f32 = 8.0;
// pub const LIGHT_WAVE_SPATIAL_FREQ_Y: f32 = 8.0;
// pub const LIGHT_WAVE_TEMPORAL_FREQ_X: f32 = 80.0;
// pub const LIGHT_WAVE_TEMPORAL_FREQ_Y: f32 = 2.3;
// pub const LIGHT_WAVE_AMPLITUDE_X: f32 = 0.03;
// pub const LIGHT_WAVE_AMPLITUDE_Y: f32 = 0.1;
// pub const UMBRAL_MASK_OUTER_RADIUS: f32 = 0.40;
// pub const UMBRAL_MASK_INNER_RADIUS: f32 = 0.08;
// pub const UMBRAL_MASK_FADE_BAND: f32 = 0.025;
// pub const UMBRAL_MASK_CENTER: Vector2 = Vector2::new(HALF, HALF);
// pub const DITHER_TEXTURE_SCALE: f32 = 8.0;
// pub const DITHER_BLEND_FACTOR: f32 = 0.75;
//
// #[inline]
// pub fn uv_to_grid_space(uv: Vector2) -> Vector2 {
//     (uv - GRID_ORIGIN_UV_OFFSET) * GRID_SCALE
// }
//
// #[inline]
// pub fn spatial_phase(grid_coords: Vector2) -> Vector2 {
//     Vector2::new(
//         grid_coords.y * LIGHT_WAVE_SPATIAL_FREQ_X,
//         grid_coords.x * LIGHT_WAVE_SPATIAL_FREQ_Y,
//     )
// }
//
// #[inline]
// pub fn temporal_phase(time: f32) -> Vector2 {
//     Vector2::new(time * LIGHT_WAVE_TEMPORAL_FREQ_X, time * LIGHT_WAVE_TEMPORAL_FREQ_Y)
// }
//
// #[inline]
// pub fn add_phase(phase: Vector2) -> Vector2 {
//     Vector2::new(
//         LIGHT_WAVE_AMPLITUDE_X * (phase.x).cos(),
//         LIGHT_WAVE_AMPLITUDE_Y * (phase.y).sin(),
//     )
// }
//
// #[inline]
// pub fn shade(
//     px: i32,
//     py: i32,
//     i_resolution: Vector2,
//     i_time: f32,
//     bayer_data: &[u8],
//     bayer_w: i32,
//     bayer_h: i32,
// ) -> u8 {
//     let frag_coord = Vector2::new(px as f32, py as f32);
//     let frag_tex_coord = frag_coord / i_resolution;
//     let mut grid_coords = uv_to_grid_space(frag_tex_coord);
//     let mut grid_phase = spatial_phase(grid_coords);
//     grid_phase += temporal_phase(i_time);
//     grid_coords += add_phase(grid_phase);
//     let mut src_color = light_radial_fade(
//         grid_coords,
//         UMBRAL_MASK_CENTER,
//         UMBRAL_MASK_OUTER_RADIUS,
//         UMBRAL_MASK_FADE_BAND,
//     );
//
//     src_color = add_dither(src_color, px, py, bayer_data, bayer_w, bayer_h);
//     (src_color * 255.0).round() as u8
// }
