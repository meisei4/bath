use crate::geometry::welding::{
    aggregate_to_welded, weld_for_smoothing_topo, SMOOTH_OLD_TO_WELDED, SMOOTH_WELDED_NEIGHBORS,
};
use crate::render::raylib::RaylibRenderer;
use asset_payload::SPHERE_PATH;
use raylib::color::Color;
use raylib::consts::PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8;
use raylib::math::{Vector2, Vector3};
use raylib::models::{Model, RaylibMesh, RaylibModel, WeakMesh};
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

const TEXTURE_MAPPING_BOUNDARY_FADE: f32 = 0.075;
pub const SILHOUETTE_TEXTURE_RES: i32 = 256;

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
    let width = silhouette_img.width as usize;
    let height = silhouette_img.height as usize;
    let center_x = (width as f32 - 1.0) * 0.5;
    let center_y = (height as f32 - 1.0) * 0.5;
    let total_bytes = width * height * 4;
    let pixels: &[u8] = unsafe { from_raw_parts(silhouette_img.data as *const u8, total_bytes) };
    let mut radii = vec![0.0_f32; SILHOUETTE_RADII_RESOLUTION];
    let step = 0.5_f32;
    let texture_bounds_padding = (width.max(height) as f32) * 0.75;
    let threshold = 1u8;
    for i in 0..SILHOUETTE_RADII_RESOLUTION {
        let theta = (i as f32) * TAU / SILHOUETTE_RADII_RESOLUTION as f32;
        let x_direction = theta.cos();
        let y_direction = theta.sin();
        let mut radial_magnitude = 0.0f32;
        while radial_magnitude < texture_bounds_padding {
            let sample_x = center_x + x_direction * radial_magnitude;
            let sample_y = center_y + y_direction * radial_magnitude;
            if sample_x < 0.0 || sample_y < 0.0 || sample_x >= (width as f32) || sample_y >= (height as f32) {
                break;
            }
            let lum = sample_lum(pixels, width, sample_x as i32, sample_y as i32);
            if lum <= threshold {
                break;
            }
            radial_magnitude += step;
        }
        radii[i] = radial_magnitude;
    }
    let max_radial_magnitude = radii.iter().cloned().fold(1e-6, f32::max);
    for radius in &mut radii {
        *radius /= max_radial_magnitude;
    }
    radii
}

#[inline]
fn sample_lum(pixels: &[u8], w: usize, x: i32, y: i32) -> u8 {
    if x < 0 || y < 0 || x as usize >= w || y as usize >= pixels.len() / (4 * w) {
        0
    } else {
        let i = 4 * (y as usize * w + x as usize);
        pixels[i]
    }
}

pub fn generate_silhouette_texture(render: &mut RaylibRenderer, texture_resolution: Vec<i32>) -> Texture2D {
    let texture_w = texture_resolution[0];
    let texture_h = texture_resolution[1];
    let padding_px: f32 = 5.0;
    let fade_fraction: f32 = TEXTURE_MAPPING_BOUNDARY_FADE.clamp(0.0, 0.95);
    let mut silhouette_img = Image::gen_image_color(texture_w, texture_h, Color::BLANK);
    let total_bytes = (texture_w * texture_h * 4) as usize;
    let pixels: &mut [u8] = unsafe { from_raw_parts_mut(silhouette_img.data as *mut u8, total_bytes) };
    let center_coord_x = (texture_w as f32 - 1.0) * 0.5;
    let center_coord_y = (texture_h as f32 - 1.0) * 0.5;
    let half_minimum = (texture_w.min(texture_h) as f32 - 1.0) * 0.5;
    let radius_outer = (half_minimum - padding_px).max(1.0);
    let radius_inner = radius_outer * (1.0 - fade_fraction.max(0.0));
    for y in 0..texture_h {
        for x in 0..texture_w {
            let sample_x = x as f32 - center_coord_x;
            let sample_y = y as f32 - center_coord_y;
            let radius = (sample_x * sample_x + sample_y * sample_y).sqrt();
            let alpha = if radius <= radius_inner {
                1.0
            } else if radius >= radius_outer {
                0.0
            } else {
                1.0 - (radius - radius_inner) / (radius_outer - radius_inner)
            };
            let rgb_val = (alpha * 255.0) as u8;
            let channel = ((y * texture_w + x) * 4) as usize;
            pixels[channel..channel + 4].copy_from_slice(&[rgb_val, rgb_val, rgb_val, (alpha * 255.0) as u8]);
        }
    }
    silhouette_img.set_format(PIXELFORMAT_UNCOMPRESSED_R8G8B8A8);
    let silhouette_texture = render
        .handle
        .load_texture_from_image(&render.thread, &silhouette_img)
        .unwrap();
    silhouette_texture
}

