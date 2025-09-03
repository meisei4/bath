use crate::fixed_func::silhouette::rotate_vertices;
use raylib::camera::Camera3D;
use raylib::color::Color;
use raylib::drawing::{RaylibDraw, RaylibDraw3D, RaylibDrawHandle, RaylibMode3DExt};
use raylib::math::glam::{Vec2, Vec3};
use raylib::math::Vector3;
use raylib::models::{RaylibMesh, WeakMesh};
use std::collections::{HashMap, HashSet};
use std::ptr::null_mut;
use std::slice::{from_raw_parts, from_raw_parts_mut};

pub struct Topology {
    pub all_faces: Vec<[u32; 3]>,
    pub front_faces: Option<HashSet<usize>>,
    pub back_faces: Option<HashSet<usize>>,
    pub silhouette_faces: Option<Vec<usize>>,
    pub welded_vertices: Vec<u32>,
    pub welded_vertices_per_face: Option<Vec<[WeldedVertex; 3]>>,
    pub neighbors_per_face: Option<Vec<[Option<usize>; 3]>>,
    pub face_vertex_positions_model: Option<Vec<[Vec3; 3]>>,
    pub face_texcoords: Option<Vec<[Vec2; 3]>>,
    pub vertex_normals: Option<Vec<Vec3>>,
    pub corner_angles_per_face: Option<Vec<[f32; 3]>>,
}

pub fn topology_init(mesh: &WeakMesh) -> Topology {
    let vertex_positions = mesh.vertices();
    assert!(
        !vertex_positions.is_empty(),
        "topology_init: mesh has no vertex positions."
    );
    let all_faces = collect_faces(mesh);
    assert!(!all_faces.is_empty(), "topology_init: mesh has no faces.");
    let welded_vertices_per_vertex = collect_welded_vertices(vertex_positions);
    Topology {
        all_faces,
        welded_vertices: welded_vertices_per_vertex,
        welded_vertices_per_face: None,
        neighbors_per_face: None,
        face_vertex_positions_model: None,
        front_faces: None,
        back_faces: None,
        silhouette_faces: None,
        face_texcoords: None,
        vertex_normals: None,
        corner_angles_per_face: None,
    }
}

pub fn collect_faces(mesh: &WeakMesh) -> Vec<[u32; 3]> {
    let faces: Vec<[u32; 3]> = mesh
        .indices()
        .chunks_exact(3)
        .map(|chunk| [chunk[0] as u32, chunk[1] as u32, chunk[2] as u32])
        .collect();
    faces
}

pub fn collect_welded_faces(topology: &mut Topology) {
    let welded_ids = &topology.welded_vertices;
    let mut welded_vertices_per_face = Vec::with_capacity(topology.all_faces.len());
    for [vertex_a_index, vertex_b_index, vertex_c_index] in topology.all_faces.iter().copied() {
        let welded_vertex_a = WeldedVertex {
            id: welded_ids[vertex_a_index as usize],
        };
        let welded_vertex_b = WeldedVertex {
            id: welded_ids[vertex_b_index as usize],
        };
        let welded_vertex_c = WeldedVertex {
            id: welded_ids[vertex_c_index as usize],
        };
        welded_vertices_per_face.push([welded_vertex_a, welded_vertex_b, welded_vertex_c]);
    }
    topology.welded_vertices_per_face = Some(welded_vertices_per_face);
}

pub fn collect_neighbors(topology: &mut Topology) {
    let welded_faces = topology
        .welded_vertices_per_face
        .as_ref()
        .expect("collect_neighbors: welded_vertices_per_face not present. Call collect_welded_faces first.");
    let mut neighbors: Vec<[Option<usize>; 3]> = vec![[None, None, None]; welded_faces.len()];
    let mut first_owner: HashMap<WeldedEdge, (usize, u8)> = HashMap::new();
    for (face_id, welded_triplet) in welded_faces.iter().copied().enumerate() {
        let edges: [(WeldedEdge, u8); 3] = [
            (WeldedEdge::new(welded_triplet[0], welded_triplet[1]), 0), // AB
            (WeldedEdge::new(welded_triplet[1], welded_triplet[2]), 1), // BC
            (WeldedEdge::new(welded_triplet[2], welded_triplet[0]), 2), // CA
        ];
        for (welded_edge, local_edge_index) in edges {
            if let Some(&(other_face_id, other_local_edge_index)) = first_owner.get(&welded_edge) {
                neighbors[face_id][local_edge_index as usize] = Some(other_face_id);
                neighbors[other_face_id][other_local_edge_index as usize] = Some(face_id);
            } else {
                first_owner.insert(welded_edge, (face_id, local_edge_index));
            }
        }
    }
    topology.neighbors_per_face = Some(neighbors);
}

