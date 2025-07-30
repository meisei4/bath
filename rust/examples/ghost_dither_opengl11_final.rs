use asset_payload::SPHERE_PATH;
use bath::render::raylib::RaylibRenderer;
use bath::render::raylib_util::N64_WIDTH;
use bath::render::renderer::Renderer;
use raylib::camera::Camera3D;
use raylib::color::Color;
use raylib::consts::CameraProjection;

use bath::fixed_func::ghost::{deform_mesh_by_phase, deform_mesh_with_yaw, precompute_thetas};
use raylib::drawing::{RaylibDraw, RaylibDraw3D, RaylibMode3DExt};
use raylib::math::{Vector2, Vector3};
use raylib::models::{RaylibMesh, RaylibModel, WeakMesh};
use raylib::prelude::Image;
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

const LIGHT_WAVE_SPATIAL_FREQ_X: f32 = 8.0;
const LIGHT_WAVE_SPATIAL_FREQ_Y: f32 = 8.0;
const LIGHT_WAVE_TEMPORAL_FREQ_X: f32 = 80.0;
const LIGHT_WAVE_TEMPORAL_FREQ_Y: f32 = 2.3;
const LIGHT_WAVE_AMPLITUDE_X: f32 = 0.0;
const LIGHT_WAVE_AMPLITUDE_Y: f32 = 0.1;
const UMBRAL_MASK_OUTER_RADIUS: f32 = 0.40;
const UMBRAL_MASK_FADE_BAND: f32 = 0.025;
const UMBRAL_MASK_CENTER: Vector2 = Vector2::new(HALF, HALF);

const ROTATION_FREQUENCY_HZ: f32 = 0.25;
const ANGULAR_VELOCITY: f32 = TAU * ROTATION_FREQUENCY_HZ;
const MODEL_POS: Vector3 = Vector3::ZERO;
const MODEL_SCALE: Vector3 = Vector3::ONE;

const DT: f32 = 0.1;
const NUM_SAMPLES: usize = 80;
fn main() {
    let mut i_time = 0.0f32;
    let mut mesh_rotation = 0.0f32;
    let mut render = RaylibRenderer::init(N64_WIDTH, N64_WIDTH);

    let observer = Camera3D {
        position: Vector3::new(0.0, 0.0, 2.0),
        target: Vector3::ZERO,
        up: Vector3::Y,
        fovy: 2.0,
        projection: CameraProjection::CAMERA_ORTHOGRAPHIC,
    };
    let screen_w = render.handle.get_screen_width();
    let screen_h = render.handle.get_screen_height();
    let mut model = render.handle.load_model(&render.thread, SPHERE_PATH).unwrap();
    let silhouette_img = generate_silhouette_image(screen_w, screen_h, i_time);
    let silhouette_radii = build_radial_magnitudes(&silhouette_img);

    //normal deform
    deform_mesh_by_silhouette_radii(&mut model.meshes_mut()[0], &silhouette_radii);

    let r0 = build_radial_magnitudes(&silhouette_img);
    let base_vertices = model.meshes()[0].vertices().to_vec();
    let thetas = precompute_thetas(&model.meshes()[0]);
    let phase = (ANGULAR_VELOCITY * i_time) % TAU;
    model.meshes_mut()[0].vertices_mut().copy_from_slice(&base_vertices);
    // phase deform
    deform_mesh_by_phase(&mut model.meshes_mut()[0], &r0.try_into().unwrap(), &thetas, phase);

    let yaw_rad = -i_time * 90.0f32.to_radians();
    //yaw deform
    deform_mesh_with_yaw(&mut model.meshes_mut()[0], &silhouette_radii, yaw_rad);

    let mut sample_vertices = Vec::with_capacity(NUM_SAMPLES);
    for i in 0..NUM_SAMPLES {
        let sample_time = i as f32 * DT;
        let sample_yaw = -sample_time * ANGULAR_VELOCITY;
        let silhouette_img = generate_silhouette_image(screen_w, screen_h, sample_time);
        let silhouette_radii = build_radial_magnitudes(&silhouette_img);
        let mut model_i = render.handle.load_model(&render.thread, SPHERE_PATH).unwrap();
        {
            let vertices = model_i.meshes_mut()[0].vertices_mut();
            for vertex in vertices.iter_mut() {
                let x0 = vertex.x;
                let z0 = vertex.z;
                vertex.x = sample_yaw.cos() * x0 + sample_yaw.sin() * z0;
                vertex.z = -sample_yaw.sin() * x0 + sample_yaw.cos() * z0;
            }
        }
        //normal deform
        deform_mesh_by_silhouette_radii(&mut model_i.meshes_mut()[0], &silhouette_radii);
        let sample_vertices_i: Vec<Vector3> = model_i.meshes()[0].vertices().iter().cloned().collect();
        sample_vertices.push(sample_vertices_i);
    }

    while !render.handle.window_should_close() {
        i_time += render.handle.get_frame_time();
        // mesh_rotation += ANGULAR_VELOCITY * render.handle.get_frame_time();
        mesh_rotation = 0.0;
        let sample_idx = ((i_time / DT).floor() as usize) % NUM_SAMPLES;
        let sample_vertices_src = &sample_vertices[sample_idx];
        let sample_vertices_dst = model.meshes_mut()[0].vertices_mut();
        for (dst, src) in sample_vertices_dst.iter_mut().zip(sample_vertices_src.iter()) {
            *dst = *src;
        }
        let mut draw_handle = render.handle.begin_drawing(&render.thread);
        draw_handle.clear_background(Color::BLACK);
        let mut rl3d = draw_handle.begin_mode3D(observer);
        rl3d.draw_model_wires_ex(
            &model,
            MODEL_POS,
            Vector3::Y,
            mesh_rotation.to_degrees(),
            MODEL_SCALE,
            Color::WHITE,
        );
    }
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

const RADIAL_SAMPLE_COUNT: usize = 32;
fn build_radial_magnitudes(source_image: &Image) -> Vec<f32> {
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
