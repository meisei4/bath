use crate::fixed_func::silhouette::rotate_vertices;
use raylib::camera::Camera3D;
use raylib::color::Color;
use raylib::drawing::{RaylibDraw, RaylibDraw3D, RaylibDrawHandle, RaylibMode3DExt};
use raylib::math::glam::{Vec2, Vec3};
use raylib::math::{Vector2, Vector3};
use raylib::models::{RaylibMesh, WeakMesh};
use std::collections::{HashMap, HashSet};
use std::slice::from_raw_parts;

pub struct Topology {
    pub all_triangles: Vec<[usize; 3]>,
    pub front_triangles: Option<HashSet<usize>>,
    pub back_triangles: Option<HashSet<usize>>,
    pub silhouette_triangles: Option<Vec<usize>>,
    pub welded_vertices: Vec<usize>,
    pub welded_vertices_per_triangle: Option<Vec<[WeldedVertex; 3]>>,
    pub neighbors_per_triangle: Option<Vec<[Option<usize>; 3]>>,
    pub triangle_vertex_positions_model: Option<Vec<[Vec3; 3]>>,
    pub triangle_texcoords: Option<Vec<[Vec2; 3]>>,
    pub vertex_normals: Option<Vec<Vec3>>,
    pub corner_angles_per_triangle: Option<Vec<[f32; 3]>>,
}
pub struct TopologyBuilder<'a> {
    topology: Topology,
    mesh: &'a WeakMesh,
}

impl Topology {
    #[inline]
    pub fn build_topology(mesh: &WeakMesh) -> TopologyBuilder<'_> {
        let vertices = mesh.vertices();
        let welded_vertices = collect_welded_vertices(vertices);
        TopologyBuilder {
            topology: Topology {
                all_triangles: Vec::new(),
                front_triangles: None,
                back_triangles: None,
                silhouette_triangles: None,
                welded_vertices,
                welded_vertices_per_triangle: None,
                neighbors_per_triangle: None,
                triangle_vertex_positions_model: None,
                triangle_texcoords: None,
                vertex_normals: None,
                corner_angles_per_triangle: None,
            },
            mesh,
        }
    }
}

impl<'a> TopologyBuilder<'a> {
    #[inline]
    pub fn collect_triangles(mut self) -> Self {
        self.topology.all_triangles = self.mesh.triangles().collect();
        self
    }

    #[inline]
    pub fn collect_welded_triangles(mut self) -> Self {
        if self.topology.welded_vertices_per_triangle.is_some() {
            return self;
        }
        let welded_ids = &self.topology.welded_vertices;
        let mut welded_vertices_per_triangle = Vec::with_capacity(self.topology.all_triangles.len());
        for [vertex_a_index, vertex_b_index, vertex_c_index] in self.topology.all_triangles.iter().copied() {
            let welded_vertex_a = WeldedVertex {
                id: welded_ids[vertex_a_index],
            };
            let welded_vertex_b = WeldedVertex {
                id: welded_ids[vertex_b_index],
            };
            let welded_vertex_c = WeldedVertex {
                id: welded_ids[vertex_c_index],
            };
            welded_vertices_per_triangle.push([welded_vertex_a, welded_vertex_b, welded_vertex_c]);
        }
        self.topology.welded_vertices_per_triangle = Some(welded_vertices_per_triangle);
        self
    }

