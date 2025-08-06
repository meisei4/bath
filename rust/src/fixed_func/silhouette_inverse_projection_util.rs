use crate::render::raylib::RaylibRenderer;
use asset_payload::SPHERE_PATH;
use raylib::color::Color;
use raylib::consts::PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8;
use raylib::math::{Vector2, Vector3};
use raylib::models::{RaylibMesh, RaylibModel, WeakMesh};
use raylib::prelude::Image;
use std::f32::consts::TAU;
use std::ptr::copy_nonoverlapping;
use std::slice::{from_raw_parts, from_raw_parts_mut};

const HALF: f32 = 0.5;
const GRID_SCALE: f32 = 4.0;
const GRID_CELL_SIZE: f32 = 1.0 / GRID_SCALE;
const GRID_ORIGIN_INDEX: Vector2 = Vector2::new(0.0, 0.0);
const GRID_ORIGIN_OFFSET_CELLS: Vector2 = Vector2::new(2.0, 2.0);
const GRID_ORIGIN_UV_OFFSET: Vector2 = Vector2::new(
    (GRID_ORIGIN_INDEX.x + GRID_ORIGIN_OFFSET_CELLS.x) * GRID_CELL_SIZE,
    (GRID_ORIGIN_INDEX.y + GRID_ORIGIN_OFFSET_CELLS.y) * GRID_CELL_SIZE,
);

pub const LIGHT_WAVE_SPATIAL_FREQ_X: f32 = 8.0;
pub const LIGHT_WAVE_SPATIAL_FREQ_Y: f32 = 8.0;
pub const LIGHT_WAVE_TEMPORAL_FREQ_X: f32 = 80.0;
pub const LIGHT_WAVE_TEMPORAL_FREQ_Y: f32 = 2.3;
pub const LIGHT_WAVE_AMPLITUDE_X: f32 = 0.0;
pub const LIGHT_WAVE_AMPLITUDE_Y: f32 = 0.1;
pub const UMBRAL_MASK_OUTER_RADIUS: f32 = 0.40;
pub const UMBRAL_MASK_FADE_BAND: f32 = 0.025;
pub const UMBRAL_MASK_CENTER: Vector2 = Vector2::new(HALF, HALF);

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

pub const SILHOUETTE_RADII_RESOLUTION: usize = 64;

pub const ROTATION_FREQUENCY_HZ: f32 = 0.25;
pub const ANGULAR_VELOCITY: f32 = TAU * ROTATION_FREQUENCY_HZ;
pub const TIME_BETWEEN_SAMPLES: f32 = 0.5;
pub const ROTATIONAL_SAMPLES_FOR_INV_PROJ: usize = 40;

const TEXTURE_MAPPING_BOUNDARY_FADE: f32 = 0.2;

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

#[inline]
pub fn generate_silhouette_image(width: i32, height: i32, i_time: f32) -> Image {
    let silhouette_img = Image::gen_image_color(width, height, Color::BLANK);
    let total_bytes = (width * height * 4) as usize;
    let pixels: &mut [u8] = unsafe { from_raw_parts_mut(silhouette_img.data as *mut u8, total_bytes) };
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
    silhouette_img
}

pub fn build_silhouette_radii(silhouette_img: &Image) -> Vec<f32> {
    let center_coord_x = (silhouette_img.width as f32 - 1.0) * 0.5;
    let center_coord_y = (silhouette_img.height as f32 - 1.0) * 0.5;
    let total_bytes = (silhouette_img.width * silhouette_img.height * 4) as usize;
    let pixel_bytes: &[u8] = unsafe { from_raw_parts(silhouette_img.data as *const u8, total_bytes) };
    let dummy_min_radius = UMBRAL_MASK_OUTER_RADIUS * silhouette_img.width.min(silhouette_img.height) as f32;
    let mut radii_normals = vec![0.0_f32; SILHOUETTE_RADII_RESOLUTION];
    for i in 0..SILHOUETTE_RADII_RESOLUTION {
        let theta = (i as f32) * TAU / SILHOUETTE_RADII_RESOLUTION as f32;
        let p_x = theta.cos();
        let p_y = theta.sin();
        let mut step = 0_i32;
        loop {
            let sample_x = center_coord_x + p_x * step as f32;
            let sample_y = center_coord_y + p_y * step as f32;
            let texel_x = sample_x as usize;
            let texel_y = sample_y as usize;
            let pixel_i = 4 * (texel_y * silhouette_img.width as usize + texel_x);
            let lum = pixel_bytes[pixel_i];
            if lum == 0 {
                break;
            }
            step += 1;
        }
        let normalized_radius_sample = step as f32 / dummy_min_radius;
        radii_normals[i] = normalized_radius_sample.max(0.0);
    }
    radii_normals
}

