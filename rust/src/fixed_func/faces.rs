use crate::fixed_func::silhouette_geometry_util::rotate_vertices;
use crate::fixed_func::welding::{collect_welded_vertices, face_normal, WeldedEdge, WeldedVertex};
use raylib::camera::Camera3D;
use raylib::color::Color;
use raylib::drawing::{RaylibDraw, RaylibDraw3D, RaylibDrawHandle, RaylibMode3DExt};
use raylib::math::glam::Vec3;
use raylib::models::{Model, RaylibMesh, RaylibModel, WeakMesh};
use std::collections::HashSet;

pub fn collect_faces(mesh: &WeakMesh) -> Vec<[u32; 3]> {
    let faces: Vec<[u32; 3]> = mesh
        .indices()
        .chunks_exact(3)
        .map(|chunk| [chunk[0] as u32, chunk[1] as u32, chunk[2] as u32])
        .collect();
    faces
}
pub fn collect_front_faces(model: &Model, mesh_rotation: f32, observer: &Camera3D) -> HashSet<usize> {
    let mesh = &model.meshes()[0];
    let line_of_sight = observed_line_of_sight(observer);
    let vertices = mesh.vertices();
    let all_faces = collect_faces(mesh);

    let mut front_faces_set = HashSet::with_capacity(all_faces.len());
    for (face_id, [vertex_a_index, vertex_b_index, vertex_c_index]) in all_faces.iter().copied().enumerate() {
        let mut face = vec![
            vertices[vertex_a_index as usize],
            vertices[vertex_b_index as usize],
            vertices[vertex_c_index as usize],
        ];
        rotate_vertices(&mut face, mesh_rotation);
        let face_normal = face_normal(face[0], face[1], face[2]);
        if face_normal.dot(line_of_sight) <= 0.0 {
            front_faces_set.insert(face_id);
        }
    }
    front_faces_set
}

pub fn collect_back_faces(model: &Model, mesh_rotation: f32, observer: &Camera3D) -> HashSet<usize> {
    let mesh = &model.meshes()[0];
    let triangle_count = mesh.triangleCount as usize;
    let front_faces = collect_front_faces(model, mesh_rotation, observer);
    (0..triangle_count).filter(|face| !front_faces.contains(face)).collect()
}

pub fn collect_silhouette_faces(model: &Model, mesh_rotation: f32, observer: &Camera3D) -> Vec<usize> {
    let mesh = &model.meshes()[0];
    let vertices = mesh.vertices();
    let all_faces = collect_faces(mesh);
    let front_faces = collect_front_faces(model, mesh_rotation, observer);
    let welded_vertices = collect_welded_vertices(vertices);

    let mut welded_vertices_per_face: Vec<[WeldedVertex; 3]> = Vec::with_capacity(all_faces.len());
    for [vertex_a_index, vertex_b_index, vertex_c_index] in all_faces.iter().copied() {
        let welded_vertex_a = WeldedVertex {
            id: welded_vertices[vertex_a_index as usize],
        };
        let welded_vertex_b = WeldedVertex {
            id: welded_vertices[vertex_b_index as usize],
        };
        let welded_vertex_c = WeldedVertex {
            id: welded_vertices[vertex_c_index as usize],
        };
        welded_vertices_per_face.push([welded_vertex_a, welded_vertex_b, welded_vertex_c]);
    }
    let mut neighbor_per_face: Vec<[Option<usize>; 3]> = vec![[None, None, None]; all_faces.len()];
    let mut first_owner_per_edge: std::collections::HashMap<WeldedEdge, (usize, u8)> = std::collections::HashMap::new();
    for (face_id, welded_triplet) in welded_vertices_per_face.iter().copied().enumerate() {
        let edges: [(WeldedEdge, u8); 3] = [
            (WeldedEdge::new(welded_triplet[0], welded_triplet[1]), 0),
            (WeldedEdge::new(welded_triplet[1], welded_triplet[2]), 1),
            (WeldedEdge::new(welded_triplet[2], welded_triplet[0]), 2),
        ];
        for (welded_edge, local_edge_index) in edges {
            if let Some(&(other_face_id, other_local_edge_index)) = first_owner_per_edge.get(&welded_edge) {
                neighbor_per_face[face_id][local_edge_index as usize] = Some(other_face_id);
                neighbor_per_face[other_face_id][other_local_edge_index as usize] = Some(face_id);
            } else {
                first_owner_per_edge.insert(welded_edge, (face_id, local_edge_index));
            }
        }
    }
    let mut silhouette_faces: HashSet<usize> = HashSet::new();
    for (face_id, welded_triplet) in welded_vertices_per_face.iter().copied().enumerate() {
        let local_edges: [(usize, usize, u8); 3] = [(0, 1, 0), (1, 2, 1), (2, 0, 2)];
        for (vertex_0, vertex_1, local_edge_index) in local_edges {
            let neighbor_face_opt = neighbor_per_face[face_id][local_edge_index as usize];
            if neighbor_face_opt.is_none() {
                continue;
            }
            let neighbor_face_id = neighbor_face_opt.unwrap();
            let is_front_here = front_faces.contains(&face_id);
            let is_front_neighbor = front_faces.contains(&neighbor_face_id);
            if is_front_here == is_front_neighbor {
                continue;
            }
            let rim_face_id = if is_front_here { face_id } else { neighbor_face_id };
            silhouette_faces.insert(rim_face_id);
            let rim_welded_triplet = welded_vertices_per_face[rim_face_id];
            let silhouette_edge_w0 = welded_triplet[vertex_0];
            let silhouette_edge_w1 = welded_triplet[vertex_1];
            let silhouette_edge_key = WeldedEdge::new(silhouette_edge_w0, silhouette_edge_w1);
            let rim_local_edge_index = {
                let test_edges = [
                    (WeldedEdge::new(rim_welded_triplet[0], rim_welded_triplet[1]), 0u8),
                    (WeldedEdge::new(rim_welded_triplet[1], rim_welded_triplet[2]), 1u8),
                    (WeldedEdge::new(rim_welded_triplet[2], rim_welded_triplet[0]), 2u8),
                ];
                let mut found: u8 = 0;
                for (edge_key, idx) in test_edges {
                    if edge_key == silhouette_edge_key {
                        found = idx;
                        break;
                    }
                }
                found
            };
            let interior_edge_index_0 = ((rim_local_edge_index as i32 + 1) % 3) as usize;
            let interior_edge_index_1 = ((rim_local_edge_index as i32 + 2) % 3) as usize;

            if let Some(neighbor_0) = neighbor_per_face[rim_face_id][interior_edge_index_0] {
                if front_faces.contains(&neighbor_0) {
                    silhouette_faces.insert(neighbor_0);
                }
            }
            if let Some(neighbor_1) = neighbor_per_face[rim_face_id][interior_edge_index_1] {
                if front_faces.contains(&neighbor_1) {
                    silhouette_faces.insert(neighbor_1);
                }
            }
        }
    }
    silhouette_faces.into_iter().collect()
}