pub fn deform_vertices_from_silhouette_radii(vertices: &mut [Vector3], radii_normals: &[f32]) {
    if vertices.is_empty() {
        return;
    }
    let original_vertices = vertices.to_vec();
    for vertex in vertices.iter_mut() {
        let interpolated_radial_magnitude =
            interpolate_radial_magnitude_from_sample_xy(vertex.x, vertex.y, radii_normals);
        vertex.x *= interpolated_radial_magnitude;
        vertex.y *= interpolated_radial_magnitude;
    }
    weld_for_smoothing_topo(&original_vertices);
    let min_inside = 1e-8;
    let target_z_per_original = compute_target_depth_from_original_radius(&original_vertices, vertices, min_inside);
    let (old_to_welded, welded_neighbors) = unsafe {
        (
            SMOOTH_OLD_TO_WELDED.as_ref().unwrap(),
            SMOOTH_WELDED_NEIGHBORS.as_ref().unwrap(),
        )
    };
    let target_z_welded = aggregate_to_welded(old_to_welded, &target_z_per_original);
    static mut SMOOTH_PREV_DEPTH_WELDED: Option<Vec<f32>> = None;
    let prev_opt = unsafe { SMOOTH_PREV_DEPTH_WELDED.as_deref() };

    let smoothness_weight: f32 = 4.0;
    let target_weight: f32 = 6.0;
    let temporal_weight: f32 = 2.0;
    let max_iter: usize = 64;
    let tol: f32 = 1e-5;

    let solved_z_welded = solve_depth_for_smooth_blob(
        welded_neighbors,
        &target_z_welded,
        prev_opt,
        smoothness_weight,
        target_weight,
        temporal_weight,
        max_iter,
        tol,
    );

    for (i, v) in vertices.iter_mut().enumerate() {
        v.z = solved_z_welded[old_to_welded[i]];
    }
    unsafe {
        SMOOTH_PREV_DEPTH_WELDED = Some(solved_z_welded);
    }
}

pub fn build_vertex_neighbors(vertex_count: usize, triangle_indices: &[u16]) -> Vec<Vec<usize>> {
    let mut neighbors = vec![Vec::<usize>::new(); vertex_count];
    for triangle in triangle_indices.chunks_exact(3) {
        let a = triangle[0] as usize;
        let b = triangle[1] as usize;
        let c = triangle[2] as usize;
        let edges = [(a, b), (b, c), (c, a)];
        for (i, j) in edges {
            if !neighbors[i].contains(&j) {
                neighbors[i].push(j);
            }
            if !neighbors[j].contains(&i) {
                neighbors[j].push(i);
            }
        }
    }
    neighbors
}

fn apply_linear_system_matrix(
    depth_values_in: &[f32],
    neighbors: &[Vec<usize>],
    smoothness_weight: f32,
    diagonal_weight: f32,
    depth_values_out: &mut [f32],
) {
    for (vertex_index, neighbors) in neighbors.iter().enumerate() {
        let mut neighbor_sum = 0.0;
        for &i in neighbors {
            neighbor_sum += depth_values_in[i];
        }
        let degree_count = neighbors.len() as f32;
        depth_values_out[vertex_index] = smoothness_weight
            * (degree_count * depth_values_in[vertex_index] - neighbor_sum)
            + diagonal_weight * depth_values_in[vertex_index];
    }
}