pub fn collect_face_positions_model(topology: &mut Topology, mesh: &WeakMesh) {
    let vertices = mesh.vertices();
    let mut face_positions = Vec::with_capacity(topology.all_faces.len());
    for [vertex_a_index, vertex_b_index, vertex_c_index] in topology.all_faces.iter().copied() {
        face_positions.push([
            vertices[vertex_a_index as usize],
            vertices[vertex_b_index as usize],
            vertices[vertex_c_index as usize],
        ]);
    }
    topology.face_vertex_positions_model = Some(face_positions);
}

pub fn collect_face_texcoords(topology: &mut Topology, mesh: &WeakMesh) {
    let vertex_count = mesh.vertexCount as usize;

    // raw pointer; safe if we check null and size properly.
    let uvs_slice: Vec<f32> = if mesh.texcoords.is_null() {
        vec![0.0; vertex_count * 2]
    } else {
        unsafe { from_raw_parts(mesh.texcoords, vertex_count * 2) }.to_vec()
    };

    let mut face_uvs = Vec::with_capacity(topology.all_faces.len());
    for [ia, ib, ic] in topology.all_faces.iter().copied() {
        let a = (uvs_slice[ia as usize * 2 + 0], uvs_slice[ia as usize * 2 + 1]);
        let b = (uvs_slice[ib as usize * 2 + 0], uvs_slice[ib as usize * 2 + 1]);
        let c = (uvs_slice[ic as usize * 2 + 0], uvs_slice[ic as usize * 2 + 1]);
        face_uvs.push([Vec2::new(a.0, a.1), Vec2::new(b.0, b.1), Vec2::new(c.0, c.1)]);
    }

    topology.face_texcoords = Some(face_uvs);
}

pub fn collect_front_faces(topology: &mut Topology, mesh: &WeakMesh, rotation: f32, observer: &Camera3D) {
    let line_of_sight = observed_line_of_sight(observer);
    if let Some(face_positions_model) = &topology.face_vertex_positions_model {
        let mut front_faces = HashSet::with_capacity(topology.all_faces.len());
        for (face_id, [vertex_a, vertex_b, vertex_c]) in face_positions_model.iter().copied().enumerate() {
            let mut triangle = vec![vertex_a, vertex_b, vertex_c];
            rotate_vertices(&mut triangle, rotation);

            let normal = face_normal(triangle[0], triangle[1], triangle[2]);
            // const SILHOUETTE_FACE_BIAS: f32 = 0.1; // try 0.02..0.08
            // if normal.dot(line_of_sight) <= -SILHOUETTE_FACE_BIAS { front_faces.insert(face_id); }
            if normal.dot(line_of_sight) <= 0.0 {
                front_faces.insert(face_id);
            }
        }
        topology.front_faces = Some(front_faces);
        return;
    }
    let vertices = mesh.vertices();
    let mut front_faces = HashSet::with_capacity(topology.all_faces.len());
    for (face_id, [vertex_a_index, vertex_b_index, vertex_c_index]) in topology.all_faces.iter().copied().enumerate() {
        let mut triangle = vec![
            vertices[vertex_a_index as usize],
            vertices[vertex_b_index as usize],
            vertices[vertex_c_index as usize],
        ];
        rotate_vertices(&mut triangle, rotation);
        let normal = face_normal(triangle[0], triangle[1], triangle[2]);
        if normal.dot(line_of_sight) <= 0.0 {
            front_faces.insert(face_id);
        }
    }
    topology.front_faces = Some(front_faces);
}

pub fn collect_back_faces(topology: &mut Topology) {
    let front_faces = topology
        .front_faces
        .as_ref()
        .expect("collect_back_faces: front_faces not present. Call collect_front_faces first.");
    let triangle_count = topology.all_faces.len();
    let mut back_faces = HashSet::with_capacity(triangle_count.saturating_sub(front_faces.len()));
    for face_id in 0..triangle_count {
        if !front_faces.contains(&face_id) {
            back_faces.insert(face_id);
        }
    }
    topology.back_faces = Some(back_faces);
}

