use crate::fixed_func::immediate_mode3d::{with_immediate_mode3d, Viewport};
use raylib::camera::Camera3D;
use raylib::color::Color;
use raylib::drawing::{RaylibDraw, RaylibDraw3D, RaylibDrawHandle, RaylibMode3DExt};
use raylib::math::glam::{Vec2, Vec3};
use raylib::math::{Vector2, Vector3};
use raylib::models::{RaylibMesh, WeakMesh};
use std::collections::{HashMap, HashSet};

pub struct WeldedMesh {
    pub vertices_per_triangle: Vec<[Vec3; 3]>,
    pub welded_vertices_per_triangle: Vec<[WeldedVertex; 3]>,
    pub texcoords_per_triangle: Vec<[Vec2; 3]>,
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

pub struct Topology {
    pub triangles_snapshot: Vec<[usize; 3]>,
    pub vertices_per_triangle_snapshot: Option<Vec<[Vec3; 3]>>,
    pub corner_angles_per_triangle_snapshot: Option<Vec<[f32; 3]>>,
    pub vertex_normals_snapshot: Option<Vec<Vec3>>,
    pub texcoords_per_triangle_snapshot: Option<Vec<[Vec2; 3]>>,
    pub welded_vertices_snapshot: Option<Vec<usize>>,
    pub welded_vertices_per_triangle_snapshot: Option<Vec<[WeldedVertex; 3]>>,
    pub neighbors_per_triangle_snapshot: Option<Vec<[Option<usize>; 3]>>,
    pub front_triangles_snapshot: Option<HashSet<usize>>,
    pub back_triangles_snapshot: Option<HashSet<usize>>,
    pub silhouette_triangles_snapshot: Option<HashSet<usize>>,
}
pub struct TopologyBuilder<'a> {
    topology: Topology,
    mesh: &'a WeakMesh,
}

impl Topology {
    #[inline]
    pub fn build_topology(mesh: &WeakMesh) -> TopologyBuilder<'_> {
        TopologyBuilder {
            topology: Topology {
                triangles_snapshot: Vec::new(), //TODO: is this neccessary? or can we just always use the Meshes Triangles?
                vertices_per_triangle_snapshot: None, //TODO: is this neccessary? arent these literally just the vertices of the mesh?
                corner_angles_per_triangle_snapshot: None,
                vertex_normals_snapshot: None, //TODO: is this neccessary? or do we have normals from the Mesh?
                texcoords_per_triangle_snapshot: None, //TODO: is this neccessary? or can we just always use the Meshes texcoords?
                welded_vertices_snapshot: None,
                welded_vertices_per_triangle_snapshot: None,
                neighbors_per_triangle_snapshot: None,
                front_triangles_snapshot: None,
                back_triangles_snapshot: None,
                silhouette_triangles_snapshot: None,
            },
            mesh,
        }
    }
}

impl<'a> TopologyBuilder<'a> {
    #[inline]
    pub fn triangles(mut self) -> Self {
        self.topology.triangles_snapshot = self.mesh.triangles().collect();
        self
    }

    #[inline]
    pub fn vertices_per_triangle(mut self) -> Self {
        if self.topology.vertices_per_triangle_snapshot.is_some() {
            return self;
        }
        let mut vertices_per_triangle_snapshot = Vec::with_capacity(self.mesh.triangle_count());
        let vertices = self.mesh.vertices();
        for [vertex_a_index, vertex_b_index, vertex_c_index] in self.mesh.triangles() {
            let triangle_abc_positions = [
                vertices[vertex_a_index],
                vertices[vertex_b_index],
                vertices[vertex_c_index],
            ];
            vertices_per_triangle_snapshot.push(triangle_abc_positions);
        }
        self.topology.vertices_per_triangle_snapshot = Some(vertices_per_triangle_snapshot);
        self
    }