fn conjugate_gradient_solve_spd<F>(
    dimension: usize,
    mut apply_matrix: F,
    right_hand_side: &[f32],
    solution_depth_values: &mut [f32],
    maximum_iterations: usize,
    tolerance: f32,
) where
    F: FnMut(&[f32], &mut [f32]),
{
    let mut residual_vector = vec![0.0; dimension];
    let mut search_direction_vector = vec![0.0; dimension];
    let mut matrix_times_direction = vec![0.0; dimension];
    apply_matrix(solution_depth_values, &mut residual_vector);
    for i in 0..dimension {
        residual_vector[i] = right_hand_side[i] - residual_vector[i];
        search_direction_vector[i] = residual_vector[i];
    }
    let tolerance_squared = tolerance * tolerance;
    let mut residual_norm_squared_previous: f32 = residual_vector.iter().map(|v| v * v).sum();
    for _ in 0..maximum_iterations {
        apply_matrix(&search_direction_vector, &mut matrix_times_direction);
        let denominator: f32 = search_direction_vector
            .iter()
            .zip(matrix_times_direction.iter())
            .map(|(p, ap)| p * ap)
            .sum();
        if denominator.abs() < 1e-20 {
            break;
        }
        let step_size_alpha = residual_norm_squared_previous / denominator;
        for i in 0..dimension {
            solution_depth_values[i] += step_size_alpha * search_direction_vector[i];
            residual_vector[i] -= step_size_alpha * matrix_times_direction[i];
        }
        let residual_norm_squared_current: f32 = residual_vector.iter().map(|v| v * v).sum();
        if residual_norm_squared_current <= tolerance_squared {
            break;
        }
        let conjugate_coefficient_beta = residual_norm_squared_current / residual_norm_squared_previous;
        for i in 0..dimension {
            search_direction_vector[i] = residual_vector[i] + conjugate_coefficient_beta * search_direction_vector[i];
        }
        residual_norm_squared_previous = residual_norm_squared_current;
    }
}

pub fn compute_target_depth_from_original_radius(
    sphere_vertices: &[Vector3],
    xy_deformed_vertices: &[Vector3],
    minimum_inside_value: f32,
) -> Vec<f32> {
    let mut target_depth_values = Vec::with_capacity(xy_deformed_vertices.len());
    for (original, scaled_xy) in sphere_vertices.iter().zip(xy_deformed_vertices.iter()) {
        let original_radius_squared = original.x * original.x + original.y * original.y + original.z * original.z;
        let scaled_xy_radius_squared = scaled_xy.x * scaled_xy.x + scaled_xy.y * scaled_xy.y;
        let inside_value = (original_radius_squared - scaled_xy_radius_squared).max(minimum_inside_value);
        let depth_magnitude = inside_value.sqrt();
        let sign = if original.z >= 0.0 { 1.0 } else { -1.0 };
        target_depth_values.push(sign * depth_magnitude);
    }
    target_depth_values
}

