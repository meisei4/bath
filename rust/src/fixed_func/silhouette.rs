use raylib::color::Color;
use raylib::math::{Vector2, Vector3};
use raylib::models::{RaylibMesh, WeakMesh};
use raylib::prelude::Image;
use std::f32::consts::TAU;
use std::ptr::copy_nonoverlapping;
use std::slice::{from_raw_parts, from_raw_parts_mut};

pub const CELL_DRIFT_AMPLITUDE: f32 = 0.2;
pub const UMBRAL_MASK_INNER_RADIUS: f32 = 0.08;
pub const UMBRAL_MASK_OFFSET_X: f32 = -UMBRAL_MASK_OUTER_RADIUS / 1.0;
pub const UMBRAL_MASK_OFFSET_Y: f32 = -UMBRAL_MASK_OUTER_RADIUS;
pub const UMBRAL_MASK_PHASE_COEFFICIENT_X: f32 = 0.6;
pub const UMBRAL_MASK_PHASE_COEFFICIENT_Y: f32 = 0.2;
pub const UMBRAL_MASK_WAVE_AMPLITUDE_X: f32 = 0.1;
pub const UMBRAL_MASK_WAVE_AMPLITUDE_Y: f32 = 0.1;

pub const DITHER_TEXTURE_SCALE: f32 = 8.0;
pub const DITHER_BLEND_FACTOR: f32 = 0.75;

const HALF: f32 = 0.5;
const GRID_SCALE: f32 = 4.0;
const GRID_CELL_SIZE: f32 = 1.0 / GRID_SCALE;
const GRID_ORIGIN_INDEX: Vector2 = Vector2::new(0.0, 0.0);
const GRID_ORIGIN_OFFSET_CELLS: Vector2 = Vector2::new(2.0, 2.0);
const GRID_ORIGIN_UV_OFFSET: Vector2 = Vector2::new(
    (GRID_ORIGIN_INDEX.x + GRID_ORIGIN_OFFSET_CELLS.x) * GRID_CELL_SIZE,
    (GRID_ORIGIN_INDEX.y + GRID_ORIGIN_OFFSET_CELLS.y) * GRID_CELL_SIZE,
);

const LIGHT_WAVE_SPATIAL_FREQ_X: f32 = 8.0;
const LIGHT_WAVE_SPATIAL_FREQ_Y: f32 = 8.0;
const LIGHT_WAVE_TEMPORAL_FREQ_X: f32 = 80.0;
const LIGHT_WAVE_TEMPORAL_FREQ_Y: f32 = 2.3;
const LIGHT_WAVE_AMPLITUDE_X: f32 = 0.0;
const LIGHT_WAVE_AMPLITUDE_Y: f32 = 0.1;
const UMBRAL_MASK_OUTER_RADIUS: f32 = 0.40;
const UMBRAL_MASK_FADE_BAND: f32 = 0.025;
const UMBRAL_MASK_CENTER: Vector2 = Vector2::new(HALF, HALF);

pub const ROTATION_FREQUENCY_HZ: f32 = 0.25;
pub const ANGULAR_VELOCITY: f32 = TAU * ROTATION_FREQUENCY_HZ;
pub const MODEL_POS: Vector3 = Vector3::ZERO;
pub const MODEL_SCALE: Vector3 = Vector3::ONE;

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
pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[inline]
pub fn generate_silhouette_image(width: i32, height: i32, i_time: f32) -> Image {
    let img = Image::gen_image_color(width, height, Color::BLANK);
    let total_bytes = (width * height * 4) as usize;
    let pixels: &mut [u8] = unsafe { from_raw_parts_mut(img.data as *mut u8, total_bytes) };
    for y in 0..height {
        for x in 0..width {
            let s = (x as f32 + 0.5) / width as f32;
            let t = (y as f32 + 0.5) / height as f32;
            let uv = Vector2::new(s, t);
            let centre_offset = GRID_ORIGIN_UV_OFFSET - Vector2::splat(0.5 / GRID_SCALE);
            let mut grid_coords = (uv - centre_offset) * GRID_SCALE;
            let mut grid_phase = Vector2::ZERO;
            grid_phase += spatial_phase(grid_coords);
            grid_phase += temporal_phase(i_time);
            grid_coords += add_phase(grid_phase);
            let distance_from_center = grid_coords.distance(UMBRAL_MASK_CENTER);
            let fade_start = UMBRAL_MASK_OUTER_RADIUS - UMBRAL_MASK_FADE_BAND;
            let alpha = 1.0 - smoothstep(fade_start, UMBRAL_MASK_OUTER_RADIUS, distance_from_center);
            let lum = alpha.clamp(0.0, 1.0);
            let lum_u8 = (lum * 255.0) as u8;
            let idx = 4 * (y as usize * width as usize + x as usize);
            pixels[idx] = lum_u8; // R
            pixels[idx + 1] = lum_u8; // G
            pixels[idx + 2] = lum_u8; // B
            pixels[idx + 3] = 255u8; // A
        }
    }
    img
}

