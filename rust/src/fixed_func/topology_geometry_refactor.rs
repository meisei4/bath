use crate::fixed_func::faces::{collect_faces, observed_line_of_sight};
use crate::fixed_func::silhouette_geometry_util::rotate_vertices;
use crate::fixed_func::welding::{
    build_edge_owner_map, collect_welded_vertices, face_normal, WeldedEdge, WeldedVertex,
};
use raylib::camera::Camera3D;
use raylib::models::{Model, RaylibMesh, RaylibModel};
use std::collections::HashSet;

pub struct FaceClassification {
    pub all_faces: Vec<[u32; 3]>,
    pub front_faces: HashSet<usize>,
    pub back_faces: HashSet<usize>,
    pub silhouette_faces: Vec<usize>,
}

pub fn classify_faces(model: &Model, mesh_rotation: f32, observer: &Camera3D) -> FaceClassification {
    let mesh = &model.meshes()[0];
    let vertices = mesh.vertices();
    let all_faces = collect_faces(mesh);
    let line_of_sight = observed_line_of_sight(observer);

    let mut front_faces_set: HashSet<usize> = HashSet::with_capacity(all_faces.len());
    for (face_id, [vertex_a_index, vertex_b_index, vertex_c_index]) in all_faces.iter().copied().enumerate() {
        let mut face_vertices = vec![
            vertices[vertex_a_index as usize],
            vertices[vertex_b_index as usize],
            vertices[vertex_c_index as usize],
        ];
        rotate_vertices(&mut face_vertices, mesh_rotation);
        let face_normal_vector = face_normal(face_vertices[0], face_vertices[1], face_vertices[2]);
        if face_normal_vector.dot(line_of_sight) >= 0.0 {
            front_faces_set.insert(face_id);
        }
    }

    let triangle_count = mesh.triangleCount as usize;
    let mut back_faces_set: HashSet<usize> =
        HashSet::with_capacity(triangle_count.saturating_sub(front_faces_set.len()));
    for face_id in 0..triangle_count {
        if !front_faces_set.contains(&face_id) {
            back_faces_set.insert(face_id);
        }
    }

    let welded_vertices = collect_welded_vertices(vertices);
    let edge_owner = build_edge_owner_map(&all_faces, &welded_vertices);

    let mut silhouette_set: HashSet<usize> = HashSet::with_capacity(all_faces.len() / 2);
    for (current_face, [vertex_a_index, vertex_b_index, vertex_c_index]) in all_faces.iter().copied().enumerate() {
        let welded_vertex_a = WeldedVertex {
            id: welded_vertices[vertex_a_index as usize],
        };
        let welded_vertex_b = WeldedVertex {
            id: welded_vertices[vertex_b_index as usize],
        };
        let welded_vertex_c = WeldedVertex {
            id: welded_vertices[vertex_c_index as usize],
        };

        for (welded_vertex_0, welded_vertex_1) in [
            (welded_vertex_a, welded_vertex_b),
            (welded_vertex_b, welded_vertex_c),
            (welded_vertex_c, welded_vertex_a),
        ] {
            let welded_edge = WeldedEdge::new(welded_vertex_0, welded_vertex_1);
            if let Some(&adjacent_face) = edge_owner.get(&welded_edge) {
                if adjacent_face != current_face {
                    let is_front_current = front_faces_set.contains(&current_face);
                    let is_front_adjacent = front_faces_set.contains(&adjacent_face);
                    if is_front_current ^ is_front_adjacent {
                        silhouette_set.insert(current_face);
                        silhouette_set.insert(adjacent_face);
                    }
                }
                // open-border case intentionally left out (your current behavior)
            }
        }
    }

    let mut silhouette_vec: Vec<usize> = silhouette_set.into_iter().collect();
    silhouette_vec.sort_unstable();

    FaceClassification {
        all_faces,
        front_faces: front_faces_set,
        back_faces: back_faces_set,
        silhouette_faces: silhouette_vec,
    }
}
