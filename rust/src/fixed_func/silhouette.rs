use crate::fixed_func::topology::{
    collect_vertex_normals, rotate_point_about_axis, smooth_vertex_normals, topology_init,
};
use crate::render::raylib::RaylibRenderer;
use asset_payload::SPHERE_PATH;
use raylib::color::Color;
use raylib::drawing::{RaylibDraw3D, RaylibDrawHandle, RaylibMode3D};
use raylib::math::glam::Vec3;
use raylib::math::{Vector2, Vector3};
use raylib::models::{Model, RaylibMesh, RaylibModel, WeakMesh};
use std::f32::consts::TAU;
use std::slice::from_raw_parts_mut;

pub const MODEL_POS: Vector3 = Vector3::ZERO;
pub const SCALE_TWEAK: f32 = 0.66;
// pub const SCALE_TWEAK: f32 = 1.0;
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

pub const RADIAL_FIELD_SIZE: usize = 64;

pub const ROTATION_FREQUENCY_HZ: f32 = 0.05;
pub const ANGULAR_VELOCITY: f32 = TAU * ROTATION_FREQUENCY_HZ;
pub const TIME_BETWEEN_SAMPLES: f32 = 0.5;
pub const ROTATIONAL_SAMPLES_FOR_INV_PROJ: usize = 40;

pub const TEXTURE_MAPPING_BOUNDARY_FADE: f32 = 0.05;
pub const SILHOUETTE_TEXTURE_RES: i32 = 256 / 2;

pub const INVERTED_HULL_EXPANSION_SCALAR: f32 = 0.15;
pub const ALPHA_FADE_RAMP_MIN: f32 = 0.0;
pub const ALPHA_FADE_RAMP_MAX: f32 = 0.5;
pub const ALPHA_FADE_RAMP_STRENGTH: f32 = 1.0;

pub const FOVY: f32 = 2.0;

pub const GAUSSIAN_ALPHA_FADE_THICKNESS_IN_PIXELS: f32 = 24.0;
pub const GAUSSIAN_STACK_SIZE: usize = 3;
fn pascal_pass(passes: usize) -> &'static [u32] {
    match passes {
        2 => &[1, 1],          // super cheap, very hard falloff
        3 => &[1, 2, 1],       // good + still cheap
        4 => &[1, 3, 3, 1],    // smoother; still inexpensive
        5 => &[1, 4, 6, 4, 1], // nicer; may be OK if you have headroom
        _ => &[1, 2, 1],       // default to 3 passes if someone asks for too many
    }
}
pub const GAUSSIAN_MAX_ALPHA: u8 = 120u8;

pub fn collect_deformed_mesh_samples(renderer: &mut RaylibRenderer) -> Vec<Vec<Vector3>> {
    let model = renderer.handle.load_model(&renderer.thread, SPHERE_PATH).unwrap();
    let vertices = model.meshes()[0].vertices();
    let mut mesh_samples = Vec::with_capacity(ROTATIONAL_SAMPLES_FOR_INV_PROJ);
    for i in 0..ROTATIONAL_SAMPLES_FOR_INV_PROJ {
        let sample_time = i as f32 * TIME_BETWEEN_SAMPLES;
        let sample_rotation = -ANGULAR_VELOCITY * sample_time;
        let mut mesh_sample = vertices.to_vec();
        rotate_vertices(&mut mesh_sample, sample_rotation);
        let radial_field = generate_silhouette_radial_field(sample_time);
        deform_vertices_with_radial_field(&mut mesh_sample, &radial_field);
        rotate_vertices(&mut mesh_sample, -sample_rotation);
        mesh_samples.push(mesh_sample);
    }
    mesh_samples
}

pub fn generate_silhouette_radial_field(i_time: f32) -> Vec<f32> {
    let mut radial_field = Vec::with_capacity(RADIAL_FIELD_SIZE);
    for i in 0..RADIAL_FIELD_SIZE {
        let radial_field_angle = (i as f32) * TAU / (RADIAL_FIELD_SIZE as f32);
        radial_field.push(deformed_silhouette_radius_at_angle(radial_field_angle, i_time));
    }
    let max_radius = radial_field.iter().cloned().fold(1e-6, f32::max);
    for radius in &mut radial_field {
        *radius /= max_radius;
    }
    radial_field
}

