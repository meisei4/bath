use raylib::math::glam::{Vec2, Vec3};
use raylib::models::{Mesh, WeakMesh};
use std::collections::{HashMap, VecDeque};
use std::f32::consts::PI;
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
        // unfolded_faces[id] = local_vertices_per_face[id];
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
    // unfolded_mesh.indices = null_mut();
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

    let mut quantized_vertex_to_welded_vertex_map: HashMap<(i32, i32, i32), WeldedVertex> = HashMap::new();
    let mut index_to_welded_vertex_map: Option<HashMap<usize, WeldedVertex>> = if use_indices_for_weld {
        Some(HashMap::new())
    } else {
        None
    };

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
                dual_graph.push(DualEdge {
                    face_a: parent_face,
                    face_b: face_id,
                    welded_edge: edge,
                    face_a_local_edge: parent_face_local_edge,
                    face_b_local_edge: (point_a, point_b),
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
    let parent_x_axis = parent_ab.normalize(); // local x
                                               // let parent_y_axis = Vec2::new(-parent_x_axis.y, parent_x_axis.x);    // local y (left turn)
    let perpendicular_rotation = Vec2::new((PI * 0.5).cos(), 1.0);
    let parent_y_axis = parent_x_axis.rotate(perpendicular_rotation); //rotate PI/2

    let child_a = child_local_vertices[aligned_child_edge.0 as usize];
    let child_b = child_local_vertices[aligned_child_edge.1 as usize];
    let child_x_axis = (child_b - child_a).normalize();
    //cosine eventually derives the angle of rotation in the rotation matrix
    let cosine_rotation = child_x_axis.dot(parent_x_axis).clamp(-1.0, 1.0);
    //sine eventually derives the sign/direction (CW or CCW) of the rotation
    let sine_rotation = child_x_axis.perp_dot(parent_x_axis);
    let rotation = Vec2::new(cosine_rotation, sine_rotation);
    let child_edge_1 = child_local_vertices[0] - child_a;
    let child_edge_2 = child_local_vertices[1] - child_a;
    let child_edge_3 = child_local_vertices[2] - child_a;

    let aligned_child_a = parent_a + child_edge_1.rotate(rotation);
    let aligned_child_b = parent_a + child_edge_2.rotate(rotation);
    let aligned_child_c = parent_a + child_edge_3.rotate(rotation);
    let mut aligned_child_face = [aligned_child_a, aligned_child_b, aligned_child_c];
    let parent_c = parent_vertices[edge_opposing_vertex(aligned_parent_edge) as usize];
    let child_c = aligned_child_face[edge_opposing_vertex(aligned_child_edge) as usize];
    let parent_sign = parent_ab.perp_dot(parent_c - parent_a);
    let child_sign = parent_ab.perp_dot(child_c - parent_a);
    //cases: 0) either negative -> negative. 1) Both positive = positive -> flip the child 2) both negative = positive -> flip the child
    // if (parent_sign * child_sign).is_sign_positive() {
    if (parent_sign * child_sign) > 1e-6 {
        for child_vertex in &mut aligned_child_face {
            let parent_offset = *child_vertex - parent_a;
            let x_aligned_magnitude = parent_offset.dot(parent_x_axis);
            let y_aligned_magnitude = parent_offset.dot(parent_y_axis);
            let x_aligned_component = parent_x_axis * x_aligned_magnitude;
            let y_aligned_component = parent_y_axis * -y_aligned_magnitude;
            *child_vertex = parent_a + x_aligned_component + y_aligned_component;
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
    let ab_x_magnitude = ab.length();
    let ac = c - a;
    let x_axis_dir = ab.normalize_or_zero();
    let ac_x_magnitude = ac.dot(x_axis_dir);
    let ac_x_component = ac_x_magnitude * x_axis_dir;
    let ac_y_component = ac - ac_x_component;
    let y_axis_dir = ac_y_component.normalize_or_zero();
    let ac_y_magnitude = ac.dot(y_axis_dir);
    let a_local = Vec2::new(0.0, 0.0);
    let b_local = Vec2::new(ab_x_magnitude, 0.0);
    let c_local = Vec2::new(ac_x_magnitude, ac_y_magnitude);
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
    let ab_x_magnitude = ab.length();
    if ab_x_magnitude <= 0.0 {
        return face;
    }
    //TODO: why not normalize this instead???? im confused now again jesus
    let x_axis = ab.normalize_or_zero();
    // TODO: this is not perfect?
    let perpendicular_rotation = Vec2::new((PI * 0.5).cos(), 1.0);
    let y_axis = x_axis.rotate(perpendicular_rotation);
    // let y_axis = Vec2::new(-x_axis.y, x_axis.x);
    let ac = c - a;
    let ac_x_magnitude = ac.dot(x_axis);
    let ac_y_magnitude = ac.dot(y_axis);

    let stable_a = Vec2::new(0.0, 0.0);
    let stable_b = Vec2::new(ab_x_magnitude, 0.0);
    let stable_c = Vec2::new(ac_x_magnitude, ac_y_magnitude);
    let mut stable_welded_face = [Vec2::ZERO; 3];
    stable_welded_face[smallest_index] = stable_a;
    stable_welded_face[second_smallest_index] = stable_b;
    stable_welded_face[largest_index] = stable_c;
    stable_welded_face
}

#[inline]
fn face_normal(a: Vec3, b: Vec3, c: Vec3) -> Vec3 {
    (b - a).cross(c - a).normalize_or_zero()
}

#[inline]
fn quantize(x: f32) -> i32 {
    const WELD_VERTEX_EPSILON: f32 = 1e-5; // -1 and up works, 0 goes crazy
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

const FOLD_DURATION_SEC: f32 = 5.0;
const FOLD_UNFOLD_DURATION: f32 = FOLD_DURATION_SEC * 2.0;

pub fn fold(mesh: &mut WeakMesh, i_time: f32, repeat_fold_unfold: bool) -> Mesh {
    let fold_progress = if repeat_fold_unfold {
        fold_unfold_time(i_time, FOLD_UNFOLD_DURATION) // 0→1→0 loop
    } else {
        (i_time / FOLD_DURATION_SEC).clamp(0.0, 1.0) // one-way fold, then stay folded
    };
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
    let mut lifted_faces = vec![[Vec3::ZERO; 3]; face_count];
    for face in 0..face_count {
        lifted_faces[face][0] = lift_dimension(unfolded_faces[face][0]);
        lifted_faces[face][1] = lift_dimension(unfolded_faces[face][1]);
        lifted_faces[face][2] = lift_dimension(unfolded_faces[face][2]);
    }

    let mut dihedral_angles = vec![0.0f32; face_count];
    for child_face in 0..face_count {
        if let Some(parent_face) = parent_links[child_face].parent {
            let link = parent_links[child_face];
            dihedral_angles[child_face] = signed_dihedral_between(
                parent_face.id,
                child_face,
                link.parent_local_edge.unwrap(),
                &welded_mesh,
            );
        }
    }

    let root_faces: Vec<usize> = (0..face_count)
        .filter(|&face| parent_links[face].parent.is_none())
        .collect();
    for &root_face in &root_faces {
        let mut root_face_queue = VecDeque::new();
        root_face_queue.push_back(root_face);
        while let Some(face) = root_face_queue.pop_front() {
            for &child_face in &children[face] {
                // hinge endpoints on the *current* parent triangle
                let parent_link = parent_links[child_face.id];
                let parent_edge = parent_link.parent_local_edge.unwrap();
                let parent_a = lifted_faces[face][parent_edge.0 as usize];
                let parent_b = lifted_faces[face][parent_edge.1 as usize];

                let parent_ab = parent_b - parent_a;
                if parent_ab.length() > 1e-8 {
                    let current_fold_angle = dihedral_angles[child_face.id] * fold_progress;
                    // rotate entire child subtree
                    let children_subtree = collect_subtree_faces(child_face.id, &children);
                    for child in children_subtree {
                        for i in 0..3 {
                            lifted_faces[child][i] = rotate_point_about_axis(
                                lifted_faces[child][i],
                                (parent_a, parent_b),
                                current_fold_angle,
                            );
                        }
                    }
                }
                root_face_queue.push_back(child_face.id);
            }
        }
        align_to_original_pose(&mut lifted_faces, &welded_mesh, root_face);
    }

    let mut vertices = Vec::with_capacity(face_count * 9);
    let mut texcoords = Vec::with_capacity(face_count * 6);
    let mut indices = Vec::with_capacity(face_count * 3);

    for face in 0..face_count {
        for i in 0..3 {
            let vertex = lifted_faces[face][i];
            vertices.extend_from_slice(&[vertex.x, vertex.y, vertex.z]);
            let texcoord = welded_mesh.texcoords[face][i];
            texcoords.extend_from_slice(&[texcoord.x, texcoord.y]);
            indices.push((vertices.len() / 3 - 1) as u16);
        }
    }

    let mut folded_mesh: Mesh = unsafe { zeroed() };
    folded_mesh.vertexCount = (vertices.len() / 3) as i32;
    folded_mesh.triangleCount = (indices.len() / 3) as i32;
    folded_mesh.vertices = Box::leak(vertices.into_boxed_slice()).as_mut_ptr();
    folded_mesh.indices = Box::leak(indices.into_boxed_slice()).as_mut_ptr();
    folded_mesh.texcoords = Box::leak(texcoords.into_boxed_slice()).as_mut_ptr();
    folded_mesh.normals = null_mut();
    folded_mesh.tangents = null_mut();
    folded_mesh.colors = null_mut();
    folded_mesh
}

fn collect_subtree_faces(root_face: usize, children: &[Vec<Face>]) -> Vec<usize> {
    let mut subtree = Vec::new();
    let mut face_stack = vec![root_face];
    while let Some(face) = face_stack.pop() {
        subtree.push(face);
        for &child_face in &children[face] {
            face_stack.push(child_face.id);
        }
    }
    subtree
}

fn signed_dihedral_between(parent_id: usize, child_id: usize, parent_edge_local: (u8, u8), welded: &WeldedMesh) -> f32 {
    let [parent_a_world, parent_b_world, parent_c_world] = welded.original_vertices[parent_id];
    let [child_a, child_b, child_c] = welded.original_vertices[child_id];
    let parent_face_normal = face_normal(parent_a_world, parent_b_world, parent_c_world);
    let child_face_normal = face_normal(child_a, child_b, child_c);
    let parent_face_world = [parent_a_world, parent_b_world, parent_c_world];
    let parent_a_local = parent_face_world[parent_edge_local.0 as usize];
    let parent_b_local = parent_face_world[parent_edge_local.1 as usize];
    let x_axis = (parent_b_local - parent_a_local).normalize_or_zero();

    let cosine = parent_face_normal.dot(child_face_normal).clamp(-1.0, 1.0);
    let dihedral_angle = cosine.acos();
    let sine = x_axis.dot(parent_face_normal.cross(child_face_normal));
    // if sine >= 0.0 {
    //     dihedral_angle
    // } else {
    //     -dihedral_angle
    // }
    dihedral_angle * sine.signum()
}

fn align_to_original_pose(unfolded_faces_lifted: &mut [[Vec3; 3]], welded_mesh: &WeldedMesh, root: usize) {
    let [unfolded_a, unfolded_b, unfolded_c] = unfolded_faces_lifted[root];
    let unfolded_x_axis = (unfolded_b - unfolded_a).normalize_or_zero();
    let unfolded_z_axis = face_normal(unfolded_a, unfolded_b, unfolded_c);
    let unfolded_y_axis = unfolded_z_axis.cross(unfolded_x_axis).normalize_or_zero();

    let [folded_a, folded_b, folded_c] = welded_mesh.original_vertices[root];
    let folded_x_axis = (folded_b - folded_a).normalize_or_zero();
    let folded_z_axis = face_normal(folded_a, folded_b, folded_c);
    let folded_y_axis = folded_z_axis.cross(folded_x_axis).normalize_or_zero();

    for unfolded_face in 0..unfolded_faces_lifted.len() {
        for i in 0..3 {
            let unfolded_edge = unfolded_faces_lifted[unfolded_face][i] - unfolded_a;
            let unfolded_x_magnitude = unfolded_edge.dot(unfolded_x_axis);
            let unfolded_y_magnitude = unfolded_edge.dot(unfolded_y_axis);
            let unfolded_z_magnitude = unfolded_edge.dot(unfolded_z_axis);
            unfolded_faces_lifted[unfolded_face][i] = folded_a
                + folded_x_axis * unfolded_x_magnitude
                + folded_y_axis * unfolded_y_magnitude
                + folded_z_axis * unfolded_z_magnitude;
        }
    }
}

#[inline]
fn fold_unfold_time(i_time: f32, period: f32) -> f32 {
    let u = (i_time / period).fract();
    if u <= 0.5 {
        u * 2.0
    } else {
        2.0 - 2.0 * u
    }
}

#[inline]
fn lift_dimension(vertex: Vec2) -> Vec3 {
    Vec3::new(vertex.x, vertex.y, 0.0)
}

#[inline]
fn rotate_point_about_axis(c: Vec3, axis: (Vec3, Vec3), theta: f32) -> Vec3 {
    let (a, b) = axis;
    let ab = b - a;
    let ab_axis_dir = ab.normalize_or_zero();
    let ac = c - a;
    let ac_z_component = ab_axis_dir.dot(ac) * ab_axis_dir; //local
    let ac_x_component = ac - ac_z_component; //local x axis of the triangles face
    let ac_y_component = ab_axis_dir.cross(ac_x_component); //local y axis of the triangles face
    let origin = a;
    let rotated_x_component = ac_x_component * theta.cos();
    let rotated_y_component = ac_y_component * theta.sin();
    //Z does not rotate?
    let rotated_c = rotated_x_component + rotated_y_component + ac_z_component;
    origin + rotated_c
}
