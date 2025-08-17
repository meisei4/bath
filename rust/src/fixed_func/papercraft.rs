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
// struct WeldedEdge(WeldedVertex, WeldedVertex);
struct WeldedEdge {
    vertex_a: WeldedVertex,
    vertex_b: WeldedVertex,
}

impl WeldedEdge {
    fn new(node_0: WeldedVertex, node_1: WeldedVertex) -> Self {
        if node_0.id <= node_1.id {
            // WeldedEdge(a, b)
            WeldedEdge {
                vertex_a: node_0,
                vertex_b: node_1,
            }
        } else {
            // WeldedEdge(b, a)
            WeldedEdge {
                vertex_a: node_1,
                vertex_b: node_0,
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
    welded_edge_link: Option<WeldedEdge>,
}

pub fn unfold(mesh: &mut WeakMesh) -> Mesh {
    let welded_mesh = weld_mesh(mesh);
    let face_count = welded_mesh.welded_faces.len();

    let mut dual_graph = build_dual_graph(&welded_mesh);
    let (parent_links, children) = build_parent_tree(face_count, &mut dual_graph);

    let mut vertices_per_face_local_space = Vec::with_capacity(face_count);
    for face_index in 0..face_count {
        let [vertex_a, vertex_b, vertex_c] = welded_mesh.original_vertices[face_index];
        vertices_per_face_local_space.push(compute_triangle_vertices_local_space(vertex_a, vertex_b, vertex_c));
    }
    let mut faces_placed_in_draw_space = vec![[Vec2::ZERO; 3]; face_count];
    let mut is_already_placed = vec![false; face_count];

    // TODO: simple row packing if multiple components (sphere should be single component)
    let mut cursor_x = 0.0f32;
    for id in 0..face_count {
        if is_already_placed[id] {
            continue;
        }
        faces_placed_in_draw_space[id] =
            canonicalize_triangle_vertices_2d_by_ids(vertices_per_face_local_space[id], &welded_mesh.welded_faces[id]); // faces_placed_in_draw_space[id] = vertices_per_face_local_space[id];
        is_already_placed[id] = true;

        let mut component_faces = vec![Face { id }];
        let mut face_stack = vec![Face { id }];

        while let Some(face) = face_stack.pop() {
            for &child_face in &children[face.id] {
                if is_already_placed[child_face.id] {
                    continue;
                }
                let parent_link = parent_links[child_face.id];
                let child_vertices_aligned_to_parent_edge = align_child_face_with_parent_in_draw_space(
                    &vertices_per_face_local_space[child_face.id],
                    &welded_mesh.welded_faces[child_face.id],
                    &faces_placed_in_draw_space[face.id],
                    &welded_mesh.welded_faces[face.id],
                    parent_link.parent_local_edge.unwrap(),
                    parent_link.child_local_edge.unwrap(),
                    parent_link.welded_edge_link.unwrap(),
                );
                faces_placed_in_draw_space[child_face.id] = child_vertices_aligned_to_parent_edge;
                is_already_placed[child_face.id] = true;
                face_stack.push(child_face);
                component_faces.push(child_face);
            }
        }
        // translate component so it doesn't overlap others if there are multiple components
        let (mut min_x, mut min_y, mut max_x, mut max_y) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
        for &component_face in &component_faces {
            for position_in_draw_space in &faces_placed_in_draw_space[component_face.id] {
                min_x = min_x.min(position_in_draw_space.x);
                max_x = max_x.max(position_in_draw_space.x);
                min_y = min_y.min(position_in_draw_space.y);
                max_y = max_y.max(position_in_draw_space.y);
            }
        }

        let step_width = max_x - min_x;
        let offset = Vec2::new(cursor_x - min_x, -min_y);
        for &component_face in &component_faces {
            for face_position_in_draw_space in &mut faces_placed_in_draw_space[component_face.id] {
                *face_position_in_draw_space += offset;
            }
        }
        cursor_x += step_width;
    }

    let mut unfolded_vertices = Vec::with_capacity(face_count * 9);
    let mut unfolded_texcoords = Vec::with_capacity(face_count * 6);
    let mut unfolded_indices = Vec::with_capacity(face_count * 3);

    for face in 0..face_count {
        let triangle = faces_placed_in_draw_space[face];
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

    let mut welded_edge_to_owner_face_and_owner_local_edge: HashMap<WeldedEdge, (Face, (u8, u8))> = HashMap::new();
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

            if let Some(&(owner_face, owner_face_local_edge)) =
                welded_edge_to_owner_face_and_owner_local_edge.get(&edge)
            {
                let normal_face_a = face_normals[owner_face.id];
                let normal_face_b = face_normals[id];
                let cosine = normal_face_a.dot(normal_face_b).clamp(-1.0, 1.0);
                let dihedral_angle = cosine.acos();
                let fold_weight = PI - dihedral_angle;

                dual_graph.push(DualEdge {
                    face_a: owner_face,
                    face_b: face_id,
                    welded_edge: edge,
                    face_a_local_edge: owner_face_local_edge,
                    face_b_local_edge: (point_a, point_b),
                    fold_weight,
                });
            } else {
                welded_edge_to_owner_face_and_owner_local_edge.insert(edge, (face_id, (point_a, point_b)));
            }
        }
    }
    dual_graph
}

fn build_parent_tree(face_count: usize, dual_graph: &mut [DualEdge]) -> (Vec<ParentLink>, Vec<Vec<Face>>) {
    // dual_graph.sort_by(|left, right| right.fold_weight.partial_cmp(&left.fold_weight).unwrap());
    dual_graph.sort_by(|left, right| edge_sort_key(left).cmp(&edge_sort_key(right)));
    let mut dsu = DisjointSetUnion::new(face_count);
    let mut adjacency_list = vec![Vec::new(); face_count];

    for edge in dual_graph.iter().copied() {
        if dsu.union(edge.face_a.id, edge.face_b.id) {
            adjacency_list[edge.face_a.id].push((edge.face_b, edge));
            adjacency_list[edge.face_b.id].push((edge.face_a, edge));
        }
    }
    for neighbors in &mut adjacency_list {
        neighbors.sort_by_key(|(f, e)| (f.id, e.welded_edge.vertex_a.id, e.welded_edge.vertex_b.id));
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
                    welded_edge_link: Some(edge.welded_edge),
                };
                children[current_face.id].push(face);
                face_queue.push_back(face);
            }
        }
    }
    (parent_links, children)
}

