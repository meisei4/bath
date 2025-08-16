use raylib::math::glam::{Vec2, Vec3};
use raylib::models::{Mesh, WeakMesh};
use std::collections::{HashMap, VecDeque};
use std::f32::consts::PI;
use std::mem::{swap, zeroed};
use std::ptr::null_mut;
use std::slice::from_raw_parts;

// ===== tunables =====
pub const TARGET_MAX_EXTENT: f32 = 1.9;
const WELD_POS_EPS: f32 = 1e-5; // weld positions only, NOT UVs

// ===== typed ids =====
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
struct TopologicalVertexId(u32); // welded-by-position id
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
struct FaceId(usize);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
struct Edge(TopologicalVertexId, TopologicalVertexId);
impl Edge {
    fn new(a: TopologicalVertexId, b: TopologicalVertexId) -> Self {
        if a.0 <= b.0 {
            Edge(a, b)
        } else {
            Edge(b, a)
        }
    }
}

// ===== topological mesh built from source buffers =====
struct TopologicalMesh {
    // welded unique positions (by position only)
    vertices_topological_space: Vec<Vec3>,
    // faces using welded position ids (3 corners each)
    faces: Vec<[TopologicalVertexId; 3]>,
    // per-face per-corner UVs copied from source (independent of welding)
    face_uvs: Vec<[Vec2; 3]>,
    // original positions per corner (for exact local 2D)
    vertices_local_space: Vec<[Vec3; 3]>,
}

// ===== dual graph with dihedral weights =====
#[derive(Copy, Clone, Debug)]
struct DualEdge {
    face_a: FaceId,
    face_b: FaceId,
    edge: Edge,
    face_a_local_space: (u8, u8),
    face_b_local_space: (u8, u8),
    weight: f32, // PI - dihedral
}

// ===== maximum spanning tree + oriented parent links =====
struct DSU {
    parents: Vec<usize>,
    rank: Vec<u8>, //TODO: rank is how many children nodes per parent. I DONT LIKE PARALLEL ARRAYS!!
}
impl DSU {
    fn new(n: usize) -> Self {
        Self {
            parents: (0..n).collect(),
            rank: vec![0; n],
        }
    }
    fn find(&mut self, x: usize) -> usize {
        if self.parents[x] != x {
            self.parents[x] = self.find(self.parents[x]);
        }
        self.parents[x]
    }
    fn union(&mut self, a: usize, b: usize) -> bool {
        let (mut a, mut b) = (self.find(a), self.find(b));
        if a == b {
            return false;
        }
        if self.rank[a] < self.rank[b] {
            swap(&mut a, &mut b);
        }
        self.parents[b] = a;
        if self.rank[a] == self.rank[b] {
            self.rank[a] += 1;
        }
        true
    }
}

#[derive(Copy, Clone, Debug, Default)]
struct ParentLink {
    parent: Option<FaceId>,
    via_edge: Option<Edge>,
    parent_local: Option<(u8, u8)>,
    child_local: Option<(u8, u8)>,
}

