use crate::render::raylib::RaylibRenderer;
use asset_payload::SPHERE_PATH;
use raylib::color::Color;
use raylib::consts::PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8;
use raylib::math::glam::{Vec2, Vec3};
use raylib::math::{Vector2, Vector3};
use raylib::models::{Model, RaylibMesh, RaylibModel};
use raylib::texture::{Image, Texture2D};
use std::f32::consts::PI;
use std::f32::consts::TAU;
use std::slice::from_raw_parts_mut;

pub const MODEL_POS: Vector3 = Vector3::ZERO;
pub const MODEL_SCALE: Vector3 = Vector3::ONE;

pub const HALF: f32 = 0.5;
pub const GRID_SCALE: f32 = 4.0;
pub const GRID_CELL_SIZE: f32 = 1.0 / GRID_SCALE;
pub const GRID_ORIGIN_INDEX: Vector2 = Vector2::new(0.0, 0.0);
pub const GRID_ORIGIN_OFFSET_CELLS: Vector2 = Vector2::new(2.0, 2.0);
pub const GRID_ORIGIN_UV_OFFSET: Vector2 = Vector2::new(
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

pub const TEXTURE_MAPPING_BOUNDARY_FADE: f32 = 0.075;
pub const SILHOUETTE_TEXTURE_RES: i32 = 256;

pub const TWO_PI: f32 = 2.0 * PI;

pub fn generate_mesh_and_texcoord_samples_from_silhouette(
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
        let radii_normals = build_silhouette_radii_fast(frame_time);
        deform_vertices_from_silhouette_radii(&mut mesh_sample, &radii_normals);
        let mut texcoord_sample = Vec::with_capacity(mesh_sample.len() * 2);
        for vertex in mesh_sample.clone() {
            let (u, v) = silhouette_uvs_polar(vertex.x, vertex.y, &radii_normals);
            texcoord_sample.push(u);
            texcoord_sample.push(v);
        }
        texcoord_samples.push(texcoord_sample);
        rotate_vertices(&mut mesh_sample, -frame_rotation);
        mesh_samples.push(mesh_sample);
    }
    (mesh_samples, texcoord_samples)
}

pub fn build_silhouette_radii_fast(time: f32) -> Vec<f32> {
    let mut radii = Vec::with_capacity(SILHOUETTE_RADII_RESOLUTION);
    for i in 0..SILHOUETTE_RADII_RESOLUTION {
        let theta = (i as f32) * TAU / (SILHOUETTE_RADII_RESOLUTION as f32);
        radii.push(silhouette_radius_at_angle(theta, time));
    }
    let max_radius = radii.iter().cloned().fold(1e-6, f32::max);
    for radius in &mut radii {
        *radius /= max_radius;
    }
    radii
}

pub fn deform_vertices_from_silhouette_radii(vertices: &mut [Vector3], radii_normals: &[f32]) {
    if vertices.is_empty() {
        return;
    }
    for vertex in vertices.iter_mut() {
        let interpolated_radial_magnitude =
            interpolate_radial_magnitude_from_sample_xy(vertex.x, vertex.y, radii_normals);
        vertex.x *= interpolated_radial_magnitude;
        vertex.y *= interpolated_radial_magnitude;
    }
}

pub fn generate_silhouette_texture_fast(
    render: &mut RaylibRenderer,
    width: i32,
    height: i32,
    fade_frac: f32,
) -> Texture2D {
    let mut img = Image::gen_image_color(width, height, Color::BLANK);
    let data = unsafe { from_raw_parts_mut(img.data as *mut u8, (width * height * 4) as usize) };

    let v_fade_start = (1.0 - fade_frac.clamp(0.0, 0.95)) * (height as f32 - 1.0);
    for y in 0..height {
        let v = y as f32;
        let alpha = if v < v_fade_start {
            1.0
        } else {
            1.0 - (v - v_fade_start) / ((height as f32 - 1.0) - v_fade_start + 1e-6)
        }
        .clamp(0.0, 1.0);

        let a = (alpha * 255.0) as u8;
        for x in 0..width {
            let i = 4 * (y as usize * width as usize + x as usize);
            data[i..i + 4].copy_from_slice(&[255, 255, 255, a]);
        }
    }
    img.set_format(PIXELFORMAT_UNCOMPRESSED_R8G8B8A8);
    render.handle.load_texture_from_image(&render.thread, &img).unwrap()
}