    #[inline]
    pub fn vertex_normals(mut self) -> Self {
        let vertices = self.mesh.vertices();
        let vertex_count = vertices.len();
        let mut vertex_normals_accumulator = vec![Vec3::ZERO; vertex_count];
        let mut corner_angles: Vec<[f32; 3]> = Vec::with_capacity(self.mesh.triangle_count());
        for [vertex_a_index, vertex_b_index, vertex_c_index] in self.mesh.triangles() {
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
            vertex_normals_accumulator[vertex_a_index] += triangle_normal * angle_a;
            vertex_normals_accumulator[vertex_b_index] += triangle_normal * angle_b;
            vertex_normals_accumulator[vertex_c_index] += triangle_normal * angle_c;
        }

        self.topology.corner_angles_per_triangle_snapshot = Some(corner_angles);
        self.topology.vertex_normals_snapshot = Some(
            vertex_normals_accumulator
                .into_iter()
                .map(|n| n.normalize_or_zero())
                .collect(),
        );
        self
    }

    #[inline]
    pub fn welded_vertices(mut self) -> Self {
        let vertices = self.mesh.vertices();
        let vertex_count = vertices.len();
        let mut welded_vertices_to_triangles_map: HashMap<(i32, i32, i32), usize> = HashMap::new();
        let mut next_wid: usize = 0;
        let mut welded_vertices = vec![0usize; vertex_count];
        for (i, v) in vertices.iter().enumerate() {
            let key = (quantize(v.x), quantize(v.y), quantize(v.z));
            let wid = *welded_vertices_to_triangles_map.entry(key).or_insert_with(|| {
                let id = next_wid;
                next_wid += 1;
                id
            });
            welded_vertices[i] = wid;
        }
        self.topology.welded_vertices_snapshot = Some(welded_vertices);
        self
    }

    pub fn smooth_vertex_normals(mut self) -> Self {
        //TODO: how do i make these panic messages actually reference the function name reflectively, not just a string literal
        let vertex_normals = self
            .topology
            .vertex_normals_snapshot
            .as_ref()
            .expect("smooth_vertex_normals: call vertex_normals() first");
        let welded_vertices = self
            .topology
            .welded_vertices_snapshot
            .as_ref()
            .expect("smooth_vertex_normals: call welded_vertices() first");
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
        let mut smooth_vertex_normals = vec![Vec3::ZERO; vertex_count];
        for i in 0..vertex_count {
            smooth_vertex_normals[i] = normalized_accumulation[welded_vertices[i]];
        }
        self.topology.vertex_normals_snapshot = Some(smooth_vertex_normals);
        self
    }

