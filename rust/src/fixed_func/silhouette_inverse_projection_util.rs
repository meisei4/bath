use crate::render::raylib::RaylibRenderer;
use asset_payload::SPHERE_PATH;
use raylib::color::Color;
use raylib::consts::PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8;
use raylib::math::{Vector2, Vector3};
use raylib::models::{RaylibMesh, RaylibModel, WeakMesh};
use raylib::texture::{Image, Texture2D};
use std::f32::consts::TAU;
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

pub const ROTATION_FREQUENCY_HZ: f32 = 0.10;
pub const ANGULAR_VELOCITY: f32 = TAU * ROTATION_FREQUENCY_HZ;
pub const TIME_BETWEEN_SAMPLES: f32 = 0.5;
pub const ROTATIONAL_SAMPLES_FOR_INV_PROJ: usize = 40;

const TEXTURE_MAPPING_BOUNDARY_FADE: f32 = 0.1;

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
        let sample_x = vertex.x;
        let sample_y = vertex.y;
        let interpolated_radial_magnitude =
            interpolate_radial_magnitude_from_sample_xy(sample_x, sample_y, &radii_normals);
        vertex.x *= interpolated_radial_magnitude;
        vertex.y *= interpolated_radial_magnitude;
        vertex.z *= interpolated_radial_magnitude; //TODO: this is just to pull shit inwards
    }
}

pub fn generate_mesh_and_texcoord_samples_from_silhouette(
    screen_w: i32,
    screen_h: i32,
    renderer: &mut RaylibRenderer,
) -> (Vec<Vec<Vector3>>, Vec<Vec<f32>>) {
    let model = renderer.handle.load_model(&renderer.thread, SPHERE_PATH).unwrap();
    let vertices = model.meshes()[0].vertices();
    let mut mesh_samples = Vec::with_capacity(ROTATIONAL_SAMPLES_FOR_INV_PROJ);
    let mut texcoord_samples = Vec::with_capacity(ROTATIONAL_SAMPLES_FOR_INV_PROJ);
    for i in 0..ROTATIONAL_SAMPLES_FOR_INV_PROJ {
        let frame_time = i as f32 * TIME_BETWEEN_SAMPLES;
        let frame_rotation = -ANGULAR_VELOCITY * frame_time;
        let mut mesh_sample = vertices.to_vec();
        rotate_vertices(&mut mesh_sample, frame_rotation);
        let silhouette_img = generate_silhouette_image(screen_w, screen_h, frame_time);
        let radii_normals = build_silhouette_radii(&silhouette_img);
        deform_vertices_from_silhouette_radii(&mut mesh_sample, &radii_normals);

        let mut texcoord_sample = Vec::with_capacity(mesh_sample.len() * 2);
        for vertex in mesh_sample.clone() {
            let x = vertex.x;
            let y = vertex.y;
            let interpolated_radial_magnitude = interpolate_radial_magnitude_from_sample_xy(x, y, &radii_normals);
            let u = x / interpolated_radial_magnitude * 0.5 + 0.5;
            let v = y / interpolated_radial_magnitude * 0.5 + 0.5;
            texcoord_sample.push(u);
            texcoord_sample.push(v);
        }
        texcoord_samples.push(texcoord_sample);
        //TODO: why do i have to rotate back i forget....
        rotate_vertices(&mut mesh_sample, -frame_rotation);
        mesh_samples.push(mesh_sample);
    }
    (mesh_samples, texcoord_samples)
}

fn rotate_vertices(sphere_vertices_vector: &mut Vec<Vector3>, rotation: f32) {
    for vertex in sphere_vertices_vector {
        let (x0, z0) = (vertex.x, vertex.z);
        vertex.x = x0 * rotation.cos() + z0 * rotation.sin();
        vertex.z = -x0 * rotation.sin() + z0 * rotation.cos();
    }
}