pub fn deform_vertices_with_radial_field(vertices: &mut [Vector3], radial_field: &[f32]) {
    if vertices.is_empty() {
        return;
    }
    for vertex in vertices.iter_mut() {
        let interpolated_radial_magnitude = interpolate_between_radial_field_elements(vertex.x, vertex.y, radial_field);
        vertex.x *= interpolated_radial_magnitude;
        vertex.y *= interpolated_radial_magnitude;
    }
}

pub fn interpolate_between_deformed_meshes(model: &mut Model, i_time: f32, mesh_samples: &Vec<Vec<Vector3>>) {
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
}

pub fn interpolate_between_radial_field_elements(sample_x: f32, sample_y: f32, radial_field: &[f32]) -> f32 {
    let radial_disk_angle = sample_y.atan2(sample_x).rem_euclid(TAU);
    let radial_index = radial_disk_angle / TAU * RADIAL_FIELD_SIZE as f32;
    let lower_index = radial_index.floor() as usize % RADIAL_FIELD_SIZE;
    let upper_index = (lower_index + 1) % RADIAL_FIELD_SIZE;
    let interpolation_toward_upper = radial_index.fract();
    radial_field[lower_index] * (1.0 - interpolation_toward_upper)
        + radial_field[upper_index] * interpolation_toward_upper
}

pub fn rotate_inverted_hull(
    model: &Model,
    inverted_hull: &mut Model,
    observed_line_of_sight: Vec3,
    mesh_rotation: f32,
) {
    let line_of_sight = rotate_point_about_axis(-observed_line_of_sight, (Vector3::ZERO, Vector3::Y), -mesh_rotation);
    let mesh = &model.meshes()[0];
    let inverted_hull_mesh = &mut inverted_hull.meshes_mut()[0];
    let vertex_count = mesh.vertexCount as usize;
    let mut topology = topology_init(mesh);
    collect_vertex_normals(&mut topology, &model.meshes()[0]);
    let welded_vertex_normals = smooth_vertex_normals(&topology);
    if inverted_hull_mesh.colors.is_null() {
        let colors = vec![255u8; vertex_count * 4];
        inverted_hull_mesh.colors = Box::leak(colors.into_boxed_slice()).as_mut_ptr();
    }
    let inverted_hull_colors = unsafe { from_raw_parts_mut(inverted_hull_mesh.colors, vertex_count * 4) };
    let vertices = mesh.vertices();
    let inverted_hull_vertices = inverted_hull_mesh.vertices_mut();
    for i in 0..vertex_count {
        let vertex = vertices[i];
        let face_normal = welded_vertex_normals.get(i).copied().unwrap_or(Vec3::Z);
        let expanded_vertex = vertex + face_normal * INVERTED_HULL_EXPANSION_SCALAR;
        inverted_hull_vertices[i] = expanded_vertex;
        let view_alignment_magnitude = face_normal.dot(line_of_sight).abs();
        let edge_weight = 1.0 - smoothstep(ALPHA_FADE_RAMP_MIN, ALPHA_FADE_RAMP_MAX, view_alignment_magnitude);
        let alpha_1_to_0 = (1.0 - edge_weight * ALPHA_FADE_RAMP_STRENGTH).clamp(0.0, 1.0);
        let j = i * 4;
        inverted_hull_colors[j + 0] = 255;
        inverted_hull_colors[j + 1] = 255;
        inverted_hull_colors[j + 2] = 255;
        // inverted_hull_colors[j + 0] = 0;
        // inverted_hull_colors[j + 1] = 0;
        // inverted_hull_colors[j + 2] = 0;
        inverted_hull_colors[j + 3] = (alpha_1_to_0 * 255.0).round() as u8;
    }
}

pub fn draw_inverted_hull_guassian_silhouette_stack(
    rl3d: &mut RaylibMode3D<RaylibDrawHandle>,
    inverted_hull_model: &Model,
    mesh_rotation: f32,
) {
    let screen_h = rl3d.get_screen_height();
    let max_silhouette_radius = max_silhouette_radius(&inverted_hull_model.meshes()[0], MODEL_SCALE * SCALE_TWEAK);
    let gaussian_stack = build_gaussian_silhouette_stack(screen_h, max_silhouette_radius);
    for (scale, alpha) in gaussian_stack {
        rl3d.draw_model_ex(
            inverted_hull_model,
            MODEL_POS,
            Vector3::Y,
            mesh_rotation.to_degrees(),
            MODEL_SCALE * SCALE_TWEAK * 0.82 * scale,
            Color::new(255, 255, 255, alpha),
        );
    }
}