pub const RADIAL_SAMPLE_COUNT: usize = 64;
pub fn build_radial_magnitudes(source_image: &Image) -> Vec<f32> {
    let image_width_in_pixels = source_image.width();
    let image_height_in_pixels = source_image.height();
    let centre_coordinate_x = (image_width_in_pixels as f32 - 1.0) * 0.5;
    let centre_coordinate_y = (image_height_in_pixels as f32 - 1.0) * 0.5;
    let total_bytes = (image_width_in_pixels * image_height_in_pixels * 4) as usize;
    let pixel_bytes: &[u8] = unsafe { from_raw_parts(source_image.data as *const u8, total_bytes) };
    let reference_pixel_radius = UMBRAL_MASK_OUTER_RADIUS * image_width_in_pixels.min(image_height_in_pixels) as f32;
    let mut normalised_radial_magnitudes = vec![0.0_f32; RADIAL_SAMPLE_COUNT];
    for sample_angle_index in 0..RADIAL_SAMPLE_COUNT {
        let angle_theta_radians = (sample_angle_index as f32) * TAU / RADIAL_SAMPLE_COUNT as f32;
        let direction_unit_x = angle_theta_radians.cos();
        let direction_unit_y = angle_theta_radians.sin();
        let mut current_step_in_pixels = 0_i32;
        loop {
            let sample_x = centre_coordinate_x + direction_unit_x * current_step_in_pixels as f32;
            let sample_y = centre_coordinate_y + direction_unit_y * current_step_in_pixels as f32;
            let texel_x = sample_x as i32 as usize;
            let texel_y = sample_y as i32 as usize;
            let pixel_index = 4 * (texel_y * image_width_in_pixels as usize + texel_x);
            let luminance_value = pixel_bytes[pixel_index];
            if luminance_value == 0 {
                break;
            }
            current_step_in_pixels += 1;
        }
        let boundary_distance_in_pixels = current_step_in_pixels as f32;
        let normalised_distance = boundary_distance_in_pixels / reference_pixel_radius;
        normalised_radial_magnitudes[sample_angle_index] = normalised_distance.max(0.0);
    }
    normalised_radial_magnitudes
}

pub fn deform_mesh_by_silhouette_radii(mesh: &mut WeakMesh, radial_magnitudes: &[f32]) {
    let vertices: &mut [Vector3] = mesh.vertices_mut();
    for vertex in vertices.iter_mut() {
        let theta = vertex.y.atan2(vertex.x).rem_euclid(TAU);
        let idx_f = theta / TAU * RADIAL_SAMPLE_COUNT as f32;
        let i0 = idx_f.floor() as usize % RADIAL_SAMPLE_COUNT;
        let i1 = (i0 + 1) % RADIAL_SAMPLE_COUNT;
        let w_hi = idx_f.fract();
        let w_lo = 1.0 - w_hi;
        let r_equator = radial_magnitudes[i0] * w_lo + radial_magnitudes[i1] * w_hi;
        vertex.x *= r_equator;
        vertex.y *= r_equator;
        vertex.z *= r_equator;
    }
    unsafe { mesh.upload(false) };
}

#[inline]
pub fn uv_to_grid_space(uv: Vector2) -> Vector2 {
    (uv - GRID_ORIGIN_UV_OFFSET) * GRID_SCALE
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

pub fn map_mesh_vertices_to_silhouette_texcoords(
    vertices: &[Vector3],
    radial_sampling_angle: f32,
    radial_magnitudes: &[f32],
) -> Vec<f32> {
    let mut silhouette_texcoords = Vec::with_capacity(vertices.len() * 2);
    for vertex in vertices {
        let sample_x = radial_sampling_angle.cos() * vertex.x + radial_sampling_angle.sin() * vertex.z;
        let sample_y = vertex.y;

        let polar_theta = sample_y.atan2(sample_x).rem_euclid(TAU);
        let radial_index = polar_theta / TAU * RADIAL_SAMPLE_COUNT as f32;
        let lower_sample_index = radial_index.floor() as usize % RADIAL_SAMPLE_COUNT;
        let upper_sample_index = (lower_sample_index + 1) % RADIAL_SAMPLE_COUNT;
        let interpolation_toward_upper = radial_index.fract();
        let lerp_radial_index = radial_magnitudes[lower_sample_index] * (1.0 - interpolation_toward_upper)
            + radial_magnitudes[upper_sample_index] * interpolation_toward_upper;
        let error_margin = lerp_radial_index.max(1e-6);
        let u = sample_x / error_margin * 0.5 + 0.5;
        let v = sample_y / error_margin * 0.5 + 0.5;
        silhouette_texcoords.push(u);
        silhouette_texcoords.push(v);
    }
    silhouette_texcoords
}

pub unsafe fn update_texcoords(mesh: &mut WeakMesh, texcoords: &[f32]) {
    assert!(
        !mesh.as_ref().texcoords.is_null(),
        "IAN! update_texcoords: mesh has no existing texcoord buffer (NULL)"
    );
    let vertex_count = mesh.vertexCount as usize;
    let expected_len = vertex_count * 2;
    assert_eq!(
        texcoords.len(),
        expected_len,
        "IAN! update_texcoords: length mismatch (got {}, expected {})",
        texcoords.len(),
        expected_len
    );
    assert_eq!(
        (mesh.as_ref().texcoords as usize) % align_of::<f32>(),
        0,
        "IAN! update_texcoords: texcoord pointer alignment invalid"
    );
    copy_nonoverlapping(texcoords.as_ptr(), mesh.texcoords, texcoords.len());
    mesh.upload(false);
}