    #[inline]
    pub fn collect_neighbors(mut self) -> Self {
        if self.topology.neighbors_per_triangle.is_some() {
            return self;
        }
        if self.topology.welded_vertices_per_triangle.is_none() {
            self = self.collect_welded_triangles();
        }
        let welded_triangles = self.topology.welded_vertices_per_triangle.as_ref().expect(
            "collect_neighbors: welded_vertices_per_triangle not present. Call collect_welded_triangles first.",
        );
        let mut neighbors: Vec<[Option<usize>; 3]> = vec![[None, None, None]; welded_triangles.len()];
        let mut first_owner: HashMap<WeldedEdge, (usize, usize)> = HashMap::new();
        for (triangle_id, welded_triplet) in welded_triangles.iter().copied().enumerate() {
            let edges: [(WeldedEdge, usize); 3] = [
                (WeldedEdge::new(welded_triplet[0], welded_triplet[1]), 0), // AB
                (WeldedEdge::new(welded_triplet[1], welded_triplet[2]), 1), // BC
                (WeldedEdge::new(welded_triplet[2], welded_triplet[0]), 2), // CA
            ];
            for (welded_edge, local_edge_index) in edges {
                if let Some(&(other_triangle_id, other_local_edge_index)) = first_owner.get(&welded_edge) {
                    neighbors[triangle_id][local_edge_index] = Some(other_triangle_id);
                    neighbors[other_triangle_id][other_local_edge_index] = Some(triangle_id);
                } else {
                    first_owner.insert(welded_edge, (triangle_id, local_edge_index));
                }
            }
        }
        self.topology.neighbors_per_triangle = Some(neighbors);
        self
    }

    #[inline]
    pub fn collect_vertex_positions_model(mut self) -> Self {
        if self.topology.triangle_vertex_positions_model.is_some() {
            return self;
        }
        let mut triangle_vertex_positions_model = Vec::with_capacity(self.mesh.triangle_count());
        let vertices = self.mesh.vertices();
        for [vertex_a_index, vertex_b_index, vertex_c_index] in self.mesh.triangles() {
            let triangle_abc_positions = [
                vertices[vertex_a_index],
                vertices[vertex_b_index],
                vertices[vertex_c_index],
            ];
            triangle_vertex_positions_model.push(triangle_abc_positions);
        }
        self.topology.triangle_vertex_positions_model = Some(triangle_vertex_positions_model);
        self
    }

    #[inline]
    pub fn collect_triangle_texcoords(mut self) -> Self {
        if self.topology.triangle_texcoords.is_some() {
            return self;
        }
        let mut triangle_texcoords = Vec::with_capacity(self.mesh.triangle_count());
        for [vertex_a_index, vertex_b_index, vertex_c_index] in self.mesh.triangles() {
            let mut triangle_abc_texcoords = [Vector2::ZERO, Vector2::ZERO, Vector2::ZERO];
            if let Some(texcoords) = self.mesh.texcoords() {
                triangle_abc_texcoords = [
                    texcoords[vertex_a_index],
                    texcoords[vertex_b_index],
                    texcoords[vertex_c_index],
                ];
            }
            triangle_texcoords.push(triangle_abc_texcoords);
        }
        self.topology.triangle_texcoords = Some(triangle_texcoords);
        self
    }

    #[inline]
    pub fn collect_vertex_normals(mut self) -> Self {
        collect_vertex_normals(&mut self.topology, self.mesh);
        self
    }

    #[inline]
    pub fn build(self) -> Topology {
        self.topology
    }
}

#[inline]
pub fn collect_triangles(topology: &mut Topology, mesh: &WeakMesh) -> Vec<[usize; 3]> {
    topology.all_triangles = mesh.triangles().collect();
    topology.all_triangles.clone()
}

#[inline]
pub fn collect_welded_triangles(topology: &mut Topology) {
    if topology.welded_vertices_per_triangle.is_some() {
        return;
    }
    let welded_ids = &topology.welded_vertices;
    let mut welded_vertices_per_triangle = Vec::with_capacity(topology.all_triangles.len());
    for [vertex_a_index, vertex_b_index, vertex_c_index] in topology.all_triangles.iter().copied() {
        let welded_vertex_a = WeldedVertex {
            id: welded_ids[vertex_a_index],
        };
        let welded_vertex_b = WeldedVertex {
            id: welded_ids[vertex_b_index],
        };
        let welded_vertex_c = WeldedVertex {
            id: welded_ids[vertex_c_index],
        };
        welded_vertices_per_triangle.push([welded_vertex_a, welded_vertex_b, welded_vertex_c]);
    }
    topology.welded_vertices_per_triangle = Some(welded_vertices_per_triangle);
}