    #[inline]
    pub fn texcoords_per_triangle(mut self) -> Self {
        if self.topology.texcoords_per_triangle_snapshot.is_some() {
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
        self.topology.texcoords_per_triangle_snapshot = Some(triangle_texcoords);
        self
    }

    #[inline]
    pub fn welded_vertices_per_triangle(mut self) -> Self {
        if self.topology.welded_vertices_per_triangle_snapshot.is_some() {
            return self;
        }
        let welded_ids = self
            .topology
            .welded_vertices_snapshot
            .as_ref()
            .expect("welded_vertices_per_triangle: call welded_vertices() first");
        let mut welded_vertices_per_triangle = Vec::with_capacity(self.topology.triangles_snapshot.len());
        for [vertex_a_index, vertex_b_index, vertex_c_index] in self.topology.triangles_snapshot.iter().copied() {
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
        self.topology.welded_vertices_per_triangle_snapshot = Some(welded_vertices_per_triangle);
        self
    }

    #[inline]
    pub fn neighbors_per_triangle(mut self) -> Self {
        if self.topology.neighbors_per_triangle_snapshot.is_some() {
            return self;
        }
        if self.topology.welded_vertices_per_triangle_snapshot.is_none() {
            self = self.welded_vertices_per_triangle();
        }
        let welded_triangles = self
            .topology
            .welded_vertices_per_triangle_snapshot
            .as_ref()
            .expect("neighbors: welded_vertices_per_triangle not present. Call welded_triangles first.");
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
        self.topology.neighbors_per_triangle_snapshot = Some(neighbors);
        self
    }

    #[inline]
    pub fn build_welded_mesh(self) -> WeldedMesh {
        let vertices_per_triangle = self
            .topology
            .vertices_per_triangle_snapshot
            .as_ref()
            .expect("build_welded_mesh: call vertices_per_triangle() first")
            .clone();
        let welded_vertices_per_triangle = self
            .topology
            .welded_vertices_per_triangle_snapshot
            .as_ref()
            .expect("build_welded_mesh: call welded_vertices_per_triangle() first")
            .clone();
        let texcoords_per_triangle = self
            .topology
            .texcoords_per_triangle_snapshot
            .as_ref()
            .expect("build_welded_mesh: call texcoords_per_triangle() first")
            .clone();
        WeldedMesh {
            vertices_per_triangle,
            welded_vertices_per_triangle,
            texcoords_per_triangle,
        }
    }

    #[inline]
    pub fn front_triangles(mut self, rotation: f32, observer: &Camera3D) -> Self {
        assert!(
            !self.topology.triangles_snapshot.is_empty(),
            "front_triangles: call triangles() first"
        );
        let line_of_sight = observed_line_of_sight(observer);
        let vertices_per_triangle = self
            .topology
            .vertices_per_triangle_snapshot
            .as_ref()
            .expect("front_triangles: call vertices_per_triangle() first");
        let mut front_triangles = HashSet::with_capacity(self.topology.triangles_snapshot.len());
        for (triangle_id, [vertex_a, vertex_b, vertex_c]) in vertices_per_triangle.iter().copied().enumerate() {
            let mut triangle = vec![vertex_a, vertex_b, vertex_c];
            rotate_vertices_in_plane_slice(&mut triangle, rotation);
            let normal = triangle_normal(triangle[0], triangle[1], triangle[2]);
            if normal.dot(line_of_sight) <= 0.0 {
                front_triangles.insert(triangle_id);
            }
        }
        self.topology.front_triangles_snapshot = Some(front_triangles);
        // return self; //TODO first time "return" keyword has ever been truly needed for me?
        self //TODO: nevermind, this was a good example but i am doing panics now
    }

    #[inline]
    pub fn back_triangles(mut self) -> Self {
        let front_triangles = self
            .topology
            .front_triangles_snapshot
            .as_ref()
            .expect("back_triangles: front_triangles not present. Call front_triangles first.");
        let triangle_count = self.topology.triangles_snapshot.len();
        let mut back_triangles = HashSet::with_capacity(triangle_count.saturating_sub(front_triangles.len()));
        for triangle_id in 0..triangle_count {
            if !front_triangles.contains(&triangle_id) {
                back_triangles.insert(triangle_id);
            }
        }
        self.topology.back_triangles_snapshot = Some(back_triangles);
        self
    }
    #[inline]
    pub fn silhouette_triangles(mut self) -> Self {
        let neighbors_per_triangle = self
            .topology
            .neighbors_per_triangle_snapshot
            .as_ref()
            .expect("silhouette_triangles: neighbors_per_triangle not present. Call neighbors first.");
        let front_triangles = self
            .topology
            .front_triangles_snapshot
            .as_ref()
            .expect("silhouette_triangles: front_triangles not present. Call front_triangles first.");
        let welded_vertices_per_triangle = self
            .topology
            .welded_vertices_per_triangle_snapshot
            .as_ref()
            .expect("silhouette_triangles: welded_vertices_per_triangle not present. Call welded_triangles first.");
        let mut silhouette_triangles: HashSet<usize> = HashSet::new();
        for (triangle_id, triangle) in welded_vertices_per_triangle.iter().copied().enumerate() {
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
                let silhouette_edge_key = WeldedEdge::new(triangle[vertex_0], triangle[vertex_1]);
                let rim_local_edge_index = {
                    let candidates = [
                        (WeldedEdge::new(vertices[0], vertices[1]), 0),
                        (WeldedEdge::new(vertices[1], vertices[2]), 1),
                        (WeldedEdge::new(vertices[2], vertices[0]), 2),
                    ];
                    let mut found = 0;
                    for (edge, idx) in candidates {
                        if edge == silhouette_edge_key {
                            found = idx;
                            break;
                        }
                    }
                    found
                };
                let interior_edge_index_0 = ((rim_local_edge_index + 1) % 3) as usize;
                let interior_edge_index_1 = ((rim_local_edge_index + 2) % 3) as usize;
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
        self.topology.silhouette_triangles_snapshot = Some(silhouette_triangles);
        self
    }

    #[inline]
    pub fn build(self) -> Topology {
        self.topology
    }
}

pub fn debug_draw_triangles(
    observer: Camera3D,
    draw_handle: &mut RaylibDrawHandle,
    topology: &Topology,
    rotation: f32,
    triangle_set: &HashSet<usize>,
    fill_color: Option<Color>,
    label: bool,
    font_size: i32,
) {
    let vertices_per_triangle_snapshot = topology
        .vertices_per_triangle_snapshot
        .as_ref()
        .expect("debug_draw_triangles_snapshot: call TopologyBuilder::vertices_per_triangle() first");

    let triangle_count = vertices_per_triangle_snapshot.len();
    for &triangle_id in triangle_set {
        if triangle_id >= triangle_count {
            continue;
        }
        let [vertex_a, vertex_b, vertex_c] = vertices_per_triangle_snapshot[triangle_id];
        let mut triangle = vec![vertex_a, vertex_b, vertex_c];
        rotate_vertices_in_plane_slice(&mut triangle, rotation);
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
        //TODO: in OpenGL 1.1 context the labels/text will not draw correctly when the viewport/screen is non-square aspect
        //TODO: something about screen scale and the actual mesh showing up (model scale?)
        if label {
            let screen_w = draw_handle.get_screen_width() as f32;
            let screen_h = draw_handle.get_screen_height() as f32;
            let centroid = (vertex_a + vertex_b + vertex_c) / 3.0;
            let sx = ((centroid.x * 0.5 + 0.5) * screen_w) as i32;
            let sy = ((-centroid.y * 0.5 + 0.5) * screen_h) as i32;
            draw_handle.draw_text(&triangle_id.to_string(), sx, sy, font_size, Color::WHITE);
        }
    }
}

pub unsafe fn debug_draw_triangles_immediate(
    observer: &Camera3D,
    viewport: Viewport,
    draw_handle: &mut RaylibDrawHandle,
    topology: &Topology,
    rotation: f32,
    triangle_set: &HashSet<usize>,
    fill_color: Option<Color>,
    label: bool,
    font_size: i32,
) {
    let vertices_per_triangle_snapshot = topology.vertices_per_triangle_snapshot.as_ref().unwrap();
    let triangle_count = vertices_per_triangle_snapshot.len();

    with_immediate_mode3d(observer, viewport, 0.01, 1000.0, |rl3d| {
        for &triangle_id in triangle_set {
            if triangle_id >= triangle_count {
                continue;
            }
            let [vertex_a, vertex_b, vertex_c] = vertices_per_triangle_snapshot[triangle_id];
            let mut triangle = vec![vertex_a, vertex_b, vertex_c];
            rotate_vertices_in_plane_slice(&mut triangle, rotation);
            let (a, b, c) = (triangle[0], triangle[1], triangle[2]);

            let color = fill_color.unwrap_or_else(|| {
                Color::new(
                    (triangle_id.wrapping_mul(60) & 255) as u8,
                    (triangle_id.wrapping_mul(120) & 255) as u8,
                    (triangle_id.wrapping_mul(240) & 255) as u8,
                    255,
                )
            });

            rl3d.draw_triangle3d(a, b, c, color);
        }
    });

    if label {
        for &triangle_id in triangle_set {
            if triangle_id >= triangle_count {
                continue;
            }
            let [vertex_a, vertex_b, vertex_c] = vertices_per_triangle_snapshot[triangle_id];
            let mut triangle = vec![vertex_a, vertex_b, vertex_c];
            rotate_vertices_in_plane_slice(&mut triangle, rotation);
            let (a, b, c) = (triangle[0], triangle[1], triangle[2]);
            let centroid = (a + b + c) / 3.0;
            let sx = viewport.x + ((centroid.x * 0.5 + 0.5) * viewport.w as f32) as i32;
            let sy = viewport.y + ((-centroid.y * 0.5 + 0.5) * viewport.h as f32) as i32;

            draw_handle.draw_text(&triangle_id.to_string(), sx, sy, font_size, Color::WHITE);
        }
    }
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
pub fn rotate_vertices_in_plane_slice(vertices: &mut [Vector3], rotation: f32) {
    for vertex in vertices {
        let (x0, z0) = (vertex.x, vertex.z);
        vertex.x = x0 * rotation.cos() + z0 * rotation.sin();
        vertex.z = -x0 * rotation.sin() + z0 * rotation.cos();
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
fn angle_between(a: Vec3, b: Vec3) -> f32 {
    let a_magnitude = a.length();
    let b_magnitude = b.length();
    if a_magnitude <= 0.0 || b_magnitude <= 0.0 {
        return 0.0;
    }
    (a.dot(b) / (a_magnitude * b_magnitude)).clamp(-1.0, 1.0).acos()
}
