use crate::fixed_func::constants::{
    ANGULAR_VELOCITY, GRID_ORIGIN_UV_OFFSET, GRID_SCALE, LIGHT_WAVE_AMPLITUDE_X, LIGHT_WAVE_AMPLITUDE_Y,
    LIGHT_WAVE_SPATIAL_FREQ_X, LIGHT_WAVE_SPATIAL_FREQ_Y, LIGHT_WAVE_TEMPORAL_FREQ_X, LIGHT_WAVE_TEMPORAL_FREQ_Y,
    ROTATIONAL_SAMPLES_FOR_INV_PROJ, SILHOUETTE_RADII_RESOLUTION, TEXTURE_MAPPING_BOUNDARY_FADE, TIME_BETWEEN_SAMPLES,
    UMBRAL_MASK_CENTER, UMBRAL_MASK_FADE_BAND, UMBRAL_MASK_OUTER_RADIUS,
};

use crate::render::raylib::RaylibRenderer;
use asset_payload::SPHERE_PATH;
use raylib::camera::Camera3D;
use raylib::color::Color;
use raylib::consts::PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8;
use raylib::drawing::{RaylibDraw, RaylibDraw3D, RaylibDrawHandle, RaylibMode3DExt};
use raylib::math::{Vector2, Vector3};
use raylib::models::{Model, RaylibMesh, RaylibModel, WeakMesh};
use raylib::texture::{Image, Texture2D};
use std::f32::consts::TAU;
use std::slice::{from_raw_parts, from_raw_parts_mut};

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
    for vertex in vertices.iter_mut() {
        let interpolated_radial_magnitude =
            interpolate_radial_magnitude_from_sample_xy(vertex.x, vertex.y, radii_normals);
        vertex.x *= interpolated_radial_magnitude;
        vertex.y *= interpolated_radial_magnitude;
    }
}

pub fn generate_mesh_and_texcoord_samples_from_silhouette(
    renderer: &mut RaylibRenderer,
) -> (Vec<Vec<Vector3>>, Vec<Vec<f32>>) {
    let screen_w = renderer.handle.get_screen_width();
    let screen_h = renderer.handle.get_screen_height();
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

pub fn rotate_vertices(vertices: &mut Vec<Vector3>, rotation: f32) {
    for vertex in vertices {
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

pub fn debug_indices(observer: Camera3D, draw_handle: &mut RaylibDrawHandle, mesh: &WeakMesh, rotation: f32) {
    let triangle_count = mesh.triangleCount as usize;
    let indices = unsafe { from_raw_parts(mesh.indices, triangle_count * 3) };
    let vertices = unsafe { from_raw_parts(mesh.vertices, mesh.vertexCount as usize * 3) };
    let screen_w = draw_handle.get_screen_width();
    let screen_h = draw_handle.get_screen_height();
    for i in 0..triangle_count {
        let ia = indices[i * 3] as usize;
        let ib = indices[i * 3 + 1] as usize;
        let ic = indices[i * 3 + 2] as usize;

        let mut vertex_a = Vector3::new(vertices[ia * 3], vertices[ia * 3 + 1], vertices[ia * 3 + 2]);
        let mut vertex_b = Vector3::new(vertices[ib * 3], vertices[ib * 3 + 1], vertices[ib * 3 + 2]);
        let mut vertex_c = Vector3::new(vertices[ic * 3], vertices[ic * 3 + 1], vertices[ic * 3 + 2]);

        let (x0, z0) = (vertex_a.x, vertex_a.z);
        vertex_a.x = x0 * rotation.cos() + z0 * rotation.sin();
        vertex_a.z = -x0 * rotation.sin() + z0 * rotation.cos();

        let (x0, z0) = (vertex_b.x, vertex_b.z);
        vertex_b.x = x0 * rotation.cos() + z0 * rotation.sin();
        vertex_b.z = -x0 * rotation.sin() + z0 * rotation.cos();

        let (x0, z0) = (vertex_c.x, vertex_c.z);
        vertex_c.x = x0 * rotation.cos() + z0 * rotation.sin();
        vertex_c.z = -x0 * rotation.sin() + z0 * rotation.cos();

        let color = Color::new(
            (i.wrapping_mul(60) & 255) as u8,
            (i.wrapping_mul(120) & 255) as u8,
            (i.wrapping_mul(240) & 255) as u8,
            255,
        );
        let centroid = (vertex_a + vertex_b + vertex_c) / 3.0;
        let centroid_x = ((centroid.x) * 0.5 + 0.5) * screen_w as f32;
        let centroid_y = ((-centroid.y) * 0.5 + 0.5) * screen_h as f32;
        {
            let mut rl3d = draw_handle.begin_mode3D(observer);
            rl3d.draw_triangle3D(vertex_a, vertex_b, vertex_c, color);
        }
        draw_handle.draw_text(&i.to_string(), centroid_x as i32, centroid_y as i32, 12, Color::WHITE);
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