pub fn collect_neighbors(topology: &mut Topology) {
    let welded_triangles = topology
        .welded_vertices_per_triangle
        .as_ref()
        .expect("collect_neighbors: welded_vertices_per_triangle not present. Call collect_welded_triangles first.");
    let mut neighbors: Vec<[Option<usize>; 3]> = vec![[None, None, None]; welded_triangles.len()];
    let mut first_owner: HashMap<WeldedEdge, (usize, u8)> = HashMap::new();
    for (triangle_id, welded_triplet) in welded_triangles.iter().copied().enumerate() {
        let edges: [(WeldedEdge, u8); 3] = [
            (WeldedEdge::new(welded_triplet[0], welded_triplet[1]), 0), // AB
            (WeldedEdge::new(welded_triplet[1], welded_triplet[2]), 1), // BC
            (WeldedEdge::new(welded_triplet[2], welded_triplet[0]), 2), // CA
        ];
        for (welded_edge, local_edge_index) in edges {
            if let Some(&(other_triangle_id, other_local_edge_index)) = first_owner.get(&welded_edge) {
                neighbors[triangle_id][local_edge_index as usize] = Some(other_triangle_id);
                neighbors[other_triangle_id][other_local_edge_index as usize] = Some(triangle_id);
            } else {
                first_owner.insert(welded_edge, (triangle_id, local_edge_index));
            }
        }
    }
    topology.neighbors_per_triangle = Some(neighbors);
}

pub fn collect_vertex_positions_model(topology: &mut Topology, mesh: &WeakMesh) {
    let mut triangle_positions_model = Vec::with_capacity(mesh.triangle_count());
    let vertices = mesh.vertices();
    for [vertex_a_index, vertex_b_index, vertex_c_index] in mesh.triangles() {
        let triangle_abc_positions = [
            vertices[vertex_a_index],
            vertices[vertex_b_index],
            vertices[vertex_c_index],
        ];
        triangle_positions_model.push(triangle_abc_positions);
    }
    topology.triangle_vertex_positions_model = Some(triangle_positions_model);
}

pub fn collect_triangle_texcoords(topology: &mut Topology, mesh: &WeakMesh) {
    let mut triangle_texcoords = Vec::with_capacity(mesh.triangle_count());
    for [vertex_a_index, vertex_b_index, vertex_c_index] in mesh.triangles() {
        let mut triangle_abc_texcoords = [Vector2::ZERO, Vector2::ZERO, Vector2::ZERO];
        if let Some(texcoords) = mesh.texcoords() {
            triangle_abc_texcoords = [
                texcoords[vertex_a_index],
                texcoords[vertex_b_index],
                texcoords[vertex_c_index],
            ];
        }
        triangle_texcoords.push(triangle_abc_texcoords);
    }
    topology.triangle_texcoords = Some(triangle_texcoords);
}

pub fn collect_front_triangles(topology: &mut Topology, mesh: &WeakMesh, rotation: f32, observer: &Camera3D) {
    let line_of_sight = observed_line_of_sight(observer);
    if let Some(triangle_positions_model) = &topology.triangle_vertex_positions_model {
        let mut front_triangles = HashSet::with_capacity(topology.all_triangles.len());
        for (triangle_id, [vertex_a, vertex_b, vertex_c]) in triangle_positions_model.iter().copied().enumerate() {
            let mut triangle = vec![vertex_a, vertex_b, vertex_c];
            rotate_vertices(&mut triangle, rotation);

            let normal = triangle_normal(triangle[0], triangle[1], triangle[2]);
            // const SILHOUETTE_triangle_BIAS: f32 = 0.1; // try 0.02..0.08
            // if normal.dot(line_of_sight) <= -SILHOUETTE_triangle_BIAS { front_triangles.insert(triangle_id); }
            if normal.dot(line_of_sight) <= 0.0 {
                front_triangles.insert(triangle_id);
            }
        }
        topology.front_triangles = Some(front_triangles);
        return;
    }
    let vertices = mesh.vertices();
    let mut front_triangles = HashSet::with_capacity(topology.all_triangles.len());
    for (triangle_id, [vertex_a_index, vertex_b_index, vertex_c_index]) in
        topology.all_triangles.iter().copied().enumerate()
    {
        let mut triangle = vec![
            vertices[vertex_a_index as usize],
            vertices[vertex_b_index as usize],
            vertices[vertex_c_index as usize],
        ];
        rotate_vertices(&mut triangle, rotation);
        let normal = triangle_normal(triangle[0], triangle[1], triangle[2]);
        if normal.dot(line_of_sight) <= 0.0 {
            front_triangles.insert(triangle_id);
        }
    }
    topology.front_triangles = Some(front_triangles);
}