pub fn solve_depth_for_smooth_blob(
    neighbors: &[Vec<usize>],
    target_depth_values: &[f32],
    previous_frame_depth_values: Option<&[f32]>,
    smoothness_weight: f32,
    target_weight: f32,
    temporal_weight: f32,
    maximum_iterations: usize,
    tolerance: f32,
) -> Vec<f32> {
    let vertex_count = target_depth_values.len();
    let diagonal_weight = target_weight
        + if previous_frame_depth_values.is_some() {
            temporal_weight
        } else {
            0.0
        };

    let mut right_hand_side = vec![0.0; vertex_count];
    for i in 0..vertex_count {
        right_hand_side[i] = target_weight * target_depth_values[i]
            + if let Some(prev) = previous_frame_depth_values {
                temporal_weight * prev[i]
            } else {
                0.0
            };
    }
    let mut solution_depth_values = if let Some(prev) = previous_frame_depth_values {
        prev.to_vec()
    } else {
        vec![0.0; vertex_count]
    };
    let mut apply_matrix = |input: &[f32], output: &mut [f32]| {
        apply_linear_system_matrix(input, neighbors, smoothness_weight, diagonal_weight, output);
    };
    conjugate_gradient_solve_spd(
        vertex_count,
        &mut apply_matrix,
        &right_hand_side,
        &mut solution_depth_values,
        maximum_iterations,
        tolerance,
    );
    solution_depth_values
}