pub fn collect_silhouette_faces(topology: &mut Topology) {
    let neighbors_per_face = topology
        .neighbors_per_face
        .as_ref()
        .expect("collect_silhouette_faces: neighbors_per_face not present. Call collect_neighbors first.");
    let front_faces = topology
        .front_faces
        .as_ref()
        .expect("collect_silhouette_faces: front_faces not present. Call collect_front_faces first.");
    let welded_vertices_per_face = topology
        .welded_vertices_per_face
        .as_ref()
        .expect("collect_silhouette_faces: welded_vertices_per_face not present. Call collect_welded_faces first.");
    let mut silhouette_faces: HashSet<usize> = HashSet::new();
    for (face_id, welded_triplet) in welded_vertices_per_face.iter().copied().enumerate() {
        let local_edges = [(0, 1, 0), (1, 2, 1), (2, 0, 2)];
        for (vertex_0, vertex_1, local_edge_index) in local_edges {
            let neighbor_face_opt = neighbors_per_face[face_id][local_edge_index as usize];
            if neighbor_face_opt.is_none() {
                continue;
            }
            let neighbor_face_id = neighbor_face_opt.unwrap();
            let is_front_here = front_faces.contains(&face_id);
            let is_front_neighbor = front_faces.contains(&neighbor_face_id);
            if is_front_here == is_front_neighbor {
                continue;
            }
            let silhouette_face = if is_front_here { face_id } else { neighbor_face_id };
            silhouette_faces.insert(silhouette_face);
            let vertices = welded_vertices_per_face[silhouette_face];
            let silhouette_edge_key = WeldedEdge::new(welded_triplet[vertex_0], welded_triplet[vertex_1]);
            let rim_local_edge_index = {
                let candidates = [
                    (WeldedEdge::new(vertices[0], vertices[1]), 0u8),
                    (WeldedEdge::new(vertices[1], vertices[2]), 1u8),
                    (WeldedEdge::new(vertices[2], vertices[0]), 2u8),
                ];
                let mut found = 0u8;
                for (edge, idx) in candidates {
                    if edge == silhouette_edge_key {
                        found = idx;
                        break;
                    }
                }
                found
            };
            let interior_edge_index_0 = ((rim_local_edge_index as i32 + 1) % 3) as usize;
            let interior_edge_index_1 = ((rim_local_edge_index as i32 + 2) % 3) as usize;
            if let Some(neighbor_0) = neighbors_per_face[silhouette_face][interior_edge_index_0] {
                if front_faces.contains(&neighbor_0) {
                    silhouette_faces.insert(neighbor_0);
                }
            }
            if let Some(neighbor_1) = neighbors_per_face[silhouette_face][interior_edge_index_1] {
                if front_faces.contains(&neighbor_1) {
                    silhouette_faces.insert(neighbor_1);
                }
            }
        }
    }
    let mut silhouette_vec: Vec<usize> = silhouette_faces.into_iter().collect();
    silhouette_vec.sort_unstable();
    topology.silhouette_faces = Some(silhouette_vec);
}