pub fn collect_back_triangles(topology: &mut Topology) {
    let front_triangles = topology
        .front_triangles
        .as_ref()
        .expect("collect_back_triangles: front_triangles not present. Call collect_front_triangles first.");
    let triangle_count = topology.all_triangles.len();
    let mut back_triangles = HashSet::with_capacity(triangle_count.saturating_sub(front_triangles.len()));
    for triangle_id in 0..triangle_count {
        if !front_triangles.contains(&triangle_id) {
            back_triangles.insert(triangle_id);
        }
    }
    topology.back_triangles = Some(back_triangles);
}

pub fn collect_silhouette_triangles(topology: &mut Topology) {
    let neighbors_per_triangle = topology
        .neighbors_per_triangle
        .as_ref()
        .expect("collect_silhouette_triangles: neighbors_per_triangle not present. Call collect_neighbors first.");
    let front_triangles = topology
        .front_triangles
        .as_ref()
        .expect("collect_silhouette_triangles: front_triangles not present. Call collect_front_triangles first.");
    let welded_vertices_per_triangle = topology.welded_vertices_per_triangle.as_ref().expect(
        "collect_silhouette_triangles: welded_vertices_per_triangle not present. Call collect_welded_triangles first.",
    );
    let mut silhouette_triangles: HashSet<usize> = HashSet::new();
    for (triangle_id, welded_triplet) in welded_vertices_per_triangle.iter().copied().enumerate() {
        let local_edges = [(0, 1, 0), (1, 2, 1), (2, 0, 2)];
        for (vertex_0, vertex_1, local_edge_index) in local_edges {
            let neighbor_triangle_opt = neighbors_per_triangle[triangle_id][local_edge_index as usize];
            if neighbor_triangle_opt.is_none() {
                continue;
            }
            let neighbor_triangle_id = neighbor_triangle_opt.unwrap();
            let is_front_here = front_triangles.contains(&triangle_id);
            let is_front_neighbor = front_triangles.contains(&neighbor_triangle_id);
            if is_front_here == is_front_neighbor {
                continue;
            }
            let silhouette_triangle = if is_front_here {
                triangle_id
            } else {
                neighbor_triangle_id
            };
            silhouette_triangles.insert(silhouette_triangle);
            let vertices = welded_vertices_per_triangle[silhouette_triangle];
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
            if let Some(neighbor_0) = neighbors_per_triangle[silhouette_triangle][interior_edge_index_0] {
                if front_triangles.contains(&neighbor_0) {
                    silhouette_triangles.insert(neighbor_0);
                }
            }
            if let Some(neighbor_1) = neighbors_per_triangle[silhouette_triangle][interior_edge_index_1] {
                if front_triangles.contains(&neighbor_1) {
                    silhouette_triangles.insert(neighbor_1);
                }
            }
        }
    }
    let mut silhouette_vec: Vec<usize> = silhouette_triangles.into_iter().collect();
    silhouette_vec.sort_unstable();
    topology.silhouette_triangles = Some(silhouette_vec);
}

