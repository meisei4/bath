use asset_payload::payloads::BAYER_PNG;
use asset_payload::BAYER_PNG_PATH;
use bath::fixed_func::silhouette_constants::{
    CELL_DRIFT_AMPLITUDE, DITHER_BLEND_FACTOR, DITHER_TEXTURE_SCALE, LIGHT_WAVE_SPATIAL_FREQ_X,
    LIGHT_WAVE_SPATIAL_FREQ_Y, LIGHT_WAVE_TEMPORAL_FREQ_Y, UMBRAL_MASK_CENTER, UMBRAL_MASK_FADE_BAND,
    UMBRAL_MASK_INNER_RADIUS, UMBRAL_MASK_OFFSET_X, UMBRAL_MASK_OFFSET_Y, UMBRAL_MASK_OUTER_RADIUS,
    UMBRAL_MASK_PHASE_COEFFICIENT_X, UMBRAL_MASK_PHASE_COEFFICIENT_Y, UMBRAL_MASK_WAVE_AMPLITUDE_X,
    UMBRAL_MASK_WAVE_AMPLITUDE_Y,
};
use bath::fixed_func::silhouette_util::{add_phase, smoothstep, spatial_phase, temporal_phase, uv_to_grid_space};
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

#[inline]
pub fn warp_and_drift_cell(grid_coords: Vector2, time: f32) -> Vector2 {
    CELL_DRIFT_AMPLITUDE * Vector2::new((time + grid_coords.y).sin(), (time + grid_coords.x).sin())
}

#[inline]
pub fn light_radial_fade(grid_coords: Vector2, center: Vector2, radius: f32, feather: f32) -> f32 {
    let distance_from_center = grid_coords.distance(center);
    let fade_start = radius - feather;
    let alpha = 1.0 - smoothstep(fade_start, radius, distance_from_center);
    alpha.clamp(0.0, 1.0)
}

#[inline]
pub fn add_umbral_mask_phase(time: f32) -> Vector2 {
    Vector2::new(
        UMBRAL_MASK_WAVE_AMPLITUDE_X * LIGHT_WAVE_SPATIAL_FREQ_X,
        UMBRAL_MASK_WAVE_AMPLITUDE_Y * LIGHT_WAVE_SPATIAL_FREQ_Y + time * LIGHT_WAVE_TEMPORAL_FREQ_Y,
    )
}

#[inline]
pub fn umbral_mask_position(x_coeff: f32, y_coeff: f32, mask_phase: Vector2) -> Vector2 {
    Vector2::new(x_coeff * (mask_phase.x).cos(), y_coeff * (mask_phase.y).sin()) + UMBRAL_MASK_CENTER
}

#[inline]
pub fn add_umbral_mask(src_color: f32, grid_coords: Vector2, mask_center: Vector2) -> f32 {
    let mask_pos = mask_center + Vector2::new(UMBRAL_MASK_OFFSET_X, UMBRAL_MASK_OFFSET_Y);
    let dist = grid_coords.distance(mask_pos);
    let half_dist = dist * 0.5;
    let mask = smoothstep(UMBRAL_MASK_INNER_RADIUS, UMBRAL_MASK_OUTER_RADIUS, half_dist);
    src_color * mask
}

#[inline]
pub fn bayer_threshold(px: i32, py: i32, data: &[u8], w: i32, h: i32) -> f32 {
    let fx = (px as f32 / DITHER_TEXTURE_SCALE).fract();
    let fy = (py as f32 / DITHER_TEXTURE_SCALE).fract();
    let sx = (fx * w as f32).floor() as usize;
    let sy = (fy * h as f32).floor() as usize;
    data[sy * w as usize + sx] as f32 / 255.0
}

#[inline]
pub fn add_dither(src: f32, px: i32, py: i32, data: &[u8], w: i32, h: i32) -> f32 {
    let t = bayer_threshold(px, py, data, w, h);
    let b = if src >= t { 1.0 } else { 0.0 };
    (1.0 - DITHER_BLEND_FACTOR) * src + DITHER_BLEND_FACTOR * b
}

#[inline]
pub fn shade(
    px: i32,
    py: i32,
    i_resolution: Vector2,
    i_time: f32,
    bayer_data: &[u8],
    bayer_w: i32,
    bayer_h: i32,
) -> u8 {
    let frag_coord = Vector2::new(px as f32, py as f32);
    let frag_tex_coord = frag_coord / i_resolution;
    let mut grid_coords = uv_to_grid_space(frag_tex_coord);
    let mut grid_phase = spatial_phase(grid_coords);
    grid_phase += temporal_phase(i_time);
    grid_coords += add_phase(grid_phase);
    grid_coords += warp_and_drift_cell(grid_coords, i_time);
    let mut src_color = light_radial_fade(
        grid_coords,
        UMBRAL_MASK_CENTER,
        UMBRAL_MASK_OUTER_RADIUS,
        UMBRAL_MASK_FADE_BAND,
    );
    let umbral_mask_phase = add_umbral_mask_phase(i_time);
    let umbral_mask_pos = umbral_mask_position(
        UMBRAL_MASK_PHASE_COEFFICIENT_X,
        UMBRAL_MASK_PHASE_COEFFICIENT_Y,
        umbral_mask_phase,
    );
    src_color = add_umbral_mask(src_color, grid_coords, umbral_mask_pos);
    src_color = add_dither(src_color, px, py, bayer_data, bayer_w, bayer_h);
    (src_color * 255.0).round() as u8
}

pub fn load_bayer_png(path: &str) -> (Vec<u8>, i32, i32) {
    if let Ok(img) = Image::load_image(path) {
        let w = img.width;
        let h = img.height;
        let data: Vec<u8> = img.get_image_data().iter().map(|c| c.r).collect();
        (data, w, h)
    } else {
        (Vec::new(), 0, 0) //TODO: idk what to do here i think ill just fix this whole thing to fail like a shader would lol
    }
}
