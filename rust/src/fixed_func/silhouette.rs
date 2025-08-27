use crate::fixed_func::silhouette_constants::{
    ANGULAR_VELOCITY, ROTATIONAL_SAMPLES_FOR_INV_PROJ, SILHOUETTE_RADII_RESOLUTION, TIME_BETWEEN_SAMPLES,
};
use std::collections::HashMap;

use crate::fixed_func::papercraft::{face_normal, WeldedEdge, WeldedVertex};
use crate::fixed_func::silhouette_interpolation::interpolate_radial_magnitude_from_sample_xy;
use crate::fixed_func::silhouette_util::{rotate_vertices, silhouette_radius_at_angle, silhouette_uvs_polar};
use crate::render::raylib::RaylibRenderer;
use asset_payload::SPHERE_PATH;
use raylib::camera::Camera3D;
use raylib::color::Color;
use raylib::consts::PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8;
use raylib::math::glam::{Vec2, Vec3};
use raylib::math::Vector3;
use raylib::models::{Model, RaylibMesh, RaylibModel};
use raylib::texture::{Image, Texture2D};
use std::f32::consts::TAU;
use std::slice::from_raw_parts_mut;

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

pub fn compute_silhouette_feather_ortho(
    model: &mut Model,
    observer: &Camera3D,
    width: i32,
    height: i32,
    fade_width: f32,
) {
    let (triangle_indices_per_face, vertex_positions_world, vertex_count) = {
        let mesh = &model.meshes()[0];
        let vertices = mesh.vertices().to_vec();
        let vertex_count = vertices.len();
        let indices: Vec<[u32; 3]> = if mesh.indices().is_empty() {
            debug_assert!(vertex_count % 3 == 0, "Non-indexed mesh must be triangle soup");
            (0..(vertex_count / 3) as u32)
                .map(|triangle_number| [3 * triangle_number, 3 * triangle_number + 1, 3 * triangle_number + 2])
                .collect()
        } else {
            mesh.indices()
                .chunks_exact(3)
                .map(|chunk| [chunk[0] as u32, chunk[1] as u32, chunk[2] as u32])
                .collect()
        };
        (indices, vertices, vertex_count)
    };
    if vertex_count == 0 || triangle_indices_per_face.is_empty() {
        return;
    }
    let observer_line_of_sight = (Vec3::new(observer.target.x, observer.target.y, observer.target.z)
        - Vec3::new(observer.position.x, observer.position.y, observer.position.z))
    .normalize_or_zero();

    let mut facing_observer_signum_per_face = Vec::with_capacity(triangle_indices_per_face.len());
    for triangle_indices in &triangle_indices_per_face {
        let normal = face_normal(
            vertex_positions_world[triangle_indices[0] as usize],
            vertex_positions_world[triangle_indices[1] as usize],
            vertex_positions_world[triangle_indices[2] as usize],
        );
        facing_observer_signum_per_face.push(normal.dot(observer_line_of_sight));
    }
    let mut welded_edge_to_first_face_index: HashMap<WeldedEdge, usize> = HashMap::new();
    for (face_index, triangle_indices) in triangle_indices_per_face.iter().enumerate() {
        let face_edges = [
            (triangle_indices[0], triangle_indices[1]),
            (triangle_indices[1], triangle_indices[2]),
            (triangle_indices[2], triangle_indices[0]),
        ];
        for (vertex_index_a_u32, vertex_index_b_u32) in face_edges {
            let welded_edge = WeldedEdge::new(
                WeldedVertex { id: vertex_index_a_u32 },
                WeldedVertex { id: vertex_index_b_u32 },
            );
            welded_edge_to_first_face_index.entry(welded_edge).or_insert(face_index);
        }
    }

    let mut screen_space_position_per_vertex = vec![Vec2::ZERO; vertex_count];
    for vertex_index in 0..vertex_count {
        screen_space_position_per_vertex[vertex_index] =
            orthographic_project_world_to_screen(vertex_positions_world[vertex_index], observer, width, height);
    }
    let mut front_faces_to_silhouette_segments: HashMap<usize, Vec<(Vec2, Vec2)>> = HashMap::new();
    for (face_index, triangle_indices) in triangle_indices_per_face.iter().enumerate() {
        let face_edges_usize = [
            (triangle_indices[0] as usize, triangle_indices[1] as usize),
            (triangle_indices[1] as usize, triangle_indices[2] as usize),
            (triangle_indices[2] as usize, triangle_indices[0] as usize),
        ];

        for (vertex_index_a, vertex_index_b) in face_edges_usize {
            let welded_edge = WeldedEdge::new(
                WeldedVertex {
                    id: vertex_index_a as u32,
                },
                WeldedVertex {
                    id: vertex_index_b as u32,
                },
            );

            if let Some(&adjacent_face_index) = welded_edge_to_first_face_index.get(&welded_edge) {
                if adjacent_face_index == face_index {
                    continue;
                }
                let sign_current = facing_observer_signum_per_face[face_index];
                let sign_adjacent = facing_observer_signum_per_face[adjacent_face_index];
                if sign_current * sign_adjacent < 0.0 {
                    let front_facing_face_index = if sign_current >= 0.0 {
                        face_index
                    } else {
                        adjacent_face_index
                    };

                    let screen_a = screen_space_position_per_vertex[vertex_index_a];
                    let screen_b = screen_space_position_per_vertex[vertex_index_b];
                    front_faces_to_silhouette_segments
                        .entry(front_facing_face_index)
                        .or_default()
                        .push((screen_a, screen_b));
                }
            } else {
                if facing_observer_signum_per_face[face_index] >= 0.0 {
                    let screen_a = screen_space_position_per_vertex[vertex_index_a];
                    let screen_b = screen_space_position_per_vertex[vertex_index_b];
                    front_faces_to_silhouette_segments
                        .entry(face_index)
                        .or_default()
                        .push((screen_a, screen_b));
                }
            }
        }
    }
    if front_faces_to_silhouette_segments.is_empty() {
        let mesh_mutable = &mut model.meshes_mut()[0];
        unsafe {
            let texcoords_slice = from_raw_parts_mut(mesh_mutable.as_mut().texcoords, vertex_count * 2);
            for vertex_index in 0..vertex_count {
                texcoords_slice[2 * vertex_index + 0] = 0.5;
                texcoords_slice[2 * vertex_index + 1] = 0.0;
            }
        }
        return;
    }
    let mut feather_value_per_vertex = vec![0.0_f32; vertex_count];
    for (front_face_index, silhouette_segments_for_face) in &front_faces_to_silhouette_segments {
        let triangle_indices = triangle_indices_per_face[*front_face_index];
        for vertex_index in triangle_indices {
            let screen_position = screen_space_position_per_vertex[vertex_index as usize];

            let mut minimum_distance_in_pixels = f32::INFINITY;
            for (segment_a, segment_b) in silhouette_segments_for_face {
                let distance = pixel_distance_point_to_segment(screen_position, *segment_a, *segment_b);
                if distance < minimum_distance_in_pixels {
                    minimum_distance_in_pixels = distance;
                }
            }

            let feather_coordinate = (1.0 - minimum_distance_in_pixels / fade_width).clamp(0.0, 1.0);
            if feather_coordinate > feather_value_per_vertex[vertex_index as usize] {
                feather_value_per_vertex[vertex_index as usize] = feather_coordinate;
            }
        }
    }

    let mesh_mutable = &mut model.meshes_mut()[0];
    unsafe {
        let texcoords_slice = from_raw_parts_mut(mesh_mutable.as_mut().texcoords, vertex_count * 2);
        for vertex_index in 0..vertex_count {
            texcoords_slice[2 * vertex_index + 0] = 0.5;
            texcoords_slice[2 * vertex_index + 1] = feather_value_per_vertex[vertex_index];
        }
    }
}

#[inline]
fn orthographic_extents_from_camera(fovy: f32, aspect_ratio: f32) -> (f32, f32, f32, f32) {
    let top = fovy;
    let bottom = -fovy;
    let right = fovy * aspect_ratio;
    let left = -right;
    (left, right, bottom, top)
}

#[inline]
fn orthographic_project_world_to_screen(world_position: Vec3, observer: &Camera3D, width: i32, height: i32) -> Vec2 {
    let aspect_ratio = width as f32 / height as f32;
    let (left, right, bottom, top) = orthographic_extents_from_camera(observer.fovy, aspect_ratio);

    let eye_space = world_position - Vec3::new(observer.position.x, observer.position.y, observer.position.z);

    let normalized_x = (eye_space.x - left) / (right - left);
    let normalized_y = (top - eye_space.y) / (top - bottom);
    Vec2::new(normalized_x * width as f32, normalized_y * height as f32)
}

#[inline]
fn pixel_distance_point_to_segment(c: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    let ac = c - a;
    let denominator = ab.length_squared().max(1e-12);
    let t = (ac.dot(ab) / denominator).clamp(0.0, 1.0);
    (ac - ab * t).length()
}
