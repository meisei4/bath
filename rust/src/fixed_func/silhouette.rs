use crate::fixed_func::topology::{
    collect_vertex_normals, observed_line_of_sight, rotate_point_about_axis, smooth_vertex_normals, topology_init,
};
use crate::render::raylib::RaylibRenderer;
use asset_payload::SPHERE_PATH;
use raylib::camera::Camera3D;
use raylib::color::Color;
use raylib::consts::PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8;
use raylib::drawing::{RaylibDraw, RaylibDraw3D, RaylibDrawHandle, RaylibMode3D};
use raylib::ffi::{rlDisableDepthMask, rlEnableDepthMask, rlReadScreenPixels, MemFree};
use raylib::math::glam::Vec3;
use raylib::math::{Rectangle, Vector2, Vector3};
use raylib::models::{Model, RaylibMesh, RaylibModel, WeakMesh};
use raylib::texture::{Image, RaylibTexture2D, WeakTexture2D};
use std::f32::consts::TAU;
use std::ffi::c_void;
use std::ptr::copy_nonoverlapping;
use std::slice::from_raw_parts_mut;

pub const MODEL_POS: Vector3 = Vector3::ZERO;
pub const SCALE_TWEAK: f32 = 0.66;
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

pub const DITHER_TEXTURE_SCALE: f32 = 16.0;
pub const DITHER_BLEND_FACTOR: f32 = 1.0;

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
pub const GAUSSIAN_STACK_SIZE: usize = 2;
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
    unsafe {
        rlDisableDepthMask();
    }
    for (scale, alpha) in gaussian_stack {
        rl3d.draw_model_ex(
            inverted_hull_model,
            MODEL_POS,
            Vector3::Y,
            mesh_rotation.to_degrees(),
            MODEL_SCALE * SCALE_TWEAK * scale,
            Color::new(255, 255, 255, alpha),
        );
    }
    unsafe {
        rlEnableDepthMask();
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

const BAYER_SIZE: usize = 8;
const BAYER_8X8_RANKS: [[u8; BAYER_SIZE]; BAYER_SIZE] = [
    [0, 32, 8, 40, 2, 34, 10, 42],
    [48, 16, 56, 24, 50, 18, 58, 26],
    [12, 44, 4, 36, 14, 46, 6, 38],
    [60, 28, 52, 20, 62, 30, 54, 22],
    [3, 35, 11, 43, 1, 33, 9, 41],
    [51, 19, 59, 27, 49, 17, 57, 25],
    [15, 47, 7, 39, 13, 45, 5, 37],
    [63, 31, 55, 23, 61, 29, 53, 21],
];

pub fn generate_silhouette_texture(texture_w: i32, texture_h: i32) -> Image {
    let mut image = Image::gen_image_color(texture_w, texture_h, Color::BLANK);
    let total_byte_count = (texture_w * texture_h * 4) as usize;
    let pixel_data_bytes = unsafe { from_raw_parts_mut(image.data as *mut u8, total_byte_count) };
    for texel_y in 0..texture_h {
        let row_normalized = texel_y as f32 / (texture_h as f32 - 1.0);
        let alpha_1_to_0 = (1.0 - smoothstep(0.0, 1.0, row_normalized)).clamp(0.0, 1.0);
        let alpha_0_to_255 = (alpha_1_to_0 * 255.0).round() as u8;
        for texel_x in 0..texture_w {
            let pixel_index = 4 * (texel_y as usize * texture_w as usize + texel_x as usize);
            pixel_data_bytes[pixel_index + 0] = 255;
            pixel_data_bytes[pixel_index + 1] = 255;
            pixel_data_bytes[pixel_index + 2] = 255;
            pixel_data_bytes[pixel_index + 3] = alpha_0_to_255;
        }
    }
    image.set_format(PIXELFORMAT_UNCOMPRESSED_R8G8B8A8);
    image
}

pub fn dither(silhouette_image: &mut Image) {
    let total_bytes = (silhouette_image.width * silhouette_image.height * 4) as usize;
    let pixel_data = unsafe { from_raw_parts_mut(silhouette_image.data as *mut u8, total_bytes) };
    let bayer_blend = DITHER_BLEND_FACTOR.clamp(0.0, 1.0);
    for y in 0..silhouette_image.height as usize {
        for x in 0..silhouette_image.width as usize {
            let pixel_index = 4 * (y * silhouette_image.width as usize + x);
            let alpha_1_to_0 = (pixel_data[pixel_index + 3] as f32) / 255.0;
            let bayer_x = x % BAYER_SIZE;
            let bayer_y = y % BAYER_SIZE;
            let threshold_0_to_1 = (BAYER_8X8_RANKS[bayer_y][bayer_x] as f32) / 64.0;
            let bayer_check = if alpha_1_to_0 > threshold_0_to_1 { 1.0 } else { 0.0 };
            let bayer_alpha_1_to_0 = alpha_1_to_0 * (1.0 - bayer_blend) + bayer_check * bayer_blend;
            pixel_data[pixel_index + 3] = (bayer_alpha_1_to_0 * 255.0).round() as u8;
        }
    }
}

pub fn rotate_silhouette_texture_dither(
    model: &mut Model,
    observer: &Camera3D,
    mesh_rotation: f32,
    screen_w: i32,
    screen_h: i32,
) {
    let mesh = &mut model.meshes_mut()[0];
    let mut vertices = mesh.vertices_mut().to_vec();
    let mut smooth_vertex_normals = {
        let mut topology = topology_init(mesh);
        collect_vertex_normals(&mut topology, mesh);
        smooth_vertex_normals(&topology)
    };
    rotate_vertices(&mut vertices, mesh_rotation);
    rotate_vertices(&mut smooth_vertex_normals, mesh_rotation);
    let observed_line_of_sight = observed_line_of_sight(observer);
    let vertex_count = mesh.vertexCount as usize;
    if mesh.as_mut().texcoords.is_null() {
        let texcoords = vec![0.0f32; vertex_count * 2];
        mesh.as_mut().texcoords = Box::leak(texcoords.into_boxed_slice()).as_mut_ptr();
    }
    let texcoords = unsafe { from_raw_parts_mut(mesh.as_mut().texcoords, vertex_count * 2) };
    let world_to_pixels = screen_h as f32 / FOVY;
    for i in 0..vertex_count {
        let vertex = vertices[i];
        let vertex_normal = smooth_vertex_normals[i];
        let x_component = vertex.x * world_to_pixels + (screen_w as f32) * 0.5;
        let s = x_component / DITHER_TEXTURE_SCALE;
        let alignment_magnitude = vertex_normal.dot(observed_line_of_sight).abs();
        // let t = smoothstep(ALPHA_FADE_RAMP_MIN, ALPHA_FADE_RAMP_MAX, alignment_magnitude);
        let t = 1.0 - smoothstep(ALPHA_FADE_RAMP_MIN, ALPHA_FADE_RAMP_MAX, alignment_magnitude);
        texcoords[i * 2 + 0] = s;
        texcoords[i * 2 + 1] = t;
    }
}

pub fn rotate_silhouette_texture(model: &mut Model, observer: &Camera3D, mesh_rotation: f32) {
    let mesh = &mut model.meshes_mut()[0];
    let mut vertices = mesh.vertices_mut().to_vec();
    let mut smooth_vertex_normals = {
        let mut topology = topology_init(mesh);
        collect_vertex_normals(&mut topology, mesh);
        smooth_vertex_normals(&topology)
    };
    rotate_vertices(&mut vertices, mesh_rotation);
    rotate_vertices(&mut smooth_vertex_normals, mesh_rotation);
    let observed_line_of_sight = observed_line_of_sight(observer);
    let vertex_count = mesh.vertexCount as usize;
    if mesh.as_mut().texcoords.is_null() {
        let texcoords = vec![0.0f32; vertex_count * 2];
        mesh.as_mut().texcoords = Box::leak(texcoords.into_boxed_slice()).as_mut_ptr();
    }
    let texcoords = unsafe { from_raw_parts_mut(mesh.as_mut().texcoords, vertex_count * 2) };
    for vertex_index in 0..vertex_count {
        let vertex = vertices[vertex_index];
        let vertex_normal = smooth_vertex_normals[vertex_index];
        let angle_component = vertex.y.atan2(vertex.x).rem_euclid(TAU);
        let s = angle_component / TAU;

        let alignment_magnitude = vertex_normal.dot(observed_line_of_sight).abs();
        let t = 1.0 - smoothstep(ALPHA_FADE_RAMP_MIN, ALPHA_FADE_RAMP_MAX, alignment_magnitude);

        texcoords[vertex_index * 2 + 0] = s;
        texcoords[vertex_index * 2 + 1] = t;
    }
}

pub struct DitherStaging {
    pub blit_texture: WeakTexture2D,
    pub is_initialized: bool,
    pub staging_rgba_bytes: Vec<u8>,
}

pub fn screen_pass_dither(draw_handle: &mut RaylibDrawHandle, dither_staging: &mut DitherStaging) {
    let screen_w = draw_handle.get_screen_width();
    let screen_h = draw_handle.get_screen_height();
    let byte_count = (screen_w * screen_h * 4) as usize;
    if dither_staging.staging_rgba_bytes.len() != byte_count {
        dither_staging.staging_rgba_bytes.resize(byte_count, 0);
    }
    unsafe {
        let screen_pixels = rlReadScreenPixels(screen_w, screen_h);
        copy_nonoverlapping(
            screen_pixels,
            dither_staging.staging_rgba_bytes.as_mut_ptr(),
            byte_count,
        );
        MemFree(screen_pixels as *mut c_void);
    }

    dither_byte_level(&mut dither_staging.staging_rgba_bytes, screen_w, screen_h);
    dither_staging
        .blit_texture
        .update_texture(&dither_staging.staging_rgba_bytes)
        .unwrap();
    let src = Rectangle {
        x: 0.0,
        y: 0.0,
        width: screen_w as f32,
        height: screen_h as f32,
    };
    let dst = Rectangle {
        x: 0.0,
        y: 0.0,
        width: screen_w as f32,
        height: screen_h as f32,
    };
    draw_handle.draw_texture_pro(&dither_staging.blit_texture, src, dst, Vector2::ZERO, 0.0, Color::WHITE);
}

pub fn dither_byte_level(pixels_rgba8: &mut [u8], width_pixels: i32, height_pixels: i32) {
    let blend_lambda = DITHER_BLEND_FACTOR.clamp(0.0, 1.0);
    for screen_y in 0..height_pixels {
        for screen_x in 0..width_pixels {
            let pixel_index = 4 * (screen_y as usize * width_pixels as usize + screen_x as usize);
            let r = pixels_rgba8[pixel_index + 0];
            let g = pixels_rgba8[pixel_index + 1];
            let b = pixels_rgba8[pixel_index + 2];
            let lum = luminance(r, g, b);
            let cell_x_index = (screen_x as usize) & 7;
            let cell_y_index = (screen_y as usize) & 7;
            let threshold_0_to_1 = (BAYER_8X8_RANKS[cell_y_index][cell_x_index] as f32) / 64.0;
            let on = if lum > threshold_0_to_1 { 1.0 } else { 0.0 };
            let dither_normal = lum * (1.0 - blend_lambda) + on * blend_lambda;
            let dither = (dither_normal * 255.0).round() as u8;
            pixels_rgba8[pixel_index + 0] = dither;
            pixels_rgba8[pixel_index + 1] = dither;
            pixels_rgba8[pixel_index + 2] = dither;
            pixels_rgba8[pixel_index + 3] = 255;
        }
    }
}

#[inline]
fn luminance(r: u8, g: u8, b: u8) -> f32 {
    (0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32) / 255.0
}

// --- tiny 4x4 PS1-ish Bayer ranks (binary mask) ---
const N: i32 = 4;
const BAYER4: [[u8; 4]; 4] = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]];