pub fn lerp_intermediate_mesh_samples_to_single_mesh(
    i_time: f32,
    mesh_samples: &[Vec<Vector3>],
    texcoord_samples: &[Vec<f32>],
    target_mesh: &mut WeakMesh,
) {
    let duration = mesh_samples.len() as f32 * TIME_BETWEEN_SAMPLES;
    let time = i_time % duration;
    let frame = time / TIME_BETWEEN_SAMPLES;
    let current_frame = frame.floor() as usize % mesh_samples.len();
    let next_frame = (current_frame + 1) % mesh_samples.len();
    let weight = frame.fract();
    let vertices = target_mesh.vertices_mut();
    for ((dst_vertex, src_vertex), next_vertex) in vertices
        .iter_mut()
        .zip(mesh_samples[current_frame].iter())
        .zip(mesh_samples[next_frame].iter())
    {
        dst_vertex.x = src_vertex.x * (1.0 - weight) + next_vertex.x * weight;
        dst_vertex.y = src_vertex.y * (1.0 - weight) + next_vertex.y * weight;
        dst_vertex.z = src_vertex.z * (1.0 - weight) + next_vertex.z * weight;
    }
    let texcoords = unsafe { from_raw_parts_mut(target_mesh.as_mut().texcoords, target_mesh.vertices().len() * 2) };
    for ((dst_texcoord, src_texcoord), next_texcoord) in texcoords
        .iter_mut()
        .zip(texcoord_samples[current_frame].iter())
        .zip(texcoord_samples[next_frame].iter())
    {
        *dst_texcoord = *src_texcoord * (1.0 - weight) + *next_texcoord * weight;
    }
}

//TODO: THIS IS THE ISSUE
pub fn generate_silhouette_texture(
    render: &mut RaylibRenderer,
    screen_w: i32,
    screen_h: i32,
    texture_resolution: Vec<i32>,
    i_time: f32,
) -> Texture2D {
    let texture_w = texture_resolution[0];
    let texture_h = texture_resolution[1];
    let mut feathered_silhouette_img = Image::gen_image_color(texture_w, texture_h, Color::BLANK);
    let silhouette_img = generate_silhouette_image(screen_w, screen_h, i_time);
    let radii_normals = build_silhouette_radii(&silhouette_img);
    let total_bytes = (texture_w * texture_h * 4) as usize;
    let pixels: &mut [u8] = unsafe { from_raw_parts_mut(feathered_silhouette_img.data as *mut u8, total_bytes) };
    let center_coord_x = (texture_w as f32 - 1.0) * 0.5;
    let center_coord_y = (texture_h as f32 - 1.0) * 0.5;
    let max_radial_magnitude = radii_normals.iter().cloned().fold(0.0_f32, f32::max).max(1e-6);
    for y in 0..texture_h {
        for x in 0..texture_w {
            let sample_x = x as f32 - center_coord_x;
            let sample_y = y as f32 - center_coord_y;
            let sample_radius = (sample_x * sample_x + sample_y * sample_y).sqrt();
            let interpolated_radial_magnitude =
                interpolate_radial_magnitude_from_sample_xy(sample_x, sample_y, &radii_normals);
            let interpolated_radial_magnitude_normal = interpolated_radial_magnitude / max_radial_magnitude;
            //TODO: what the fuck is happening here
            let radius_outer = interpolated_radial_magnitude_normal + (sample_radius * 1.0) ;
            let radius_inner = radius_outer * (1.0 - TEXTURE_MAPPING_BOUNDARY_FADE);
            let alpha = if sample_radius <= radius_inner {
                1.0
            } else if sample_radius >= radius_outer {
                0.0
            } else {
                1.0 - (sample_radius - radius_inner) / (radius_outer - radius_inner)
            };
            let rgb_channel = (y * texture_w + x) as usize * 4;
            pixels[rgb_channel..rgb_channel + 4].copy_from_slice(&[255, 255, 255, (alpha * 255.0) as u8]);
        }
    }
    feathered_silhouette_img.set_format(PIXELFORMAT_UNCOMPRESSED_R8G8B8A8);
    let silhouette_texture = render
        .handle
        .load_texture_from_image(&render.thread, &feathered_silhouette_img)
        .unwrap();
    silhouette_texture
}

pub fn interpolate_radial_magnitude_from_sample_xy(sample_x: f32, sample_y: f32, radii_normals: &[f32]) -> f32 {
    let radial_disk_angle = sample_y.atan2(sample_x).rem_euclid(TAU);
    let radial_index = radial_disk_angle / TAU * SILHOUETTE_RADII_RESOLUTION as f32;
    let lower_index = radial_index.floor() as usize % SILHOUETTE_RADII_RESOLUTION;
    let upper_index = (lower_index + 1) % SILHOUETTE_RADII_RESOLUTION;
    let interpolation_toward_upper = radial_index.fract();
    radii_normals[lower_index] * (1.0 - interpolation_toward_upper)
        + radii_normals[upper_index] * interpolation_toward_upper
}