pub struct WeldedMesh {
    pub original_vertices: Vec<[Vec3; 3]>,
    pub welded_triangles: Vec<[WeldedVertex; 3]>,
    pub texcoords: Vec<[Vec2; 3]>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct WeldedVertex {
    pub id: usize,
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
    if topology.welded_vertices_per_triangle.is_none() {
        collect_welded_triangles(topology);
    }
    if topology.triangle_vertex_positions_model.is_none() {
        collect_vertex_positions_model(topology, mesh);
    }
    if topology.triangle_texcoords.is_none() {
        collect_triangle_texcoords(topology, mesh);
    }

    WeldedMesh {
        original_vertices: topology
            .triangle_vertex_positions_model
            .as_ref()
            .expect("triangle_vertex_positions_model missing")
            .clone(),
        welded_triangles: topology
            .welded_vertices_per_triangle
            .as_ref()
            .expect("welded_vertices_per_triangle missing")
            .clone(),
        texcoords: topology
            .triangle_texcoords
            .as_ref()
            .expect("triangle_texcoords missing")
            .clone(),
    }
}

pub fn collect_welded_vertices(original_vertices: &[Vector3]) -> Vec<usize> {
    let mut welded_vertices_to_triangles_map: HashMap<(i32, i32, i32), usize> = HashMap::new();
    let mut next_wid: usize = 0;
    let mut welded_id_per_vertex = vec![0usize; original_vertices.len()];
    for (i, v) in original_vertices.iter().enumerate() {
        let key = (quantize(v.x), quantize(v.y), quantize(v.z));
        let wid = *welded_vertices_to_triangles_map.entry(key).or_insert_with(|| {
            let id = next_wid;
            next_wid += 1;
            id
        });
        welded_id_per_vertex[i] = wid;
    }
    welded_id_per_vertex
}

pub fn build_edge_owner_map(triangles: &[[usize; 3]], welded_vertices: &[usize]) -> HashMap<WeldedEdge, usize> {
    let mut edge_owner: HashMap<WeldedEdge, usize> = HashMap::new();
    for (triangle, [vertex_a_index, vertex_b_index, vertex_c_index]) in triangles.iter().copied().enumerate() {
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
            edge_owner.entry(WeldedEdge::new(w0, w1)).or_insert(triangle);
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
pub fn triangle_normal(a: Vec3, b: Vec3, c: Vec3) -> Vec3 {
    (b - a).cross(c - a).normalize_or_zero()
}

#[inline]
pub fn triangle_normal_area_weighted(a: Vec3, b: Vec3, c: Vec3) -> Vec3 {
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
    // TODO: should this check or whatever even be in my project? otherwise in Mesh improvements in raylib-rs?
    let indices: Vec<u16> = if mesh.indices.is_null() {
        (0..vertex_count as u16).collect()
    } else {
        unsafe { from_raw_parts(mesh.indices, (mesh.triangleCount as usize) * 3) }.to_vec()
    };
    for triangle in indices.chunks_exact(3) {
        let vertex_a_index = triangle[0] as usize;
        let vertex_b_index = triangle[1] as usize;
        let vertex_c_index = triangle[2] as usize;
        let vertex_a = vertices[vertex_a_index];
        let vertex_b = vertices[vertex_b_index];
        let vertex_c = vertices[vertex_c_index];
        let triangle_normal = triangle_normal(vertex_a, vertex_b, vertex_c);
        accum[vertex_a_index] += triangle_normal;
        accum[vertex_b_index] += triangle_normal;
        accum[vertex_c_index] += triangle_normal;
    }
    // accum.into_iter().map(|triangle_normal| triangle_normal).collect()
    accum
}

pub fn collect_vertex_normals(topology: &mut Topology, mesh: &WeakMesh) {
    let vertices = mesh.vertices();
    let vertex_count = mesh.vertexCount as usize;
    let mut accumulated_vertex_normals = vec![Vec3::ZERO; vertex_count];
    let mut corner_angles: Vec<[f32; 3]> = Vec::with_capacity(mesh.triangle_count());
    for [vertex_a_index, vertex_b_index, vertex_c_index] in mesh.triangles() {
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

        let triangle_normal = triangle_normal_area_weighted(vertex_a, vertex_b, vertex_c); // area-weighted direction
        accumulated_vertex_normals[vertex_a_index] += triangle_normal * angle_a;
        accumulated_vertex_normals[vertex_b_index] += triangle_normal * angle_b;
        accumulated_vertex_normals[vertex_c_index] += triangle_normal * angle_c;
    }

    topology.corner_angles_per_triangle = Some(corner_angles);
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
    let welded_count = (welded_vertices.iter().copied().max().unwrap_or(0)) + 1;
    let mut welded_vertex_normals_accumulator = vec![Vec3::ZERO; welded_count];
    for i in 0..vertex_count {
        let welded_vertex = welded_vertices[i];
        welded_vertex_normals_accumulator[welded_vertex] += vertex_normals[i];
    }
    let normalized_accumulation: Vec<Vec3> = welded_vertex_normals_accumulator
        .into_iter()
        .map(|normal| normal.normalize_or_zero())
        .collect();
    let mut smoothed_vertex_normals = vec![Vec3::ZERO; vertex_count];
    for i in 0..vertex_count {
        smoothed_vertex_normals[i] = normalized_accumulation[welded_vertices[i]];
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

//TODO: this is even crazier
pub fn reverse_vertex_winding(mesh: &mut WeakMesh) {
    if let Some(indices) = mesh.indices_mut() {
        for triangle in indices.chunks_exact_mut(3) {
            triangle.swap(1, 2);
        }
    } else {
        let mut indices = Vec::<u16>::with_capacity(mesh.vertexCount as usize);
        for t in 0..mesh.triangleCount {
            let base = (t * 3) as u16;
            indices.extend_from_slice(&[base, base + 2, base + 1]);
        }
        let mesh = mesh.as_mut();
        mesh.indices = Box::leak(indices.into_boxed_slice()).as_mut_ptr();
    }
}

pub fn debug_draw_triangles(
    observer: Camera3D,
    draw_handle: &mut RaylibDrawHandle,
    mesh: &WeakMesh,
    rotation: f32,
    triangles: &[usize],
    fill_color: Option<Color>,
    label: bool,
) {
    let vertices = mesh.vertices();
    // TODO: should this check or whatever even be in my project? otherwise in Mesh improvements in raylib-rs?
    if vertices.is_empty() {
        return;
    }
    let all_triangles: Vec<[usize; 3]> = mesh.triangles().collect();
    for &triangle_id in triangles {
        if triangle_id >= all_triangles.len() {
            continue;
        }
        let [vertex_a_index, vertex_b_index, vertex_c_index] = all_triangles[triangle_id];
        let mut triangle = vec![
            vertices[vertex_a_index],
            vertices[vertex_b_index],
            vertices[vertex_c_index],
        ];
        rotate_vertices(&mut triangle, rotation);
        let (vertex_a, vertex_b, vertex_c) = (triangle[0], triangle[1], triangle[2]);
        let color = if let Some(c) = fill_color {
            c
        } else {
            Color::new(
                (triangle_id.wrapping_mul(60) & 255) as u8,
                (triangle_id.wrapping_mul(120) & 255) as u8,
                (triangle_id.wrapping_mul(240) & 255) as u8,
                255,
            )
        };
        draw_handle.draw_mode3D(observer, |mut rl3d| {
            rl3d.draw_triangle3D(vertex_a, vertex_b, vertex_c, color);
        });

        if label {
            let screen_w = draw_handle.get_screen_width() as f32;
            let screen_h = draw_handle.get_screen_height() as f32;
            let centroid = (vertex_a + vertex_b + vertex_c) / 3.0;
            let sx = ((centroid.x) * 0.5 + 0.5) * screen_w;
            let sy = ((-centroid.y) * 0.5 + 0.5) * screen_h;
            draw_handle.draw_text(&triangle_id.to_string(), sx as i32, sy as i32, 12, Color::WHITE);
        }
    }
}