pub fn build_gaussian_silhouette_stack(screen_h: i32, max_silhouette_radius: f32) -> Vec<(f32, u8)> {
    if GAUSSIAN_STACK_SIZE == 0 {
        return Vec::new();
    }
    let pixels_to_world = FOVY / screen_h as f32;
    let alpha_feather_thickness_in_world = GAUSSIAN_ALPHA_FADE_THICKNESS_IN_PIXELS * pixels_to_world;
    let step_world = alpha_feather_thickness_in_world / GAUSSIAN_STACK_SIZE as f32;
    let pascal = pascal_pass(GAUSSIAN_STACK_SIZE);
    let weight_sum: u32 = pascal.iter().copied().sum();
    let inverse_sum = if weight_sum > 0 { 1.0 / weight_sum as f32 } else { 0.0 };
    let mut scale_alpha_pairs: Vec<(f32, u8)> = Vec::with_capacity(GAUSSIAN_STACK_SIZE);
    for pass in 1..=GAUSSIAN_STACK_SIZE {
        let outward_offset = pass as f32 * step_world;
        let silhouette_scale = 1.0 + outward_offset / max_silhouette_radius.max(1e-6);
        let weight = pascal[pass - 1] as f32 * inverse_sum;
        let alpha = (weight * GAUSSIAN_MAX_ALPHA as f32).round().clamp(0.0, 255.0) as u8;
        scale_alpha_pairs.push((silhouette_scale, alpha));
    }
    scale_alpha_pairs
}

pub fn max_silhouette_radius(mesh: &WeakMesh, model_scale: Vector3) -> f32 {
    let vertices = mesh.vertices();
    if vertices.is_empty() {
        return 1.0;
    }
    let mut max_silhouette_radius = 0.0f32;
    for vertex in vertices {
        let x = vertex.x * model_scale.x;
        let y = vertex.y * model_scale.y;
        let radius = (x * x + y * y).sqrt();
        if radius > max_silhouette_radius {
            max_silhouette_radius = radius;
        }
    }
    max_silhouette_radius.max(1e-6)
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
pub fn grid_phase_magnitude(grid_coord: &mut Vector2, i_time: f32) -> f32 {
    let mut grid_phase = spatial_phase(*grid_coord);
    grid_phase += temporal_phase(i_time);
    *grid_coord += add_phase(grid_phase);
    grid_coord.distance(UMBRAL_MASK_CENTER)
}

#[inline]
pub fn deformed_silhouette_radius_at_angle(radial_field_angle: f32, i_time: f32) -> f32 {
    let direction_vector = Vector2::new(radial_field_angle.cos(), radial_field_angle.sin());
    let phase = LIGHT_WAVE_AMPLITUDE_X.hypot(LIGHT_WAVE_AMPLITUDE_Y) + 2.0;
    let mut lower_phase_radius = 0.0_f32;
    let mut upper_phase_radius = UMBRAL_MASK_OUTER_RADIUS + phase;
    for _ in 0..8 {
        let current_radius = grid_phase_magnitude(
            &mut (UMBRAL_MASK_CENTER + direction_vector * upper_phase_radius),
            i_time,
        );
        if current_radius >= UMBRAL_MASK_OUTER_RADIUS {
            break;
        }
        upper_phase_radius *= 1.5;
    }
    for _ in 0..20 {
        let mid_phase_radius = 0.5 * (lower_phase_radius + upper_phase_radius);
        let current_radius =
            grid_phase_magnitude(&mut (UMBRAL_MASK_CENTER + direction_vector * mid_phase_radius), i_time);
        if current_radius >= UMBRAL_MASK_OUTER_RADIUS {
            upper_phase_radius = mid_phase_radius;
        } else {
            lower_phase_radius = mid_phase_radius;
        }
    }
    upper_phase_radius
}