pub fn deform_vertices_from_silhouette_radii(vertices: &mut [Vector3], radii_normals: &[f32]) {
    for vertex in vertices {
        let angle = vertex.y.atan2(vertex.x).rem_euclid(TAU);
        let sample_position = angle / TAU * SILHOUETTE_RADII_RESOLUTION as f32;
        let lower_i = sample_position.floor() as usize % SILHOUETTE_RADII_RESOLUTION;
        let upper_i = (lower_i + 1) % SILHOUETTE_RADII_RESOLUTION;
        let upper_weight = sample_position.fract();
        let lower_weight = 1.0 - upper_weight;
        let radial_mag = radii_normals[lower_i] * lower_weight + radii_normals[upper_i] * upper_weight;
        vertex.x *= radial_mag;
        vertex.y *= radial_mag;
        vertex.z *= radial_mag;
    }
}

pub fn generate_inverse_projection_samples_from_silhouette(
    screen_w: i32,
    screen_h: i32,
    renderer: &mut RaylibRenderer,
) -> Vec<Vec<Vector3>> {
    let sphere_model = renderer.handle.load_model(&renderer.thread, SPHERE_PATH).unwrap();
    let sphere_vertices = sphere_model.meshes()[0].vertices();
    let mut per_frame_vertex_samples = Vec::with_capacity(ROTATIONAL_SAMPLES_FOR_INV_PROJ);
    for i in 0..ROTATIONAL_SAMPLES_FOR_INV_PROJ {
        let frame_time = i as f32 * TIME_BETWEEN_SAMPLES;
        let frame_rotation = -frame_time * ANGULAR_VELOCITY;
        let mut sphere_vertices_vector = sphere_vertices.to_vec();
        rotate_vertices(&mut sphere_vertices_vector, frame_rotation);
        let silhouette_img = generate_silhouette_image(screen_w, screen_h, frame_time);
        let radii_normals = build_silhouette_radii(&silhouette_img);
        deform_vertices_from_silhouette_radii(&mut sphere_vertices_vector, &radii_normals);
        //TODO: why do i have to rotate back i forget....
        rotate_vertices(&mut sphere_vertices_vector, -frame_rotation);
        per_frame_vertex_samples.push(sphere_vertices_vector);
    }
    per_frame_vertex_samples
}

fn rotate_vertices(sphere_vertices_vector: &mut Vec<Vector3>, rotation: f32) {
    for vertex in sphere_vertices_vector {
        let (x0, z0) = (vertex.x, vertex.z);
        vertex.x = x0 * rotation.cos() + z0 * rotation.sin();
        vertex.z = -x0 * rotation.sin() + z0 * rotation.cos();
    }
}