/// How many discrete coverage levels to approximate (more = smoother fade, still cheap)
pub const STIPPLE_LEVELS: i32 = 17; // 0..16 ≈ 5-bit like PS1 feel

pub fn build_stipple_atlas_rgba() -> Image {
    // Atlas is Nx(N*L) where each NxN slice is the mask for one level
    let w = N;
    let h = N * STIPPLE_LEVELS;
    let mut img = Image::gen_image_color(w, h, Color::BLANK);
    let total_bytes: usize = (w * h * 4) as usize;
    let px: &mut [u8] = unsafe { from_raw_parts_mut(img.data as *mut u8, total_bytes) };

    for level in 0..STIPPLE_LEVELS {
        // Threshold is the number of "on" ranks we allow for this level
        // level==0 -> nothing on (fully transparent), level==16 -> mostly on
        for y in 0..N {
            for x in 0..N {
                let idx = 4 * (((level * N + y) * w + x) as usize);
                let on = (BAYER4[y as usize][x as usize] as i32) < level;
                // White color, alpha 0/255 as ON/OFF mask
                px[idx + 0] = 255;
                px[idx + 1] = 255;
                px[idx + 2] = 255;
                px[idx + 3] = if on { 255 } else { 0 };
            }
        }
    }
    img.set_format(PIXELFORMAT_UNCOMPRESSED_R8G8B8A8);
    img
}
pub fn rotate_silhouette_texture_stipple_screen_locked(
    model: &mut Model,
    observer: &Camera3D,
    mesh_rotation: f32,
    screen_w: i32,
    screen_h: i32,
) {
    let mesh = &mut model.meshes_mut()[0];
    let mut vertices = mesh.vertices_mut().to_vec();
    let mut smooth_vertex_normals = {
        let mut topo = topology_init(mesh);
        collect_vertex_normals(&mut topo, mesh);
        smooth_vertex_normals(&topo)
    };
    rotate_vertices(&mut vertices, mesh_rotation);
    rotate_vertices(&mut smooth_vertex_normals, mesh_rotation);
    let los = observed_line_of_sight(observer);
    let vertex_count = mesh.vertexCount as usize;
    if mesh.as_mut().texcoords.is_null() {
        let tex = vec![0.0f32; vertex_count * 2];
        mesh.as_mut().texcoords = Box::leak(tex.into_boxed_slice()).as_mut_ptr();
    }
    let texcoords = unsafe { from_raw_parts_mut(mesh.as_mut().texcoords, vertex_count * 2) };
    let world_to_px = (screen_h as f32) / FOVY;
    let half_w = (screen_w as f32) * 0.5;
    let half_h = (screen_h as f32) * 0.5;

    for i in 0..vertex_count {
        let vtx = vertices[i];
        let x_px = (vtx.x * world_to_px + half_w).floor() as i32;
        let y_px = (vtx.y * world_to_px + half_h).floor() as i32;
        let n = smooth_vertex_normals[i];
        let align = n.dot(los).abs();
        let t = 1.0 - smoothstep(ALPHA_FADE_RAMP_MIN, ALPHA_FADE_RAMP_MAX, align);
        let coverage = (1.0 - t).clamp(0.0, 1.0);
        let level_f = (coverage * (STIPPLE_LEVELS as f32 - 1.0)).round();
        let level = level_f as i32;
        let u = ((x_px & (N - 1)) as f32 + 0.5) / (N as f32);
        let v = ((level * N + (y_px & (N - 1))) as f32 + 0.5) / ((STIPPLE_LEVELS * N) as f32);
        texcoords[i * 2 + 0] = u;
        texcoords[i * 2 + 1] = v;
    }
}