fn order_local_edge_with_welded_edge(
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

fn align_child_face_with_parent_in_draw_space(
    child_local_space_vertices: &[Vec2; 3],
    child_face_welded_vertices: &[WeldedVertex; 3],
    parent_draw_space_vertices: &[Vec2; 3],
    parent_face_welded_vertices: &[WeldedVertex; 3],
    parent_edge_local: (u8, u8),
    child_edge_local: (u8, u8),
    welded_edge_link: WeldedEdge,
) -> [Vec2; 3] {
    // 1) reorder endpoints on BOTH faces to match the welded_edge_link order (min, max)
    let parent_edge =
        order_local_edge_with_welded_edge(parent_face_welded_vertices, parent_edge_local, welded_edge_link);
    let child_edge = order_local_edge_with_welded_edge(child_face_welded_vertices, child_edge_local, welded_edge_link);

    // 2) fetch endpoints in those (min,max) orders
    let parent_vertex_a = parent_draw_space_vertices[parent_edge.0 as usize];
    let parent_vertex_b = parent_draw_space_vertices[parent_edge.1 as usize];
    let parent_vector_ab = parent_vertex_b - parent_vertex_a;

    let child_vertex_a = child_local_space_vertices[child_edge.0 as usize];
    let child_vertex_b = child_local_space_vertices[child_edge.1 as usize];
    let child_vector_ab = child_vertex_b - child_vertex_a;

    // 3) rotate (no scale) to align child edge direction with parent edge direction
    let parent_edge_vector_length = parent_vector_ab.length();
    let child_edge_vector_length = child_vector_ab.length();

    // Guard against degenerate lengths
    let parent_edge_alignment_direction = if parent_edge_vector_length > 0.0 {
        parent_vector_ab / parent_edge_vector_length
    } else {
        Vec2::new(1.0, 0.0)
    };
    let current_child_edge_direction = if child_edge_vector_length > 0.0 {
        child_vector_ab / child_edge_vector_length
    } else {
        Vec2::new(1.0, 0.0)
    };

    let cosine = current_child_edge_direction
        .dot(parent_edge_alignment_direction)
        .clamp(-1.0, 1.0);
    let sine = cross_2d(current_child_edge_direction, parent_edge_alignment_direction);
    let rotation = |direction_vector: Vec2| -> Vec2 {
        Vec2::new(
            cosine * direction_vector.x - sine * direction_vector.y,
            sine * direction_vector.x + cosine * direction_vector.y,
        )
    };

    // 4) translate so child edge lands on parent edge and then rotate around that point
    let mut child_draw_space_vertices = [Vec2::ZERO; 3];
    for i in 0..3 {
        let offset = child_local_space_vertices[i] - child_vertex_a;
        child_draw_space_vertices[i] = parent_vertex_a + rotation(offset);
    }

    // 5) Robust mirror test with epsilon (avoid signum jitter)
    let eps = 1.0e-6_f32;

    let parent_third_vertex = parent_draw_space_vertices[opposite_vertex_from_edge(parent_edge) as usize];
    let child_third_vertex = child_draw_space_vertices[opposite_vertex_from_edge(child_edge) as usize];

    let parent_side = cross_2d(parent_vector_ab, parent_third_vertex - parent_vertex_a); // signed area
    let child_side = cross_2d(parent_vector_ab, child_third_vertex - parent_vertex_a);

    // If the child is clearly on the same side as the parent's third vertex, mirror it.
    // Ignore cases where the child is almost on the hinge line to avoid flip-flop.
    if (parent_side * child_side) > eps {
        let parent_edge_normal = Vec2::new(-parent_edge_alignment_direction.y, parent_edge_alignment_direction.x);
        for vertex in &mut child_draw_space_vertices {
            let relative = *vertex - parent_vertex_a;
            let projection_along = relative.dot(parent_edge_alignment_direction);
            let projection_perp = relative.dot(parent_edge_normal);
            *vertex = parent_vertex_a + projection_along * parent_edge_alignment_direction
                - projection_perp * parent_edge_normal;
        }
    }

    child_draw_space_vertices
}

fn compute_triangle_vertices_local_space(vertex_a: Vec3, vertex_b: Vec3, vertex_c: Vec3) -> [Vec2; 3] {
    let vector_ab = vertex_b - vertex_a;
    let vector_ac = vertex_c - vertex_a;
    let vector_ab_normal = vector_ab.normalize_or_zero();
    let ac_u = vector_ac.dot(vector_ab_normal);
    let v = (vector_ac - ac_u * vector_ab_normal).normalize_or_zero(); // orthonormal y-axis
    [
        Vec2::new(0.0, 0.0),
        Vec2::new(vector_ab.length(), 0.0),
        Vec2::new(ac_u, vector_ac.dot(v)),
    ]
}

fn edge_sort_key(e: &DualEdge) -> (u32, u32, usize, usize) {
    let a = e.welded_edge.vertex_a.id;
    let b = e.welded_edge.vertex_b.id;
    let fa = e.face_a.id.min(e.face_b.id);
    let fb = e.face_a.id.max(e.face_b.id);
    (a, b, fa, fb)
}

fn canonicalize_triangle_vertices_2d_by_ids(tri: [Vec2; 3], welded_face: &[WeldedVertex; 3]) -> [Vec2; 3] {
    // pick base edge by stable rule: two smallest welded vertex ids in this face
    let mut idx = [0usize, 1, 2];
    idx.sort_by_key(|&i| welded_face[i].id); // ascending by welded id
    let ia = idx[0];
    let ib = idx[1];
    let ic = idx[2];

    let p = tri[ia];
    let q = tri[ib];
    let r = tri[ic];

    let e = q - p;
    let len = e.length();
    if len <= 0.0 {
        return tri;
    }
    let ex = e / len;
    let ey = Vec2::new(-ex.y, ex.x); // left-normal

    let rp = r - p;
    let r2 = Vec2::new(rp.dot(ex), rp.dot(ey));

    let p2 = Vec2::new(0.0, 0.0);
    let q2 = Vec2::new(len, 0.0);
    let r2 = if r2.y >= 0.0 { r2 } else { Vec2::new(r2.x, -r2.y) };

    // write back preserving original vertex indices 0,1,2
    let mut out = [Vec2::ZERO; 3];
    out[ia] = p2;
    out[ib] = q2;
    out[ic] = r2;
    out
}

#[inline]
fn face_normal(vertex_a: Vec3, vertex_b: Vec3, vertex_c: Vec3) -> Vec3 {
    (vertex_b - vertex_a).cross(vertex_c - vertex_a).normalize_or_zero()
}

#[inline]
fn quantize(x: f32) -> i32 {
    //TODO: this is to define when a vertex is identical between two faces?
    const WELD_VERTEX_EPSILON: f32 = 1e-5; // -1 and up works, 0 goes crazy
    (x / WELD_VERTEX_EPSILON).round() as i32
}

#[inline]
fn cross_2d(a: Vec2, b: Vec2) -> f32 {
    a.x * b.y - a.y * b.x
}

#[inline]
fn opposite_vertex_from_edge(edge: (u8, u8)) -> u8 {
    3 - (edge.0 + edge.1)
}

#[inline]
fn eq(a: WeldedVertex, b: WeldedVertex) -> bool {
    a.id == b.id
}
