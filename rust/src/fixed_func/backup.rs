use raylib::math::glam::{Vec2, Vec3};
use raylib::models::{Mesh, WeakMesh};
use std::collections::{HashMap, VecDeque};
use std::f32::consts::{FRAC_2_PI, PI};
use std::mem::{swap, zeroed};
use std::ptr::null_mut;
use std::slice::from_raw_parts;

pub const ZOOM_SCALE: f32 = 2.0;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
struct WeldedVertex {
    id: u32,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
struct Face {
    id: usize,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
struct WeldedEdge {
    vertex_a: WeldedVertex,
    vertex_b: WeldedVertex,
}

impl WeldedEdge {
    fn new(node_a: WeldedVertex, node_b: WeldedVertex) -> Self {
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

struct WeldedMesh {
    original_vertices: Vec<[Vec3; 3]>,
    welded_vertices: Vec<Vec3>, //TODO: this is never used? shouldnt we be able to use it somehow? faces welded is enough?
    welded_faces: Vec<[WeldedVertex; 3]>,
    texcoords: Vec<[Vec2; 3]>,
}

#[derive(Copy, Clone, Debug)]
struct DualEdge {
    face_a: Face,
    face_b: Face,
    face_a_local_edge: (u8, u8),
    face_b_local_edge: (u8, u8),
    welded_edge: WeldedEdge,
    fold_weight: f32, //TODO: PI - dihedral angle? because PI/ 180 is a fully flat crease between two faces
}

struct DisjointSetUnion {
    disjoint_sets: Vec<usize>,
    rank: Vec<u8>, //TODO: rank is how many nodes in already exist in a given disjoint set...? I DONT LIKE PARALLEL ARRAYS!!
}
impl DisjointSetUnion {
    fn new(nodes: usize) -> Self {
        Self {
            disjoint_sets: (0..nodes).collect(),
            rank: vec![0; nodes],
        }
    }
    fn find(&mut self, node: usize) -> usize {
        if self.disjoint_sets[node] != node {
            self.disjoint_sets[node] = self.find(self.disjoint_sets[node]);
        }
        self.disjoint_sets[node]
    }
    fn union(&mut self, node_a: usize, node_b: usize) -> bool {
        let (mut a_representative, mut b_representative) = (self.find(node_a), self.find(node_b));
        if a_representative == b_representative {
            return false;
        }
        if self.rank[a_representative] < self.rank[b_representative] {
            swap(&mut a_representative, &mut b_representative);
        }
        self.disjoint_sets[b_representative] = a_representative;
        if self.rank[a_representative] == self.rank[b_representative] {
            self.rank[a_representative] += 1;
        }
        true
    }
}

#[derive(Copy, Clone, Debug, Default)]
struct ParentLink {
    parent: Option<Face>, //TODO: why arent we using this?
    parent_local_edge: Option<(u8, u8)>,
    child_local_edge: Option<(u8, u8)>,
    welded_edge: Option<WeldedEdge>,
}

pub fn unfold(mesh: &mut WeakMesh) -> Mesh {
    let welded_mesh = weld_mesh(mesh);
    let face_count = welded_mesh.welded_faces.len();

    let mut dual_graph = build_dual_graph(&welded_mesh);
    let (parent_links, children) = build_parent_tree(face_count, &mut dual_graph);

    let mut local_vertices_per_face = Vec::with_capacity(face_count);
    for face_index in 0..face_count {
        let [a, b, c] = welded_mesh.original_vertices[face_index];
        local_vertices_per_face.push(derive_local_plane_vertices(a, b, c));
    }
    let mut unfolded_faces = vec![[Vec2::ZERO; 3]; face_count];
    let mut is_already_unfolded = vec![false; face_count];

    for id in 0..face_count {
        if is_already_unfolded[id] {
            continue;
        }
        unfolded_faces[id] = anchor_welded_face(local_vertices_per_face[id], &welded_mesh.welded_faces[id]);
        // faces_placed_in_draw_space[id] = vertices_per_face_local_space[id];
        is_already_unfolded[id] = true;
        let mut face_stack = vec![Face { id }];

        while let Some(face) = face_stack.pop() {
            for &child_face in &children[face.id] {
                if is_already_unfolded[child_face.id] {
                    continue;
                }
                let parent_link = parent_links[child_face.id];
                let aligned_child_face = align_child_to_parent(
                    &local_vertices_per_face[child_face.id],
                    &welded_mesh.welded_faces[child_face.id],
                    &unfolded_faces[face.id],
                    &welded_mesh.welded_faces[face.id],
                    parent_link.parent_local_edge.unwrap(),
                    parent_link.child_local_edge.unwrap(),
                    parent_link.welded_edge.unwrap(),
                );
                unfolded_faces[child_face.id] = aligned_child_face;
                is_already_unfolded[child_face.id] = true;
                face_stack.push(child_face);
            }
        }
    }

    let mut unfolded_vertices = Vec::with_capacity(face_count * 9);
    let mut unfolded_texcoords = Vec::with_capacity(face_count * 6);
    let mut unfolded_indices = Vec::with_capacity(face_count * 3);

    for face in 0..face_count {
        let triangle = unfolded_faces[face];
        for i in 0..3 {
            let vertex = triangle[i];
            unfolded_vertices.extend_from_slice(&[vertex.x, vertex.y, 0.0]);
            let texcoords = welded_mesh.texcoords[face][i];
            unfolded_texcoords.extend_from_slice(&[texcoords.x, texcoords.y]);
            unfolded_indices.push((unfolded_vertices.len() / 3 - 1) as u16);
        }
    }

    let (mut min_x, mut min_y, mut max_x, mut max_y) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    for i in (0..unfolded_vertices.len()).step_by(3) {
        let (x, y) = (unfolded_vertices[i], unfolded_vertices[i + 1]);
        min_x = min_x.min(x);
        max_x = max_x.max(x);
        min_y = min_y.min(y);
        max_y = max_y.max(y);
    }
    let (center_x, center_y) = ((min_x + max_x) * 0.5, (min_y + max_y) * 0.5);
    let (step_size_x, step_size_y) = (max_x - min_x, max_y - min_y);
    let step = ZOOM_SCALE / step_size_x.max(step_size_y);
    for i in (0..unfolded_vertices.len()).step_by(3) {
        unfolded_vertices[i] = (unfolded_vertices[i] - center_x) * step;
        unfolded_vertices[i + 1] = (unfolded_vertices[i + 1] - center_y) * step;
    }

    let mut unfolded_mesh: Mesh = unsafe { zeroed() };
    unfolded_mesh.vertexCount = (unfolded_vertices.len() / 3) as i32;
    unfolded_mesh.triangleCount = (unfolded_indices.len() / 3) as i32;
    unfolded_mesh.vertices = Box::leak(unfolded_vertices.into_boxed_slice()).as_mut_ptr();
    unfolded_mesh.indices = Box::leak(unfolded_indices.into_boxed_slice()).as_mut_ptr();
    unfolded_mesh.texcoords = Box::leak(unfolded_texcoords.into_boxed_slice()).as_mut_ptr();
    unfolded_mesh.normals = null_mut();
    unfolded_mesh.tangents = null_mut();
    unfolded_mesh.colors = null_mut();
    unfolded_mesh
}

fn weld_mesh(mesh: &WeakMesh) -> WeldedMesh {
    let triangle_count = mesh.triangleCount as usize;
    let vertex_count = mesh.vertexCount as usize;
    let src_vertices = unsafe { from_raw_parts(mesh.vertices, 3 * vertex_count) };
    let src_texcoords = unsafe { from_raw_parts(mesh.texcoords, 2 * vertex_count) };

    let src_indices = if mesh.indices.is_null() {
        debug_assert!(vertex_count % 3 == 0, "non-indexed mesh must be triangle soup");
        (0..vertex_count as u16).collect()
    } else {
        unsafe { from_raw_parts(mesh.indices, 3 * triangle_count) }.to_vec()
    };

    let use_indices_for_weld = !mesh.indices.is_null();

    // keep your original map, but add an index-based one (only used if indices exist)
    let mut quantized_vertex_to_welded_vertex_map: HashMap<(i32, i32, i32), WeldedVertex> = HashMap::new();
    let mut index_to_welded_vertex_map: Option<HashMap<usize, WeldedVertex>> = if use_indices_for_weld {
        Some(HashMap::new())
    } else {
        None
    };

    let welded_vertices = Vec::new();
    let mut welded_vertices_count = 0;

    let mut welded_faces = Vec::with_capacity(src_indices.len() / 3);
    let mut face_texcoords = Vec::with_capacity(src_indices.len() / 3);
    let mut face_vertices = Vec::with_capacity(src_indices.len() / 3);

    for face_index in 0..(src_indices.len() / 3) {
        let mut face_welded_vertices = [WeldedVertex { id: 0 }; 3];
        let mut face_texcoord = [Vec2::ZERO; 3];
        let mut face_vertex = [Vec3::ZERO; 3];

        for i in 0..3 {
            let vertex_index = src_indices[face_index * 3 + i] as usize;
            let vertex_x = src_vertices[vertex_index * 3 + 0];
            let vertex_y = src_vertices[vertex_index * 3 + 1];
            let vertex_z = src_vertices[vertex_index * 3 + 2];
            let vertex = Vec3::new(vertex_x, vertex_y, vertex_z);

            let welded_vertex_id = if let Some(ref mut map) = index_to_welded_vertex_map {
                *map.entry(vertex_index).or_insert_with(|| {
                    let weld_id = WeldedVertex {
                        id: welded_vertices_count,
                    };
                    welded_vertices_count += 1;
                    weld_id
                })
            } else {
                *quantized_vertex_to_welded_vertex_map
                    .entry((quantize(vertex_x), quantize(vertex_y), quantize(vertex_z)))
                    .or_insert_with(|| {
                        let weld_id = WeldedVertex {
                            id: welded_vertices_count,
                        };
                        welded_vertices_count += 1;
                        weld_id
                    })
            };

            face_welded_vertices[i] = welded_vertex_id;
            face_vertex[i] = vertex;

            let s = src_texcoords[vertex_index * 2 + 0];
            let t = src_texcoords[vertex_index * 2 + 1];
            face_texcoord[i] = Vec2::new(s, t);
        }
        welded_faces.push(face_welded_vertices);
        face_texcoords.push(face_texcoord);
        face_vertices.push(face_vertex);
    }

    WeldedMesh {
        original_vertices: face_vertices,
        welded_vertices,
        welded_faces,
        texcoords: face_texcoords,
    }
}

fn build_dual_graph(welded_mesh: &WeldedMesh) -> Vec<DualEdge> {
    let face_count = welded_mesh.welded_faces.len();
    let mut face_normals = Vec::with_capacity(face_count);
    for face_index in 0..face_count {
        let [vertex_a, vertex_b, vertex_c] = welded_mesh.original_vertices[face_index];
        face_normals.push(face_normal(vertex_a, vertex_b, vertex_c));
    }

    let mut welded_edge_to_parent: HashMap<WeldedEdge, (Face, (u8, u8))> = HashMap::new();
    let mut dual_graph = Vec::new();

    for id in 0..face_count {
        let face_id = Face { id };
        let [welded_vertex_a, welded_vertex_b, welded_vertex_c] = welded_mesh.welded_faces[id];
        let local_edges = [(0u8, 1u8), (1, 2), (2, 0)];
        let face_welded_vertices = [welded_vertex_a, welded_vertex_b, welded_vertex_c];

        for &(point_a, point_b) in &local_edges {
            let edge = WeldedEdge::new(
                face_welded_vertices[point_a as usize],
                face_welded_vertices[point_b as usize],
            );

            if let Some(&(parent_face, parent_face_local_edge)) = welded_edge_to_parent.get(&edge) {
                let face_a_normal = face_normals[parent_face.id];
                let face_b_normal = face_normals[id];
                // vector_0 · vector_1 = cos(θ)
                // where θ is the angle between them.
                // Think of dot as an alignment score:
                // +1 → same direction (θ = 0°)
                // 0 → perpendicular (θ = 90°)
                // −1 → opposite directions (θ = 180°)
                let cosine_of_normals = face_a_normal.dot(face_b_normal).clamp(-1.0, 1.0);
                let dihedral_angle_between_normals = cosine_of_normals.acos();
                let fold_weight = PI - dihedral_angle_between_normals;

                dual_graph.push(DualEdge {
                    face_a: parent_face,
                    face_b: face_id,
                    welded_edge: edge,
                    face_a_local_edge: parent_face_local_edge,
                    face_b_local_edge: (point_a, point_b),
                    fold_weight,
                });
            } else {
                welded_edge_to_parent.insert(edge, (face_id, (point_a, point_b)));
            }
        }
    }
    dual_graph
}

fn build_parent_tree(face_count: usize, dual_graph: &mut [DualEdge]) -> (Vec<ParentLink>, Vec<Vec<Face>>) {
    // dual_graph.sort_by(|left, right| right.fold_weight.partial_cmp(&left.fold_weight).unwrap());
    //TODO: biggest change for the anchored faces
    dual_graph.sort_by(|left, right| dual_edge_sorting_order(left).cmp(&dual_edge_sorting_order(right)));
    let mut dsu = DisjointSetUnion::new(face_count);
    let mut adjacency_list = vec![Vec::new(); face_count];

    for edge in dual_graph.iter().copied() {
        if dsu.union(edge.face_a.id, edge.face_b.id) {
            adjacency_list[edge.face_a.id].push((edge.face_b, edge));
            adjacency_list[edge.face_b.id].push((edge.face_a, edge));
        }
    }
    for adjacent_faces in &mut adjacency_list {
        adjacent_faces
            .sort_by_key(|(face, edge)| (face.id, edge.welded_edge.vertex_a.id, edge.welded_edge.vertex_b.id));
    }
    let mut parent_links = vec![ParentLink::default(); face_count];
    let mut children = vec![Vec::new(); face_count];
    let mut seen = vec![false; face_count]; //TODO: stupid fucking parallel arrays again
    let mut face_queue = VecDeque::new();
    // orient from each unseen root (handle possible multiple components)
    for id in 0..face_count {
        if seen[id] {
            continue;
        }
        seen[id] = true;
        face_queue.push_back(Face { id });
        while let Some(current_face) = face_queue.pop_front() {
            for &(face, edge) in &adjacency_list[current_face.id] {
                if seen[face.id] {
                    continue;
                }
                seen[face.id] = true;
                let (parent_local_edge, child_local_edge) = if current_face == edge.face_a {
                    (edge.face_a_local_edge, edge.face_b_local_edge)
                } else {
                    (edge.face_b_local_edge, edge.face_a_local_edge)
                };
                parent_links[face.id] = ParentLink {
                    parent: Some(current_face),
                    parent_local_edge: Some(parent_local_edge),
                    child_local_edge: Some(child_local_edge),
                    welded_edge: Some(edge.welded_edge),
                };
                children[current_face.id].push(face);
                face_queue.push_back(face);
            }
        }
    }
    (parent_links, children)
}

fn align_to_welded_edge(
    parent_face_welded_vertices: &[WeldedVertex; 3],
    local_edge: (u8, u8),
    welded_edge: WeldedEdge,
) -> (u8, u8) {
    let local_vertex_a = local_edge.0 as usize;
    let local_vertex_b = local_edge.1 as usize;
    let welded_vertex_a = parent_face_welded_vertices[local_vertex_a];
    let welded_vertex_b = parent_face_welded_vertices[local_vertex_b];
    if eq(welded_vertex_a, welded_edge.vertex_a) && eq(welded_vertex_b, welded_edge.vertex_b) {
        (local_edge.0, local_edge.1)
    } else if eq(welded_vertex_a, welded_edge.vertex_b) && eq(welded_vertex_b, welded_edge.vertex_a) {
        (local_edge.1, local_edge.0)
    } else {
        panic!("local_edge does not match welded_edge (bad adjacency / welding).");
    }
}

fn align_child_to_parent(
    child_local_vertices: &[Vec2; 3],
    child_welded_vertices: &[WeldedVertex; 3],
    parent_vertices: &[Vec2; 3],
    parent_welded_vertices: &[WeldedVertex; 3],
    parent_edge_local: (u8, u8),
    child_edge_local: (u8, u8),
    welded_edge: WeldedEdge,
) -> [Vec2; 3] {
    let aligned_parent_edge = align_to_welded_edge(parent_welded_vertices, parent_edge_local, welded_edge);
    let aligned_child_edge = align_to_welded_edge(child_welded_vertices, child_edge_local, welded_edge);

    let parent_a = parent_vertices[aligned_parent_edge.0 as usize];
    let parent_b = parent_vertices[aligned_parent_edge.1 as usize];
    let parent_ab = parent_b - parent_a;

    let child_a = child_local_vertices[aligned_child_edge.0 as usize];
    let child_b = child_local_vertices[aligned_child_edge.1 as usize];
    let child_ab = child_b - child_a;

    // child_edge_direction    parent_edge_direction
    // →                               ↑
    // \                               │
    //  \                              │
    //   \                             │
    //    \       rotate by θ ->       │
    //     \θ                          │
    let parent_direction = parent_ab / parent_ab.length();
    let child_direction = child_ab / child_ab.length();
    //cosine eventually derives the angle of rotation in the rotation matrix
    let cosine_rotation = child_direction.dot(parent_direction).clamp(-1.0, 1.0);
    //sine eventually derives the sign/direction (CW or CCW) of the rotation
    let sine_rotation = child_direction.perp_dot(parent_direction);
    let rotation_vector = Vec2::new(cosine_rotation, sine_rotation);
    let alignment_origin = parent_a;
    // let child_aa_transform = child_local_vertices[0] - child_a;
    // let child_ab_transform = child_local_vertices[1] - child_a;
    // let child_ac_transform = child_local_vertices[2] - child_a;
    //
    // let aligned_child_a = alignment_origin + child_aa_transform.rotate(rotation_vector);
    // let aligned_child_b = alignment_origin + child_ab_transform.rotate(rotation_vector);
    // let aligned_child_c = alignment_origin + child_ac_transform.rotate(rotation_vector);
    // let mut aligned_child_vertices = [aligned_child_a, aligned_child_b, aligned_child_c];
    let mut aligned_child_face = [Vec2::ZERO; 3];
    for i in 0..3 {
        let translation_offset = child_local_vertices[i] - child_a;
        aligned_child_face[i] = alignment_origin + translation_offset.rotate(rotation_vector);
    }
    let parents_edge_opposing_vertex = parent_vertices[edge_opposing_vertex(aligned_parent_edge) as usize];
    let childs_edge_opposing_vertex = aligned_child_face[edge_opposing_vertex(aligned_child_edge) as usize];
    let parent_sign = parent_ab.perp_dot(parents_edge_opposing_vertex - alignment_origin);
    let child_sign = parent_ab.perp_dot(childs_edge_opposing_vertex - alignment_origin);
    //cases: 0) either negative -> negative. 1) Both positive = positive -> flip the child 2) both negative = positive -> flip the child
    // if (parent_sign * child_sign).is_sign_positive() {
    if (parent_sign * child_sign) > 1e-6 {
        // let parent_edge_normal = Vec2::new(-parent_direction.y, parent_direction.x);
        let normal_rotation = Vec2::new((FRAC_2_PI).cos(), 1.0);
        let parent_edge_normal = parent_direction.rotate(normal_rotation); //rotate PI/2

        for child_vertex in &mut aligned_child_face {
            let edge = *child_vertex - alignment_origin;
            let parallel_projection = edge.dot(parent_direction) * parent_direction;
            let perpendicular_projection = edge.dot(parent_edge_normal) * parent_edge_normal;
            *child_vertex = parent_a + parallel_projection - perpendicular_projection;
        }
    }
    aligned_child_face
}

fn derive_local_plane_vertices(a: Vec3, b: Vec3, c: Vec3) -> [Vec2; 3] {
    //                          C
    //                        / |
    //                      /   |
    //                 ac /     |    ←  ac - x * x_axis
    //                  /       |   (perpendicular part of ac)
    //                /         |
    //              /           |
    //             A------------B
    //    x_axis ---------------x---->
    let ab = b - a;
    let ab_x_component = ab.length();
    let ac = c - a;
    let x_axis_dir = ab.normalize_or_zero();
    let ac_x_component = ac.dot(x_axis_dir);
    let ac_x_vec = ac_x_component * x_axis_dir;
    let ac_y_vec = ac - ac_x_vec;
    let y_axis = ac_y_vec.normalize_or_zero();
    let ac_y_component = ac.dot(y_axis);
    let a_local = Vec2::new(0.0, 0.0);
    let b_local = Vec2::new(ab_x_component, 0.0);
    let c_local = Vec2::new(ac_x_component, ac_y_component);
    [a_local, b_local, c_local]
}

fn dual_edge_sorting_order(edge: &DualEdge) -> (u32, u32, usize, usize) {
    let welded_edge_vertex_a = edge.welded_edge.vertex_a.id;
    let welded_edge_vertex_b = edge.welded_edge.vertex_b.id;
    let lesser_face = edge.face_a.id.min(edge.face_b.id);
    let greater_face = edge.face_a.id.max(edge.face_b.id);
    (welded_edge_vertex_a, welded_edge_vertex_b, lesser_face, greater_face)
}

fn anchor_welded_face(face: [Vec2; 3], welded_face: &[WeldedVertex; 3]) -> [Vec2; 3] {
    // base edge stability rule: edge between the two SMALLEST welded vertices
    let mut indices = [0, 1, 2];
    indices.sort_by_key(|&i| welded_face[i].id);
    let smallest_index = indices[0];
    let second_smallest_index = indices[1];
    let largest_index = indices[2];

    let a = face[smallest_index];
    let b = face[second_smallest_index];
    let c = face[largest_index];

    let ab = b - a;
    let ab_x_component = ab.length();
    let ab_dir = ab / ab_x_component;
    let left_normal_rotation = Vec2::new((FRAC_2_PI).cos(), 1.0);
    let left_normal_ab = ab_dir.rotate(left_normal_rotation); //rotate PI/2?
    let ac = c - a;
    let ac_x_component = ac.dot(ab_dir);
    let ac_y_component = ac.dot(left_normal_ab);

    let stable_a = Vec2::new(0.0, 0.0);
    let stable_b = Vec2::new(ab_x_component, 0.0);
    let stable_c = Vec2::new(ac_x_component, ac_y_component);
    // let stable_c = if ac_y_component >= 0.0 {
    //     Vec2::new(ac_x_component, ac_y_component)
    // } else {
    //     Vec2::new(ac_x_component, -ac_y_component)
    // };
    let mut stable_welded_face = [Vec2::ZERO; 3];
    stable_welded_face[smallest_index] = stable_a;
    stable_welded_face[second_smallest_index] = stable_b;
    stable_welded_face[largest_index] = stable_c;
    stable_welded_face
}

#[inline]
fn face_normal(vertex_a: Vec3, vertex_b: Vec3, vertex_c: Vec3) -> Vec3 {
    (vertex_b - vertex_a).cross(vertex_c - vertex_a).normalize_or_zero()
}

#[inline]
fn quantize(x: f32) -> i32 {
    const WELD_VERTEX_EPSILON: f32 = 1e-1; // -1 and up works, 0 goes crazy
    (x / WELD_VERTEX_EPSILON).round() as i32
}

#[inline]
fn edge_opposing_vertex(edge: (u8, u8)) -> u8 {
    3 - (edge.0 + edge.1)
}

#[inline]
fn eq(a: WeldedVertex, b: WeldedVertex) -> bool {
    a.id == b.id
}