pub fn debug_draw_faces(
    observer: Camera3D,
    draw_handle: &mut RaylibDrawHandle,
    mesh: &WeakMesh,
    rotation: f32,
    faces: &[usize],
    fill_color: Option<Color>,
    label: bool,
) {
    let vertices = mesh.vertices();
    if vertices.is_empty() {
        return;
    }
    let all_faces = collect_faces(mesh);
    for &face_id in faces {
        if face_id >= all_faces.len() {
            continue;
        }
        let [vertex_a_index, vertex_b_index, vertex_c_index] = all_faces[face_id];
        let mut face = vec![
            vertices[vertex_a_index as usize],
            vertices[vertex_b_index as usize],
            vertices[vertex_c_index as usize],
        ];
        rotate_vertices(&mut face, rotation);
        let (vertex_a, vertex_b, vertex_c) = (face[0], face[1], face[2]);
        let color = if let Some(c) = fill_color {
            c
        } else {
            Color::new(
                (face_id.wrapping_mul(60) & 255) as u8,
                (face_id.wrapping_mul(120) & 255) as u8,
                (face_id.wrapping_mul(240) & 255) as u8,
                255,
            )
        };
        {
            let mut rl3d = draw_handle.begin_mode3D(observer);
            rl3d.draw_triangle3D(vertex_a, vertex_b, vertex_c, color);
        }

        if label {
            let screen_w = draw_handle.get_screen_width() as f32;
            let screen_h = draw_handle.get_screen_height() as f32;
            let centroid = (vertex_a + vertex_b + vertex_c) / 3.0;
            let sx = ((centroid.x) * 0.5 + 0.5) * screen_w;
            let sy = ((-centroid.y) * 0.5 + 0.5) * screen_h;
            draw_handle.draw_text(&face_id.to_string(), sx as i32, sy as i32, 14, Color::WHITE);
        }
    }
}

#[inline]
pub fn observed_line_of_sight(observer: &Camera3D) -> Vec3 {
    Vec3::new(
        observer.target.x - observer.position.x,
        observer.target.y - observer.position.y,
        observer.target.z - observer.position.z,
    )
    .normalize_or_zero()
}

#[inline]
fn ensure_indices(mesh: &mut WeakMesh) {
    if mesh.indices.is_null() {
        let vertex_count = mesh.vertexCount as usize;
        debug_assert!(vertex_count % 3 == 0, "triangle soup must be multiple of 3");
        let indices: Vec<u16> = (0..vertex_count as u16).collect();
        mesh.indices = Box::leak(indices.into_boxed_slice()).as_mut_ptr();
        mesh.triangleCount = (vertex_count / 3) as i32;
    }
}

#[inline]
fn ensure_texcoords(mesh: &mut WeakMesh) {
    if mesh.texcoords.is_null() {
        let vertex_count = mesh.vertexCount as usize;
        let texcoords = vec![0.0f32; vertex_count * 2];
        mesh.texcoords = Box::leak(texcoords.into_boxed_slice()).as_mut_ptr();
    }
}

#[inline]
pub fn ensure_drawable_with_texture(mesh: &mut WeakMesh) {
    ensure_indices(mesh);
    ensure_texcoords(mesh);
}