pub struct WeldedMesh {
    pub original_vertices: Vec<[Vec3; 3]>,
    pub welded_faces: Vec<[WeldedVertex; 3]>,
    pub texcoords: Vec<[Vec2; 3]>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct WeldedVertex {
    pub id: u32,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct WeldedEdge {
    pub vertex_a: WeldedVertex,
    pub vertex_b: WeldedVertex,
}

impl WeldedEdge {
    pub fn new(node_a: WeldedVertex, node_b: WeldedVertex) -> Self {
        if node_a.id <= node_b.id {
            WeldedEdge {
                vertex_a: node_a,
                vertex_b: node_b,
            }
        } else {
            WeldedEdge {
                vertex_a: node_b,
                vertex_b: node_a,
            }
        }
    }
}

pub fn build_weld_view(topology: &mut Topology, mesh: &WeakMesh) -> WeldedMesh {
    if topology.welded_vertices_per_face.is_none() {
        collect_welded_faces(topology);
    }
    if topology.face_vertex_positions_model.is_none() {
        collect_face_positions_model(topology, mesh);
    }
    if topology.face_texcoords.is_none() {
        collect_face_texcoords(topology, mesh);
    }

    WeldedMesh {
        original_vertices: topology
            .face_vertex_positions_model
            .as_ref()
            .expect("face_vertex_positions_model missing")
            .clone(),
        welded_faces: topology
            .welded_vertices_per_face
            .as_ref()
            .expect("welded_vertices_per_face missing")
            .clone(),
        texcoords: topology
            .face_texcoords
            .as_ref()
            .expect("face_texcoords missing")
            .clone(),
    }
}

pub fn collect_welded_vertices(original_vertices: &[Vector3]) -> Vec<u32> {
    let mut welded_vertices_to_faces_map: HashMap<(i32, i32, i32), u32> = HashMap::new();
    let mut next_wid: u32 = 0;
    let mut welded_id_per_vertex = vec![0u32; original_vertices.len()];
    for (i, v) in original_vertices.iter().enumerate() {
        let key = (quantize(v.x), quantize(v.y), quantize(v.z));
        let wid = *welded_vertices_to_faces_map.entry(key).or_insert_with(|| {
            let id = next_wid;
            next_wid += 1;
            id
        });
        welded_id_per_vertex[i] = wid;
    }
    welded_id_per_vertex
}

pub fn build_edge_owner_map(faces: &[[u32; 3]], welded_vertices: &[u32]) -> HashMap<WeldedEdge, usize> {
    let mut edge_owner: HashMap<WeldedEdge, usize> = HashMap::new();
    for (face, [vertex_a_index, vertex_b_index, vertex_c_index]) in faces.iter().copied().enumerate() {
        let welded_vertex_a = WeldedVertex {
            id: welded_vertices[vertex_a_index as usize],
        };
        let welded_vertex_b = WeldedVertex {
            id: welded_vertices[vertex_b_index as usize],
        };
        let welded_vertex_c = WeldedVertex {
            id: welded_vertices[vertex_c_index as usize],
        };
        for (w0, w1) in [
            (welded_vertex_a, welded_vertex_b),
            (welded_vertex_b, welded_vertex_c),
            (welded_vertex_c, welded_vertex_a),
        ] {
            edge_owner.entry(WeldedEdge::new(w0, w1)).or_insert(face);
        }
    }
    edge_owner
}

#[inline]
pub fn quantize(x: f32) -> i32 {
    const WELD_VERTEX_EPSILON: f32 = 1e-5; // e-1 and up works, 0 goes crazy
    (x / WELD_VERTEX_EPSILON).round() as i32
}

#[inline]
pub fn edge_opposing_vertex(edge: (u8, u8)) -> u8 {
    3 - (edge.0 + edge.1)
}

#[inline]
pub fn welded_eq(a: WeldedVertex, b: WeldedVertex) -> bool {
    a.id == b.id
}

#[inline]
pub fn face_normal(a: Vec3, b: Vec3, c: Vec3) -> Vec3 {
    (b - a).cross(c - a).normalize_or_zero()
}

#[inline]
pub fn face_normal_area_weighted(a: Vec3, b: Vec3, c: Vec3) -> Vec3 {
    (b - a).cross(c - a)
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

#[inline]
pub fn observed_line_of_sight(observer: &Camera3D) -> Vec3 {
    Vec3::new(
        observer.target.x - observer.position.x,
        observer.target.y - observer.position.y,
        observer.target.z - observer.position.z,
    )
    .normalize_or_zero()
}

pub fn vertex_normals(mesh: &WeakMesh) -> Vec<Vec3> {
    let vertices = mesh.vertices();
    let vertex_count = vertices.len();
    let mut accum = vec![Vec3::ZERO; vertex_count];
    // TODO: guaranteed by ensure_drawable()????
    let indices: Vec<u16> = if mesh.indices.is_null() {
        (0..vertex_count as u16).collect()
    } else {
        unsafe { from_raw_parts(mesh.indices, (mesh.triangleCount as usize) * 3) }.to_vec()
    };
    for face in indices.chunks_exact(3) {
        let vertex_a_index = face[0] as usize;
        let vertex_b_index = face[1] as usize;
        let vertex_c_index = face[2] as usize;
        let vertex_a = vertices[vertex_a_index];
        let vertex_b = vertices[vertex_b_index];
        let vertex_c = vertices[vertex_c_index];
        let face_normal = face_normal(vertex_a, vertex_b, vertex_c);
        accum[vertex_a_index] += face_normal;
        accum[vertex_b_index] += face_normal;
        accum[vertex_c_index] += face_normal;
    }
    // accum.into_iter().map(|face_normal| face_normal).collect()
    accum
}

pub fn collect_vertex_normals(topology: &mut Topology, mesh: &WeakMesh) {
    let vertices = mesh.vertices();
    let vertex_count = vertices.len();
    let mut accumulated_vertex_normals = vec![Vec3::ZERO; vertex_count];
    let mut corner_angles: Vec<[f32; 3]> = Vec::with_capacity(topology.all_faces.len());
    // TODO: guaranteed by ensure_drawable()????
    let indices: Vec<u16> = if mesh.indices.is_null() {
        (0..vertex_count as u16).collect()
    } else {
        unsafe { from_raw_parts(mesh.indices, (mesh.triangleCount as usize) * 3) }.to_vec()
    };
    for face in indices.chunks_exact(3) {
        let vertex_a_index = face[0] as usize;
        let vertex_b_index = face[1] as usize;
        let vertex_c_index = face[2] as usize;
        let vertex_a = vertices[vertex_a_index];
        let vertex_b = vertices[vertex_b_index];
        let vertex_c = vertices[vertex_c_index];
        let ab = vertex_b - vertex_a;
        let ac = vertex_c - vertex_a;
        let ba = vertex_a - vertex_b;
        let bc = vertex_c - vertex_b;
        let ca = vertex_a - vertex_c;
        let cb = vertex_b - vertex_c;
        let angle_a = angle_between(ab, ac);
        let angle_b = angle_between(ba, bc);
        let angle_c = angle_between(ca, cb);
        corner_angles.push([angle_a, angle_b, angle_c]);
        let face_normal = face_normal_area_weighted(vertex_a, vertex_b, vertex_c); // area-weighted direction
        accumulated_vertex_normals[vertex_a_index] += face_normal * angle_a;
        accumulated_vertex_normals[vertex_b_index] += face_normal * angle_b;
        accumulated_vertex_normals[vertex_c_index] += face_normal * angle_c;
    }
    topology.corner_angles_per_face = Some(corner_angles);
    topology.vertex_normals = Some(
        accumulated_vertex_normals
            .into_iter()
            .map(|n| n.normalize_or_zero())
            .collect(),
    );
}

pub fn smooth_vertex_normals(topology: &Topology) -> Vec<Vec3> {
    let vertex_normals = topology
        .vertex_normals
        .as_ref()
        .expect("smooth_vertex_normals: topology.vertex_normals missing");
    let welded_vertices = &topology.welded_vertices;
    let vertex_count = vertex_normals.len();
    let welded_count = (welded_vertices.iter().copied().max().unwrap_or(0) as usize) + 1;
    let mut welded_vertex_normals_accumulator = vec![Vec3::ZERO; welded_count];
    for i in 0..vertex_count {
        let welded_vertex = welded_vertices[i] as usize;
        welded_vertex_normals_accumulator[welded_vertex] += vertex_normals[i];
    }
    let normalized_accumulation: Vec<Vec3> = welded_vertex_normals_accumulator
        .into_iter()
        .map(|normal| normal.normalize_or_zero())
        .collect();
    let mut smoothed_vertex_normals = vec![Vec3::ZERO; vertex_count];
    for i in 0..vertex_count {
        smoothed_vertex_normals[i] = normalized_accumulation[welded_vertices[i] as usize];
    }
    smoothed_vertex_normals
}

#[inline]
fn angle_between(a: Vec3, b: Vec3) -> f32 {
    let a_magnitude = a.length();
    let b_magnitude = b.length();
    if a_magnitude <= 0.0 || b_magnitude <= 0.0 {
        return 0.0;
    }
    (a.dot(b) / (a_magnitude * b_magnitude)).clamp(-1.0, 1.0).acos()
}

pub fn reverse_vertex_winding(mesh: &mut WeakMesh) {
    if mesh.indices.is_null() {
        return;
    }
    let triangle_count = mesh.triangleCount as usize;
    let indices = unsafe { from_raw_parts_mut(mesh.indices, triangle_count * 3) };
    for face in 0..triangle_count {
        indices.swap(face * 3 + 1, face * 3 + 2);
    }
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
pub fn ensure_drawable(mesh: &mut WeakMesh) {
    mesh.normals = null_mut(); //TODO: what in the fuck? find out where this happens in the raylib updates i guess?
    ensure_indices(mesh);
    mesh.colors = null_mut();
    ensure_texcoords(mesh);
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