pub fn unfold(mesh: &mut WeakMesh) -> Mesh {
    // 1) build topological mesh from the source (do NOT mutate source buffers)
    let topological_mesh = build_topology_from_mesh(mesh);

    // 2) dual graph + MST
    let mut dual_graph = build_dual_edges(&topological_mesh);
    let (parent_links, children) = build_parent_tree(topological_mesh.faces.len(), &mut dual_graph);

    // 3) local 2D per face
    let mut vertices_per_face_local_space = Vec::with_capacity(topological_mesh.faces.len());
    for face_index in 0..topological_mesh.faces.len() {
        let [vertex_a, vertex_b, vertex_c] = topological_mesh.vertices_local_space[face_index];
        vertices_per_face_local_space.push(compute_triangle_vertices_local_space(vertex_a, vertex_b, vertex_c));
    }

    // 4) place faces for each connected component (pack components to avoid overlap)
    let face_count = topological_mesh.faces.len();
    let mut faces_placed_in_draw_space = vec![[Vec2::ZERO; 3]; face_count];
    let mut is_placed = vec![false; face_count];

    // simple row packing if multiple components (sphere should be single component)
    let mut cursor_x = 0.0f32;
    let padding = 0.05f32;

    for face in 0..face_count {
        if is_placed[face] {
            continue;
        }

        faces_placed_in_draw_space[face] = vertices_per_face_local_space[face];
        is_placed[face] = true;

        let mut component_faces = vec![FaceId(face)];

        let mut face_stack = vec![FaceId(face)];
        while let Some(face_id) = face_stack.pop() {
            for &child_face in &children[face_id.0] {
                if is_placed[child_face.0] {
                    continue;
                }
                let parent_link = parent_links[child_face.0];

                let placed_child = place_child_on_parent(
                    &vertices_per_face_local_space[child_face.0],
                    &faces_placed_in_draw_space[face_id.0],
                    &topological_mesh.faces[face_id.0],
                    &topological_mesh.faces[child_face.0],
                    parent_link.parent_local.unwrap(),
                    parent_link.child_local.unwrap(),
                    parent_link.via_edge.unwrap(),
                );

                faces_placed_in_draw_space[child_face.0] = placed_child;
                is_placed[child_face.0] = true;
                face_stack.push(child_face);
                component_faces.push(child_face);
            }
        }

        // translate component so it doesn't overlap others if there are multiple components
        let (mut minx, mut miny, mut maxx, mut maxy) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
        for &face_id in &component_faces {
            for position_in_draw_space in &faces_placed_in_draw_space[face_id.0] {
                minx = minx.min(position_in_draw_space.x);
                maxx = maxx.max(position_in_draw_space.x);
                miny = miny.min(position_in_draw_space.y);
                maxy = maxy.max(position_in_draw_space.y);
            }
        }
        let width = maxx - minx;

        let offset = Vec2::new(cursor_x - minx, -miny);
        for &face_id in &component_faces {
            for position_in_draw_space in &mut faces_placed_in_draw_space[face_id.0] {
                *position_in_draw_space += offset;
            }
        }
        cursor_x += width + padding;
    }

    // 5) emit triangle soup with UVs
    let mut unfolded_vertices: Vec<f32> = Vec::with_capacity(face_count * 9);
    let mut unfolded_texcoords: Vec<f32> = Vec::with_capacity(face_count * 6);
    let mut unfolded_indices: Vec<u16> = Vec::with_capacity(face_count * 3);

    for face in 0..face_count {
        let triangle = faces_placed_in_draw_space[face];
        for i in 0..3 {
            let vertex = triangle[i];
            unfolded_vertices.extend_from_slice(&[vertex.x, vertex.y, 0.0]);
            let texcoords = topological_mesh.face_uvs[face][i];
            unfolded_texcoords.extend_from_slice(&[texcoords.x, texcoords.y]);
            unfolded_indices.push((unfolded_vertices.len() / 3 - 1) as u16);
        }
    }

    // 6) center+uniform scale to TARGET_MAX_EXTENT
    let (mut minx, mut miny, mut maxx, mut maxy) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    for i in (0..unfolded_vertices.len()).step_by(3) {
        let (x, y) = (unfolded_vertices[i], unfolded_vertices[i + 1]);
        minx = minx.min(x);
        maxx = maxx.max(x);
        miny = miny.min(y);
        maxy = maxy.max(y);
    }
    let (cx, cy) = (0.5 * (minx + maxx), 0.5 * (miny + maxy));
    let (sx, sy) = (maxx - minx, maxy - miny);
    let s = TARGET_MAX_EXTENT / sx.max(sy).max(1e-20);
    for i in (0..unfolded_vertices.len()).step_by(3) {
        unfolded_vertices[i] = (unfolded_vertices[i] - cx) * s;
        unfolded_vertices[i + 1] = (unfolded_vertices[i + 1] - cy) * s;
    }

    // 7) build raylib Mesh (triangle soup)
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

fn build_topology_from_mesh(mesh: &WeakMesh) -> TopologicalMesh {
    let triangle_count = mesh.triangleCount as usize;
    let vertex_count = mesh.vertexCount as usize;
    let src_vertices = unsafe { from_raw_parts(mesh.vertices, 3 * vertex_count) };
    let src_texcoords = unsafe { from_raw_parts(mesh.texcoords, 2 * vertex_count) };

    // if indices are null, the mesh is triangle-soup: triplets of vertices form faces
    let src_indices: Vec<u16> = if mesh.indices.is_null() {
        debug_assert!(vertex_count % 3 == 0, "non-indexed mesh must be triangle soup");
        (0..vertex_count as u16).collect()
    } else {
        unsafe { from_raw_parts(mesh.indices, 3 * triangle_count) }.to_vec()
    };

    // weld by POSITION ONLY to build topological vertex ids
    let mut quantized_vertex_to_topo_vertex_id_map: HashMap<(i32, i32, i32), TopologicalVertexId> = HashMap::new();
    let mut vertices_topological_space: Vec<Vec3> = Vec::new();

    let mut faces: Vec<[TopologicalVertexId; 3]> = Vec::with_capacity(src_indices.len() / 3);
    let mut face_uvs: Vec<[Vec2; 3]> = Vec::with_capacity(src_indices.len() / 3);
    let mut vertices_local_space: Vec<[Vec3; 3]> = Vec::with_capacity(src_indices.len() / 3);

    for face_index in 0..(src_indices.len() / 3) {
        let mut face_topo_vertex_ids = [TopologicalVertexId(0); 3];
        let mut face_texcoords = [Vec2::ZERO; 3];
        let mut face_vertices = [Vec3::ZERO; 3];

        for i in 0..3 {
            let vertex_index = src_indices[face_index * 3 + i] as usize;

            let vertex_x = src_vertices[vertex_index * 3 + 0];
            let vertex_y = src_vertices[vertex_index * 3 + 1];
            let vertex_z = src_vertices[vertex_index * 3 + 2];
            let vertex = Vec3::new(vertex_x, vertex_y, vertex_z);

            let quantized_vertex = (
                quantize(vertex_x, WELD_POS_EPS),
                quantize(vertex_y, WELD_POS_EPS),
                quantize(vertex_z, WELD_POS_EPS),
            );
            let topological_vertex_id = *quantized_vertex_to_topo_vertex_id_map
                .entry(quantized_vertex)
                .or_insert_with(|| {
                    let id = TopologicalVertexId(vertices_topological_space.len() as u32);
                    vertices_topological_space.push(vertex);
                    id
                });

            face_topo_vertex_ids[i] = topological_vertex_id;
            face_vertices[i] = vertex;

            let u = src_texcoords[vertex_index * 2 + 0];
            let v = src_texcoords[vertex_index * 2 + 1];
            face_texcoords[i] = Vec2::new(u, v);
        }

        faces.push(face_topo_vertex_ids);
        face_uvs.push(face_texcoords);
        vertices_local_space.push(face_vertices);
    }

    TopologicalMesh {
        vertices_topological_space,
        faces,
        face_uvs,
        vertices_local_space,
    }
}

fn build_dual_edges(topo_mesh: &TopologicalMesh) -> Vec<DualEdge> {
    let face_count = topo_mesh.faces.len();

    // cache face normals from original positions (not welded)
    let mut normals = Vec::with_capacity(face_count);
    for face_index in 0..face_count {
        let [vertex_a, vertex_b, vertex_c] = topo_mesh.vertices_local_space[face_index];
        normals.push(face_normal(vertex_a, vertex_b, vertex_c));
    }

    // map topological edge -> (owner face, owner local edge)
    let mut owner: HashMap<Edge, (FaceId, (u8, u8))> = HashMap::new();
    let mut dual = Vec::new();

    for face_index in 0..face_count {
        let face_id = FaceId(face_index);
        let [v0, v1, v2] = topo_mesh.faces[face_index];
        let local = [(0u8, 1u8), (1, 2), (2, 0)];
        let vids = [v0, v1, v2];

        for &(i0, i1) in &local {
            let key = Edge::new(vids[i0 as usize], vids[i1 as usize]);

            if let Some(&(of, ol)) = owner.get(&key) {
                let na = normals[of.0];
                let nb = normals[face_index];
                let cos = na.dot(nb).clamp(-1.0, 1.0);
                let dihedral = cos.acos();
                let weight = PI - dihedral;

                dual.push(DualEdge {
                    face_a: of,
                    face_b: face_id,
                    edge: key,
                    face_a_local_space: ol,
                    face_b_local_space: (i0, i1),
                    weight,
                });
            } else {
                owner.insert(key, (face_id, (i0, i1)));
            }
        }
    }

    dual
}

fn build_parent_tree(face_count: usize, dual: &mut [DualEdge]) -> (Vec<ParentLink>, Vec<Vec<FaceId>>) {
    // maximum spanning tree
    dual.sort_by(|left, right| right.weight.partial_cmp(&left.weight).unwrap());

    let mut dsu = DSU::new(face_count);
    let mut adj = vec![Vec::new(); face_count];

    for edge in dual.iter().copied() {
        if dsu.union(edge.face_a.0, edge.face_b.0) {
            adj[edge.face_a.0].push((edge.face_b, edge));
            adj[edge.face_b.0].push((edge.face_a, edge));
        }
    }

    // orient from each unseen root (handle possible multiple components)
    let mut links = vec![ParentLink::default(); face_count];
    let mut children = vec![Vec::new(); face_count];
    let mut seen = vec![false; face_count];
    let mut queue = VecDeque::new();

    for start in 0..face_count {
        if seen[start] {
            continue;
        }
        seen[start] = true;
        queue.push_back(FaceId(start));
        while let Some(u) = queue.pop_front() {
            for &(v, e) in &adj[u.0] {
                if seen[v.0] {
                    continue;
                }
                seen[v.0] = true;
                // orient u -> v
                let (pl, cl) = if u == e.face_a {
                    (e.face_a_local_space, e.face_b_local_space)
                } else {
                    (e.face_b_local_space, e.face_a_local_space)
                };
                links[v.0] = ParentLink {
                    parent: Some(u),
                    via_edge: Some(e.edge),
                    parent_local: Some(pl),
                    child_local: Some(cl),
                };
                children[u.0].push(v);
                queue.push_back(v);
            }
        }
    }

    (links, children)
}

// ===== exact per-face local 2D and rigid placement =====
fn compute_triangle_vertices_local_space(vertex_a: Vec3, vertex_b: Vec3, vertex_c: Vec3) -> [Vec2; 3] {
    let ab = vertex_b - vertex_a;
    let ac = vertex_c - vertex_a;
    let u = ab.normalize_or_zero();
    let ac_u = ac.dot(u);
    let v = (ac - ac_u * u).normalize_or_zero(); // orthonormal y-axis
    [
        Vec2::new(0.0, 0.0),
        Vec2::new(ab.length(), 0.0),
        Vec2::new(ac_u, ac.dot(v)),
    ]
}

#[inline]
fn perp_dot(a: Vec2, b: Vec2) -> f32 {
    a.x * b.y - a.y * b.x
}

#[inline]
fn third_of(edge: (u8, u8)) -> u8 {
    3 - (edge.0 + edge.1)
}

#[inline]
fn same_vid(a: TopologicalVertexId, b: TopologicalVertexId) -> bool {
    a.0 == b.0
}

#[inline]
fn quantize(x: f32, eps: f32) -> i32 {
    (x / eps).round() as i32
}

#[inline]
fn face_normal(vertex_a: Vec3, vertex_b: Vec3, vertex_c: Vec3) -> Vec3 {
    (vertex_b - vertex_a).cross(vertex_c - vertex_a).normalize_or_zero()
}

/// Return the local corner indices (iA, iB) **ordered to match edge_key**:
/// index for edge_key.0 first, then index for edge_key.1.
/// Panics if the provided local pair doesn't actually reference the edge.
fn order_local_edge_to_key(face_vids: &[TopologicalVertexId; 3], local_edge: (u8, u8), edge_key: Edge) -> (u8, u8) {
    let (i0, i1) = (local_edge.0 as usize, local_edge.1 as usize);
    let a = face_vids[i0];
    let b = face_vids[i1];

    if same_vid(a, edge_key.0) && same_vid(b, edge_key.1) {
        (local_edge.0, local_edge.1)
    } else if same_vid(a, edge_key.1) && same_vid(b, edge_key.0) {
        (local_edge.1, local_edge.0)
    } else {
        panic!("local_edge does not match edge_key (bad adjacency / welding).");
    }
}

fn place_child_on_parent(
    child_local: &[Vec2; 3],
    parent_placed: &[Vec2; 3],
    parent_face_vids: &[TopologicalVertexId; 3],
    child_face_vids: &[TopologicalVertexId; 3],
    parent_edge_local: (u8, u8),
    child_edge_local: (u8, u8),
    edge_key: Edge,
) -> [Vec2; 3] {
    // 1) reorder endpoints on BOTH faces to match the edge_key order (min,max)
    let pe_local = order_local_edge_to_key(parent_face_vids, parent_edge_local, edge_key);
    let ce_local = order_local_edge_to_key(child_face_vids, child_edge_local, edge_key);

    // 2) fetch endpoints in those (min,max) orders
    let pa = parent_placed[pe_local.0 as usize];
    let pb = parent_placed[pe_local.1 as usize];
    let pe = pb - pa;

    let ca = child_local[ce_local.0 as usize];
    let cb = child_local[ce_local.1 as usize];
    let ce = cb - ca;

    // 3) rotate (no scale) to align child edge direction with parent edge direction
    let pe_len = pe.length().max(1e-20);
    let ce_len = ce.length().max(1e-20);
    debug_assert!(
        ((pe_len - ce_len) / pe_len).abs() < 1e-3,
        "shared edge lengths must match"
    );

    let u = pe / pe_len; // desired direction
    let cu = ce / ce_len; // current child direction
    let cos = cu.dot(u);
    let sin = perp_dot(cu, u);
    let rot = |p: Vec2| -> Vec2 { Vec2::new(cos * p.x - sin * p.y, sin * p.x + cos * p.y) };

    // 4) translate so child 'ca' lands on 'pa', rotate around that point
    let mut out = [Vec2::ZERO; 3];
    for i in 0..3 {
        let shifted = child_local[i] - ca;
        out[i] = pa + rot(shifted); // no scale
    }

    // 5) mirror across the hinge if the child ends up on the same side as the parent's third
    let p_third = parent_placed[third_of(pe_local) as usize];
    let c_third = out[third_of(ce_local) as usize];
    let side_parent = perp_dot(pe, p_third - pa).signum();
    let side_child = perp_dot(pe, c_third - pa).signum();
    if side_parent == side_child {
        let n = Vec2::new(-u.y, u.x); // perp(u)
        for v in &mut out {
            let rel = *v - pa;
            let along = rel.dot(u);
            let perpv = rel.dot(n);
            *v = pa + along * u - perpv * n;
        }
    }

    out
}