pub fn update_mesh_with_vertex_sample_interpolation(
    i_time: f32,
    per_frame_vertex_samples: &[Vec<Vector3>],
    mesh: &mut WeakMesh,
) {
    let duration = per_frame_vertex_samples.len() as f32 * TIME_BETWEEN_SAMPLES;
    let time = i_time % duration;
    let frame = time / TIME_BETWEEN_SAMPLES;
    let current_frame = frame.floor() as usize % per_frame_vertex_samples.len();
    let next_frame = (current_frame + 1) % per_frame_vertex_samples.len();
    let weight = frame.fract();
    let vertices = mesh.vertices_mut();
    //TODO: this is batshit, make it easier, i have no idea how iterators and zipping iterators would even work...
    for ((dst_vertex, src_vertex), next_vertex) in vertices
        .iter_mut()
        .zip(per_frame_vertex_samples[current_frame].iter())
        .zip(per_frame_vertex_samples[next_frame].iter())
    {
        dst_vertex.x = src_vertex.x * (1.0 - weight) + next_vertex.x * weight;
        dst_vertex.y = src_vertex.y * (1.0 - weight) + next_vertex.y * weight;
        dst_vertex.z = src_vertex.z * (1.0 - weight) + next_vertex.z * weight;
    }
}

pub fn make_radial_gradient_face(texture_width_height: i32, mags: &[f32]) -> Image {
    let mut img = Image::gen_image_color(texture_width_height, texture_width_height, Color::BLANK);
    let data = unsafe {
        from_raw_parts_mut(
            img.data as *mut u8,
            (texture_width_height * texture_width_height * 4) as usize,
        )
    };
    let c = (texture_width_height - 1) as f32 * 0.5;
    let mags_max = mags.iter().cloned().fold(0.0_f32, f32::max).max(1e-6);
    for y in 0..texture_width_height {
        for x in 0..texture_width_height {
            let dx = x as f32 - c;
            let dy = y as f32 - c;
            let r = (dx * dx + dy * dy).sqrt();
            let theta = dy.atan2(dx).rem_euclid(TAU);
            let f = theta / TAU * SILHOUETTE_RADII_RESOLUTION as f32;
            let i0 = f.floor() as usize % SILHOUETTE_RADII_RESOLUTION;
            let i1 = (i0 + 1) % SILHOUETTE_RADII_RESOLUTION;
            let w_hi = f.fract();
            let w_lo = 1.0 - w_hi;
            let edge_norm = (mags[i0] * w_lo + mags[i1] * w_hi) / mags_max; // 0‥1
            let radius_outer = edge_norm * c;
            if radius_outer <= 0.0 {
                continue;
            }
            let radius_inner = radius_outer * (1.0 - TEXTURE_MAPPING_BOUNDARY_FADE);
            let alpha = if r <= radius_inner {
                1.0
            } else if r >= radius_outer {
                0.0
            } else {
                1.0 - (r - radius_inner) / (radius_outer - radius_inner)
            };

            let px = (y * texture_width_height + x) as usize * 4;
            let a = (alpha * 255.0) as u8;
            data[px..px + 4].copy_from_slice(&[255, 255, 255, a]);
        }
    }
    img.set_format(PIXELFORMAT_UNCOMPRESSED_R8G8B8A8);
    img
}

//TODO: this is for tomorrow, this needs to recognize the interpolation stuff not just the first frame and whatever,
// Start with just simple getting all samples of the textures of each mesh that makes up the final single mesh
pub fn map_mesh_vertices_to_silhouette_texcoords(
    vertices: &[Vector3],
    sample_theta: f32,
    radii_normals: &[f32],
) -> Vec<f32> {
    let mut silhouette_texcoords = Vec::with_capacity(vertices.len() * 2);
    for vertex in vertices {
        let sample_x = sample_theta.cos() * vertex.x + sample_theta.sin() * vertex.z;
        let sample_y = vertex.y;

        let polar_theta = sample_y.atan2(sample_x).rem_euclid(TAU);
        let radial_index = polar_theta / TAU * SILHOUETTE_RADII_RESOLUTION as f32;
        let lower_sample_index = radial_index.floor() as usize % SILHOUETTE_RADII_RESOLUTION;
        let upper_sample_index = (lower_sample_index + 1) % SILHOUETTE_RADII_RESOLUTION;
        let interpolation_toward_upper = radial_index.fract();
        let lerp_radial_index = radii_normals[lower_sample_index] * (1.0 - interpolation_toward_upper)
            + radii_normals[upper_sample_index] * interpolation_toward_upper;
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
}