pub fn interpolate_mesh_samples_and_texcoord_samples(
    model: &mut Model,
    i_time: f32,
    mesh_samples: &Vec<Vec<Vector3>>,
    texcoord_samples: &Vec<Vec<f32>>,
) {
    let target_mesh = &mut model.meshes_mut()[0];
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

pub fn interpolate_radial_magnitude_from_sample_xy(sample_x: f32, sample_y: f32, radii_normals: &[f32]) -> f32 {
    let radial_disk_angle = sample_y.atan2(sample_x).rem_euclid(TAU);
    let radial_index = radial_disk_angle / TAU * SILHOUETTE_RADII_RESOLUTION as f32;
    let lower_index = radial_index.floor() as usize % SILHOUETTE_RADII_RESOLUTION;
    let upper_index = (lower_index + 1) % SILHOUETTE_RADII_RESOLUTION;
    let interpolation_toward_upper = radial_index.fract();
    radii_normals[lower_index] * (1.0 - interpolation_toward_upper)
        + radii_normals[upper_index] * interpolation_toward_upper
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
pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[inline]
pub fn uv_to_grid_space(uv: Vector2) -> Vector2 {
    (uv - GRID_ORIGIN_UV_OFFSET) * GRID_SCALE
}

#[inline]
pub fn rotate_vertices(vertices: &mut Vec<Vector3>, rotation: f32) {
    for vertex in vertices {
        let (x0, z0) = (vertex.x, vertex.z);
        vertex.x = x0 * rotation.cos() + z0 * rotation.sin();
        vertex.z = -x0 * rotation.sin() + z0 * rotation.cos();
    }
}

#[inline]
pub fn warped_distance_from_mask_center_in_grid(grid_point: &mut Vector2, time_seconds: f32) -> f32 {
    let mut grid_phase = spatial_phase(*grid_point);
    grid_phase += temporal_phase(time_seconds);
    *grid_point += add_phase(grid_phase);
    grid_point.distance(UMBRAL_MASK_CENTER)
}

pub fn silhouette_uvs_polar(x: f32, y: f32, radii: &[f32]) -> (f32, f32) {
    let angle = y.atan2(x).rem_euclid(TAU);
    let sample_index = angle / TAU * (SILHOUETTE_RADII_RESOLUTION as f32);
    let i0 = sample_index.floor() as usize % SILHOUETTE_RADII_RESOLUTION;
    let i1 = (i0 + 1) % SILHOUETTE_RADII_RESOLUTION;
    let lerp_t = sample_index.fract();
    let edge_radius = radii[i0] * (1.0 - lerp_t) + radii[i1] * lerp_t;
    let distance_from_center = (x * x + y * y).sqrt();
    let u = sample_index / (SILHOUETTE_RADII_RESOLUTION as f32);
    let v = (distance_from_center / (edge_radius + 1e-6)).clamp(0.0, 1.0);
    (u, v)
}

#[inline]
pub fn silhouette_radius_at_angle(theta: f32, time_s: f32) -> f32 {
    let dir = Vector2::new(theta.cos(), theta.sin());
    let wiggle = LIGHT_WAVE_AMPLITUDE_X.hypot(LIGHT_WAVE_AMPLITUDE_Y) + 2.0;
    let mut lo = 0.0_f32;
    let mut hi = UMBRAL_MASK_OUTER_RADIUS + wiggle;
    for _ in 0..8 {
        let d = warped_distance_from_mask_center_in_grid(&mut (UMBRAL_MASK_CENTER + dir * hi), time_s);
        if d >= UMBRAL_MASK_OUTER_RADIUS {
            break;
        }
        hi *= 1.5;
    }
    for _ in 0..20 {
        let mid = 0.5 * (lo + hi);
        let d = warped_distance_from_mask_center_in_grid(&mut (UMBRAL_MASK_CENTER + dir * mid), time_s);
        if d >= UMBRAL_MASK_OUTER_RADIUS {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    hi
}

#[inline]
pub fn lift_dimension(vertex: Vec2) -> Vec3 {
    Vec3::new(vertex.x, vertex.y, 0.0)
}

#[inline]
pub fn rotate_point_about_axis(c: Vec3, axis: (Vec3, Vec3), theta: f32) -> Vec3 {
    let (a, b) = axis;
    let ab = b - a;
    let ab_axis_dir = ab.normalize_or_zero();
    let ac = c - a;
    let ac_z_component = ab_axis_dir.dot(ac) * ab_axis_dir;
    let ac_x_component = ac - ac_z_component;
    let ac_y_component = ab_axis_dir.cross(ac_x_component);
    let origin = a;
    let rotated_x_component = ac_x_component * theta.cos();
    let rotated_y_component = ac_y_component * theta.sin();
    //rotate in the xy plane
    let rotated_c = rotated_x_component + rotated_y_component + ac_z_component;
    origin + rotated_c
}
