use raylib::math::Vector2;
use raylib::prelude::Image;

// cargo run --example ghost_dither_fixed_function --features tests-only
// cargo run --example ghost_dither --features tests-only,glsl-100

pub const HALF: f32 = 0.5;
pub const GRID_SCALE: f32 = 4.0;
pub const GRID_CELL_SIZE: f32 = 1.0 / GRID_SCALE;
pub const GRID_ORIGIN_INDEX: Vector2 = Vector2::new(0.0, 0.0);
pub const GRID_ORIGIN_OFFSET_CELLS: Vector2 = Vector2::new(2.0, 2.0);
pub const GRID_ORIGIN_UV_OFFSET: Vector2 = Vector2::new(
    (GRID_ORIGIN_INDEX.x + GRID_ORIGIN_OFFSET_CELLS.x) * GRID_CELL_SIZE,
    (GRID_ORIGIN_INDEX.y + GRID_ORIGIN_OFFSET_CELLS.y) * GRID_CELL_SIZE,
);
pub const CELL_DRIFT_AMPLITUDE: f32 = 0.2;
pub const LIGHT_WAVE_SPATIAL_FREQ_X: f32 = 8.0;
pub const LIGHT_WAVE_SPATIAL_FREQ_Y: f32 = 8.0;
pub const LIGHT_WAVE_TEMPORAL_FREQ_X: f32 = 80.0;
pub const LIGHT_WAVE_TEMPORAL_FREQ_Y: f32 = 2.3;
pub const LIGHT_WAVE_AMPLITUDE_X: f32 = 0.0;
pub const LIGHT_WAVE_AMPLITUDE_Y: f32 = 0.1;
pub const UMBRAL_MASK_OUTER_RADIUS: f32 = 0.40;
pub const UMBRAL_MASK_INNER_RADIUS: f32 = 0.08;
pub const UMBRAL_MASK_FADE_BAND: f32 = 0.025;
pub const UMBRAL_MASK_CENTER: Vector2 = Vector2::new(HALF, HALF);
pub const UMBRAL_MASK_OFFSET_X: f32 = -UMBRAL_MASK_OUTER_RADIUS / 1.0;
pub const UMBRAL_MASK_OFFSET_Y: f32 = -UMBRAL_MASK_OUTER_RADIUS;
pub const UMBRAL_MASK_PHASE_COEFFICIENT_X: f32 = 0.6;
pub const UMBRAL_MASK_PHASE_COEFFICIENT_Y: f32 = 0.2;
pub const UMBRAL_MASK_WAVE_AMPLITUDE_X: f32 = 0.1;
pub const UMBRAL_MASK_WAVE_AMPLITUDE_Y: f32 = 0.1;

pub const DITHER_TEXTURE_SCALE: f32 = 8.0;
pub const DITHER_BLEND_FACTOR: f32 = 0.75;

#[inline]
pub fn uv_to_grid_space(uv: Vector2) -> Vector2 {
    (uv - GRID_ORIGIN_UV_OFFSET) * GRID_SCALE
}

#[inline]
pub fn warp_and_drift_cell(grid_coords: Vector2, time: f32) -> Vector2 {
    CELL_DRIFT_AMPLITUDE * Vector2::new((time + grid_coords.y).sin(), (time + grid_coords.x).sin())
}

#[inline]
pub fn spatial_phase(grid_coords: Vector2) -> Vector2 {
    Vector2::new(
        grid_coords.y * LIGHT_WAVE_SPATIAL_FREQ_X,
        grid_coords.x * LIGHT_WAVE_SPATIAL_FREQ_Y,
    )
}

#[inline]
pub fn temporal_phase(time: f32) -> Vector2 {
    Vector2::new(time * LIGHT_WAVE_TEMPORAL_FREQ_X, time * LIGHT_WAVE_TEMPORAL_FREQ_Y)
}

#[inline]
pub fn add_phase(phase: Vector2) -> Vector2 {
    Vector2::new(
        LIGHT_WAVE_AMPLITUDE_X * (phase.x).cos(),
        LIGHT_WAVE_AMPLITUDE_Y * (phase.y).sin(),
    )
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
    let half_dist = dist * HALF;
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
    //grid_coords += warp_and_drift_cell(grid_coords, i_time);
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
    //src_color = add_umbral_mask(src_color, grid_coords, umbral_mask_pos);
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

#[inline]
pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