pub fn update_z_depth(vertices: &mut [Vector3], depth_values: &[f32]) {
    for (vertex, depth) in vertices.iter_mut().zip(depth_values.iter()) {
        vertex.z = *depth;
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

            let uv_shrink = 0.98; // try 0.98..0.995
            let uv_bias = 0.5 * (1.0 - uv_shrink);
            let u = uv_bias + uv_shrink * (0.5 + 0.5 * (x / interpolated_radial_magnitude));
            let v = uv_bias + uv_shrink * (0.5 + 0.5 * (y / interpolated_radial_magnitude));

            // let u = x / interpolated_radial_magnitude * 0.5 + 0.5;
            // let v = y / interpolated_radial_magnitude * 0.5 + 0.5;
            texcoord_sample.push(u);
            texcoord_sample.push(v);
        }
        texcoord_samples.push(texcoord_sample);
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

pub fn interpolate_mesh_and_texcoord_samples(
    model: &mut Model,
    i_time: f32,
    mesh_samples: &Vec<Vec<Vector3>>,
    texcoord_samples: &Vec<Vec<f32>>,
) {
    let mesh = &mut model.meshes_mut()[0];
    lerp_intermediate_mesh_samples_to_single_mesh(i_time, mesh_samples, texcoord_samples, mesh);
    // subdivide_tris_no_index(main_mesh, 1); //TODO: incredible graphics glitch i guess??
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

pub fn interpolate_radial_magnitude_from_sample_xy(sample_x: f32, sample_y: f32, radii_normals: &[f32]) -> f32 {
    let radial_disk_angle = sample_y.atan2(sample_x).rem_euclid(TAU);
    let radial_index = radial_disk_angle / TAU * SILHOUETTE_RADII_RESOLUTION as f32;
    let lower_index = radial_index.floor() as usize % SILHOUETTE_RADII_RESOLUTION;
    let upper_index = (lower_index + 1) % SILHOUETTE_RADII_RESOLUTION;
    let interpolation_toward_upper = radial_index.fract();
    radii_normals[lower_index] * (1.0 - interpolation_toward_upper)
        + radii_normals[upper_index] * interpolation_toward_upper
}

pub fn subdivide_tris_no_index(mesh: &mut WeakMesh, iterations: usize) {
    for _ in 0..iterations {
        // assume no indices: 3 vertices per tri
        let vc = mesh.vertexCount as usize;
        assert_eq!(vc % 3, 0);
        let old_v = unsafe { from_raw_parts(mesh.vertices, vc * 3) };
        let old_t = if mesh.texcoords.is_null() {
            vec![]
        } else {
            unsafe { from_raw_parts(mesh.texcoords, vc * 2) }.to_vec()
        };
        let mut new_v: Vec<f32> = Vec::with_capacity(vc * 2 * 3);
        let mut new_t: Vec<f32> = Vec::with_capacity(old_t.len() * 2);
        for tri in 0..(vc / 3) {
            let i0 = tri * 3;
            let i1 = i0 + 1;
            let i2 = i0 + 2;
            let a = &old_v[i0 * 3..i0 * 3 + 3];
            let b = &old_v[i1 * 3..i1 * 3 + 3];
            let c = &old_v[i2 * 3..i2 * 3 + 3];
            let ab = [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5, (a[2] + b[2]) * 0.5];
            let bc = [(b[0] + c[0]) * 0.5, (b[1] + c[1]) * 0.5, (b[2] + c[2]) * 0.5];
            let ca = [(c[0] + a[0]) * 0.5, (c[1] + a[1]) * 0.5, (c[2] + a[2]) * 0.5];

            // texcoords (if present)
            let (ta, tb, tc, tab, tbc, tca) = if !old_t.is_empty() {
                let ta = &old_t[i0 * 2..i0 * 2 + 2];
                let tb = &old_t[i1 * 2..i1 * 2 + 2];
                let tc = &old_t[i2 * 2..i2 * 2 + 2];
                let tab = vec![(ta[0] + tb[0]) * 0.5, (ta[1] + tb[1]) * 0.5];
                let tbc = vec![(tb[0] + tc[0]) * 0.5, (tb[1] + tc[1]) * 0.5];
                let tca = vec![(tc[0] + ta[0]) * 0.5, (tc[1] + ta[1]) * 0.5];
                (ta.to_vec(), tb.to_vec(), tc.to_vec(), tab, tbc, tca)
            } else {
                (
                    vec![0.0, 0.0],
                    vec![0.0, 0.0],
                    vec![0.0, 0.0],
                    vec![0.0, 0.0],
                    vec![0.0, 0.0],
                    vec![0.0, 0.0],
                )
            };
            // 4 new triangles: A-AB-CA, AB-B-BC, CA-BC-C, AB-BC-CA
            let tris = [
                (a, &ab, &ca, &ta, &tab, &tca),
                (&ab, <&[f32; 3]>::try_from(b).unwrap(), &bc, &tab, &tb, &tbc),
                (&ca, &bc, <&[f32; 3]>::try_from(c).unwrap(), &tca, &tbc, &tc),
                (&ab, &bc, &ca, &tab, &tbc, &tca),
            ];

            for (p0, p1, p2, t0, t1, t2) in tris {
                new_v.extend_from_slice(p0);
                new_v.extend_from_slice(p1);
                new_v.extend_from_slice(p2);
                if !old_t.is_empty() {
                    new_t.extend_from_slice(t0);
                    new_t.extend_from_slice(t1);
                    new_t.extend_from_slice(t2);
                }
            }
        }
        unsafe {
            // replace mesh data
            raylib::ffi::MemFree(mesh.vertices as *mut _);
            mesh.vertices = raylib::ffi::MemAlloc((new_v.len() * size_of::<f32>()) as u32) as *mut f32;
            std::ptr::copy_nonoverlapping(new_v.as_ptr(), mesh.vertices, new_v.len());
            mesh.vertexCount = (new_v.len() / 3) as i32;
            mesh.triangleCount = (new_v.len() / 9) as i32;

            if !old_t.is_empty() {
                if !mesh.texcoords.is_null() {
                    raylib::ffi::MemFree(mesh.texcoords as *mut _);
                }
                mesh.texcoords = raylib::ffi::MemAlloc((new_t.len() * std::mem::size_of::<f32>()) as u32) as *mut f32;
                std::ptr::copy_nonoverlapping(new_t.as_ptr(), mesh.texcoords, new_t.len());
            }
        }
    }
}
pub fn deform_vertices_from_silhouette_radii1(vertices: &mut [Vector3], radii_normals: &[f32]) {
    for vertex in vertices {
        let sample_x = vertex.x;
        let sample_y = vertex.y;
        let interpolated_radial_magnitude =
            interpolate_radial_magnitude_from_sample_xy(sample_x, sample_y, &radii_normals);
        vertex.x *= interpolated_radial_magnitude;
        vertex.y *= interpolated_radial_magnitude;
        vertex.z *= interpolated_radial_magnitude;
    }
}
