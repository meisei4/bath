use crate::fixed_func::silhouette_constants::{
    ANGULAR_VELOCITY, GRID_ORIGIN_UV_OFFSET, GRID_SCALE, LIGHT_WAVE_AMPLITUDE_X, LIGHT_WAVE_AMPLITUDE_Y,
    LIGHT_WAVE_SPATIAL_FREQ_X, LIGHT_WAVE_SPATIAL_FREQ_Y, LIGHT_WAVE_TEMPORAL_FREQ_X, LIGHT_WAVE_TEMPORAL_FREQ_Y,
    ROTATIONAL_SAMPLES_FOR_INV_PROJ, SILHOUETTE_RADII_RESOLUTION, TEXTURE_MAPPING_BOUNDARY_FADE, TIME_BETWEEN_SAMPLES,
    UMBRAL_MASK_CENTER, UMBRAL_MASK_FADE_BAND, UMBRAL_MASK_OUTER_RADIUS,
};

use crate::fixed_func::silhouette_interpolation::interpolate_radial_magnitude_from_sample_xy;
use crate::render::raylib::RaylibRenderer;
use asset_payload::SPHERE_PATH;
use raylib::camera::Camera3D;
use raylib::color::Color;
use raylib::consts::PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8;
use raylib::drawing::{RaylibDraw, RaylibDraw3D, RaylibDrawHandle, RaylibMode3DExt};
use raylib::math::{Vector2, Vector3};
use raylib::models::{RaylibMesh, RaylibModel, WeakMesh};
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
fn sample_lum(pixels: &[u8], w: usize, x: i32, y: i32) -> u8 {
    if x < 0 || y < 0 || x as usize >= w || y as usize >= pixels.len() / (4 * w) {
        0
    } else {
        let i = 4 * (y as usize * w + x as usize);
        pixels[i]
    }
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

//TODO: this isnt that useful
pub fn debug_uv_seams(
    observer: Camera3D,
    draw_handle: &mut RaylibDrawHandle,
    mesh: &WeakMesh,
    rotation: f32,
    v_rim_min: f32,
    max_logs: usize,
) -> usize {
    let vertex_count = mesh.vertexCount as usize;
    let triangle_count = mesh.triangleCount as usize;

    if vertex_count == 0 || triangle_count == 0 || mesh.vertices.is_null() || mesh.texcoords.is_null() {
        return 0;
    }
    let vertices: &[f32] = unsafe { from_raw_parts(mesh.vertices, vertex_count * 3) };
    let texcoords: &[f32] = unsafe { from_raw_parts(mesh.texcoords, vertex_count * 2) };
    let indexed = !mesh.indices.is_null();
    let indices: &[u16] = unsafe {
        if indexed {
            from_raw_parts(mesh.indices, triangle_count * 3)
        } else {
            &[]
        }
    };
    let screen_width = draw_handle.get_screen_width() as f32;
    let screen_height = draw_handle.get_screen_height() as f32;
    let cos_theta = rotation.cos();
    let sin_theta = rotation.sin();
    let mut logs_left = max_logs;
    log_header(triangle_count, vertex_count, indexed, &mut logs_left);
    let mut artifacts = 0usize;
    for triangle_index in 0..triangle_count {
        let (index_a, index_b, index_c) = indices_for_triangle(indexed, indices, triangle_index);
        let (u0, v0) = uv_at(texcoords, index_a);
        let (u1, v1) = uv_at(texcoords, index_b);
        let (u2, v2) = uv_at(texcoords, index_c);
        if !tri_crosses_u_wrap(u0, u1, u2) {
            continue;
        }
        if v_rim_min > 0.0 && v0.max(v1.max(v2)) < v_rim_min {
            continue;
        }
        let a_rot = rotate_y(vertex_at(vertices, index_a), cos_theta, sin_theta);
        let b_rot = rotate_y(vertex_at(vertices, index_b), cos_theta, sin_theta);
        let c_rot = rotate_y(vertex_at(vertices, index_c), cos_theta, sin_theta);
        {
            let mut rl3d = draw_handle.begin_mode3D(observer);
            rl3d.draw_triangle3D(a_rot, b_rot, c_rot, Color::new(255, 32, 32, 110));
        }
        let center = (a_rot + b_rot + c_rot) / 3.0;
        let screen_x = ((center.x) * 0.5 + 0.5) * screen_width;
        let screen_y = ((-center.y) * 0.5 + 0.5) * screen_height;
        draw_handle.draw_text(
            &triangle_index.to_string(),
            screen_x as i32,
            screen_y as i32,
            14,
            Color::RED,
        );
        log_triangle(
            triangle_index,
            indexed,
            index_a,
            index_b,
            index_c,
            u0,
            v0,
            u1,
            v1,
            u2,
            v2,
            &mut logs_left,
        );
        artifacts += 1;
    }
    if artifacts == 0 {
        println!("[UV-SEAM] no triangles matched (try lowering v_rim_min, e.g. 0.9)");
    }
    artifacts
}

#[inline]
fn rotate_y(v: Vector3, cos_theta: f32, sin_theta: f32) -> Vector3 {
    let x = v.x * cos_theta + v.z * sin_theta;
    let z = -v.x * sin_theta + v.z * cos_theta;
    Vector3::new(x, v.y, z)
}

#[inline]
fn vertex_at(vertices: &[f32], index: usize) -> Vector3 {
    let base = index * 3;
    Vector3::new(vertices[base + 0], vertices[base + 1], vertices[base + 2])
}

#[inline]
fn uv_at(texcoords: &[f32], index: usize) -> (f32, f32) {
    let base = index * 2;
    (texcoords[base + 0], texcoords[base + 1])
}

#[inline]
fn indices_for_triangle(indexed: bool, indices: &[u16], triangle_index: usize) -> (usize, usize, usize) {
    if indexed {
        let base = triangle_index * 3;
        (
            indices[base + 0] as usize,
            indices[base + 1] as usize,
            indices[base + 2] as usize,
        )
    } else {
        (triangle_index * 3 + 0, triangle_index * 3 + 1, triangle_index * 3 + 2)
    }
}

#[inline]
fn tri_crosses_u_wrap(u0: f32, u1: f32, u2: f32) -> bool {
    let u0 = u0.rem_euclid(1.0);
    let u1 = u1.rem_euclid(1.0);
    let u2 = u2.rem_euclid(1.0);
    let min_u = u0.min(u1.min(u2));
    let max_u = u0.max(u1.max(u2));
    (max_u - min_u) > 0.5
}

fn log_header(triangle_count: usize, vertex_count: usize, indexed: bool, logs_left: &mut usize) {
    if *logs_left > 0 {
        println!(
            "[UV-SEAM] tri_count={} vtx_count={} indexed={}",
            triangle_count, vertex_count, indexed
        );
        *logs_left -= 1;
    }
}

fn log_triangle(
    triangle_index: usize,
    indexed: bool,
    index_a: usize,
    index_b: usize,
    index_c: usize,
    u0: f32,
    v0: f32,
    u1: f32,
    v1: f32,
    u2: f32,
    v2: f32,
    logs_left: &mut usize,
) {
    if *logs_left == 0 {
        return;
    }
    let u0_wrapped = u0.rem_euclid(1.0);
    let u1_wrapped = u1.rem_euclid(1.0);
    let u2_wrapped = u2.rem_euclid(1.0);
    let min_u = u0_wrapped.min(u1_wrapped.min(u2_wrapped));
    let max_u = u0_wrapped.max(u1_wrapped.max(u2_wrapped));
    let span_u = max_u - min_u;
    if indexed {
        println!(
            "[UV-SEAM] tri={} idx=({},{},{}) U=({:.3},{:.3},{:.3}) V=({:.3},{:.3},{:.3}) spanU={:.3}",
            triangle_index, index_a, index_b, index_c, u0, u1, u2, v0, v1, v2, span_u
        );
    } else {
        println!(
            "[UV-SEAM] tri={} (soup) U=({:.3},{:.3},{:.3}) V=({:.3},{:.3},{:.3}) spanU={:.3}",
            triangle_index, u0, u1, u2, v0, v1, v2, span_u
        );
    }
    *logs_left -= 1;
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

///TODO :this is the old way??????
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
            let interpolated_radial_magnitude =
                interpolate_radial_magnitude_from_sample_xy(vertex.x, vertex.y, &radii_normals);
            let u = vertex.x / interpolated_radial_magnitude * 0.5 + 0.5;
            let v = vertex.y / interpolated_radial_magnitude * 0.5 + 0.5;
            texcoord_sample.push(u);
            texcoord_sample.push(v);
        }
        texcoord_samples.push(texcoord_sample);
        rotate_vertices(&mut mesh_sample, -frame_rotation);
        mesh_samples.push(mesh_sample);
    }
    (mesh_samples, texcoord_samples)
}

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
