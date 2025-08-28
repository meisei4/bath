use crate::fixed_func::silhouette::{lift_dimension, rotate_point_about_axis};
use crate::fixed_func::topology::{
    build_weld_view, collect_neighbors, collect_welded_faces, edge_opposing_vertex, face_normal, topology_init,
    welded_eq, WeldedEdge, WeldedMesh, WeldedVertex,
};
use raylib::math::glam::{Vec2, Vec3};
use raylib::models::{Mesh, WeakMesh};
use std::collections::{HashMap, VecDeque};
use std::f32::consts::PI;
use std::mem::{swap, zeroed};
use std::ptr::null_mut;

pub const ZOOM_SCALE: f32 = 2.0;

pub struct DisjointSetUnion {
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

#[derive(Copy, Clone, Debug)]
pub struct DualEdge {
    face_a: usize,
    face_b: usize,
    face_a_local_edge: (u8, u8),
    face_b_local_edge: (u8, u8),
    welded_edge: WeldedEdge,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct ParentLink {
    pub parent: Option<usize>, //TODO: why arent we using this?
    pub parent_local_edge: Option<(u8, u8)>,
    pub child_local_edge: Option<(u8, u8)>,
    pub welded_edge: Option<WeldedEdge>,
}

pub struct HingeAngleContext {
    pub parent_face: usize,
    pub child_face: usize,
    pub parent_local_edge: (u8, u8),
    pub welded_edge: WeldedEdge,
    pub original_signed_dihedral_angle: f32,
    pub breadth_first_depth_from_parent: u32,
    pub hinge_edge_length_in_unfolded_plane: f32,
    pub graph_distance_from_parent: f32,
}

const FOLD_DURATION_SEC: f32 = 5.0;
const FOLD_UNFOLD_DURATION: f32 = FOLD_DURATION_SEC * 2.0;

pub fn fold(mesh: &mut WeakMesh, i_time: f32, repeat_fold_unfold: bool) -> Mesh {
    let fold_progress = if repeat_fold_unfold {
        fold_unfold_time(i_time, FOLD_UNFOLD_DURATION)
    } else {
        (i_time / FOLD_DURATION_SEC).clamp(0.0, 1.0)
    };
    let (welded_mesh, parent_links, children, mut lifted_faces, parent_faces) = prepare_mesh_for_folding(mesh);
    for &parent_face in &parent_faces {
        apply_hinge_rotation_with_equation(
            parent_face,
            &children,
            &parent_links,
            &mut lifted_faces,
            &welded_mesh,
            i_time,
            &|context, _t| context.original_signed_dihedral_angle * fold_progress,
        );
        align_to_original_pose(&mut lifted_faces, &welded_mesh, parent_face);
    }
    build_unfolded_mesh(&lifted_faces, &welded_mesh)
}

pub fn unfold(mesh: &mut WeakMesh) -> Mesh {
    let (welded_mesh, parent_links, children, mut lifted_faces, parent_faces) = prepare_mesh_for_folding(mesh);
    for &parent_face in &parent_faces {
        apply_hinge_rotation_with_equation(
            parent_face,
            &children,
            &parent_links,
            &mut lifted_faces,
            &welded_mesh,
            0.0,
            &|_ctx, _t| 0.0,
        );
    }
    fit_unfolded_faces_to_zoom_scale(&mut lifted_faces);
    build_unfolded_mesh(&lifted_faces, &welded_mesh)
}

fn apply_hinge_rotation_with_equation(
    parent_face: usize,
    children_faces: &[Vec<usize>],
    parent_links: &[ParentLink],
    lifted_faces: &mut [[Vec3; 3]],
    welded_mesh: &WeldedMesh,
    i_time: f32,
    hinge_angle_equation: &dyn Fn(&HingeAngleContext, f32) -> f32,
) {
    let mut queue = VecDeque::new();
    queue.push_back((parent_face, 0u32, 0.0f32));

    while let Some((current_parent, depth_from_parent, dist_from_parent)) = queue.pop_front() {
        for &child_face in &children_faces[current_parent] {
            let link = parent_links[child_face];
            let parent_edge = link.parent_local_edge.unwrap();
            let parent_a = lifted_faces[current_parent][parent_edge.0 as usize];
            let parent_b = lifted_faces[current_parent][parent_edge.1 as usize];
            let hinge_len = (parent_b - parent_a).length();
            let breadth_first_depth_from_parent = depth_from_parent + 1;
            let graph_distance_from_parent = dist_from_parent + hinge_len;
            let original_signed_dihedral_angle = {
                let parent_id = current_parent;
                let child_id = child_face;
                signed_dihedral_between(parent_id, child_id, parent_edge, welded_mesh)
            };

            if hinge_len > 1e-8 {
                let context = HingeAngleContext {
                    parent_face: current_parent,
                    child_face,
                    parent_local_edge: parent_edge,
                    welded_edge: link.welded_edge.unwrap(),
                    original_signed_dihedral_angle,
                    breadth_first_depth_from_parent,
                    hinge_edge_length_in_unfolded_plane: hinge_len,
                    graph_distance_from_parent,
                };

                let angle = hinge_angle_equation(&context, i_time);
                let subtree = collect_subtree_faces(child_face, children_faces);
                for face in subtree {
                    for vertex_index in 0..3 {
                        lifted_faces[face][vertex_index] =
                            rotate_point_about_axis(lifted_faces[face][vertex_index], (parent_a, parent_b), angle);
                    }
                }
            }
            queue.push_back((child_face, breadth_first_depth_from_parent, graph_distance_from_parent));
        }
    }
}

fn prepare_mesh_for_folding(
    mesh: &mut WeakMesh,
) -> (WeldedMesh, Vec<ParentLink>, Vec<Vec<usize>>, Vec<[Vec3; 3]>, Vec<usize>) {
    let mut topology = topology_init(mesh);
    collect_welded_faces(&mut topology);
    collect_neighbors(&mut topology);
    let welded_mesh = build_weld_view(&mut topology, mesh);
    let face_count = welded_mesh.welded_faces.len();
    let mut dual_graph = build_dual_graph(&welded_mesh);
    let (parent_links, children_faces) = build_parent_tree(face_count, &mut dual_graph);
    let mut local_vertices_per_face = Vec::with_capacity(face_count);
    for face_index in 0..face_count {
        let [vertex_a, vertex_b, vertex_c] = welded_mesh.original_vertices[face_index];
        local_vertices_per_face.push(derive_local_plane_vertices(vertex_a, vertex_b, vertex_c));
    }

    let mut unfolded_faces = vec![[Vec2::ZERO; 3]; face_count];
    let mut is_already_unfolded = vec![false; face_count];
    for face in 0..face_count {
        if is_already_unfolded[face] {
            continue;
        }
        unfolded_faces[face] = anchor_welded_face(local_vertices_per_face[face], &welded_mesh.welded_faces[face]);
        is_already_unfolded[face] = true;
        let mut face_stack = vec![face];
        while let Some(parent_face) = face_stack.pop() {
            for &child_face in &children_faces[parent_face] {
                if is_already_unfolded[child_face] {
                    continue;
                }
                let parent_link = parent_links[child_face];
                let aligned_child = align_child_to_parent(
                    &local_vertices_per_face[child_face],
                    &welded_mesh.welded_faces[child_face],
                    &unfolded_faces[parent_face],
                    &welded_mesh.welded_faces[parent_face],
                    parent_link.parent_local_edge.unwrap(),
                    parent_link.child_local_edge.unwrap(),
                    parent_link.welded_edge.unwrap(),
                );
                unfolded_faces[child_face] = aligned_child;
                is_already_unfolded[child_face] = true;
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

    let parent_faces: Vec<usize> = (0..face_count).filter(|&f| parent_links[f].parent.is_none()).collect();

    (welded_mesh, parent_links, children_faces, lifted_faces, parent_faces)
}

fn fit_unfolded_faces_to_zoom_scale(unfolded_faces: &mut [[Vec3; 3]]) {
    for face in 0..unfolded_faces.len() {
        for vertex_index in 0..3 {
            unfolded_faces[face][vertex_index].z = 0.0;
        }
    }
    let mut min_x = f32::MAX;
    let mut min_y = f32::MAX;
    let mut max_x = f32::MIN;
    let mut max_y = f32::MIN;
    for face in 0..unfolded_faces.len() {
        for vertex_index in 0..3 {
            let vertex = unfolded_faces[face][vertex_index];
            min_x = min_x.min(vertex.x);
            max_x = max_x.max(vertex.x);
            min_y = min_y.min(vertex.y);
            max_y = max_y.max(vertex.y);
        }
    }
    let center_x = 0.5 * (min_x + max_x);
    let center_y = 0.5 * (min_y + max_y);
    let step_x = max_x - min_x;
    let step_y = max_y - min_y;
    let step = ZOOM_SCALE / step_x.max(step_y).max(1e-8);

    for face in 0..unfolded_faces.len() {
        for vertex_index in 0..3 {
            unfolded_faces[face][vertex_index].x = (unfolded_faces[face][vertex_index].x - center_x) * step;
            unfolded_faces[face][vertex_index].y = (unfolded_faces[face][vertex_index].y - center_y) * step;
        }
    }
}

fn build_unfolded_mesh(unfolded_faces: &[[Vec3; 3]], welded_mesh: &WeldedMesh) -> Mesh {
    let face_count = unfolded_faces.len();
    let mut vertices = Vec::with_capacity(face_count * 9);
    let mut texcoords = Vec::with_capacity(face_count * 6);
    let mut indices = Vec::with_capacity(face_count * 3);
    for face in 0..face_count {
        for vertex_index in 0..3 {
            let vertex = unfolded_faces[face][vertex_index];
            vertices.extend_from_slice(&[vertex.x, vertex.y, vertex.z]);
            let texcoord = welded_mesh.texcoords[face][vertex_index];
            texcoords.extend_from_slice(&[texcoord.x, texcoord.y]);
            indices.push((vertices.len() / 3 - 1) as u16);
        }
    }
    let mut unfolded_mesh: Mesh = unsafe { zeroed() };
    unfolded_mesh.vertexCount = (vertices.len() / 3) as i32;
    unfolded_mesh.triangleCount = (indices.len() / 3) as i32;
    unfolded_mesh.vertices = Box::leak(vertices.into_boxed_slice()).as_mut_ptr();
    unfolded_mesh.indices = Box::leak(indices.into_boxed_slice()).as_mut_ptr();
    unfolded_mesh.texcoords = Box::leak(texcoords.into_boxed_slice()).as_mut_ptr();
    unfolded_mesh.normals = null_mut();
    unfolded_mesh.tangents = null_mut();
    unfolded_mesh.colors = null_mut();
    unfolded_mesh
}

pub fn build_dual_graph(welded_mesh: &WeldedMesh) -> Vec<DualEdge> {
    let face_count = welded_mesh.welded_faces.len();
    let mut welded_edge_to_parent: HashMap<WeldedEdge, (usize, (u8, u8))> = HashMap::new();
    let mut dual_graph = Vec::new();
    for face in 0..face_count {
        let [welded_vertex_a, welded_vertex_b, welded_vertex_c] = welded_mesh.welded_faces[face];
        let local_edges = [(0u8, 1u8), (1, 2), (2, 0)];
        let welded_vertices = [welded_vertex_a, welded_vertex_b, welded_vertex_c];
        for &(point_a, point_b) in &local_edges {
            let edge = WeldedEdge::new(welded_vertices[point_a as usize], welded_vertices[point_b as usize]);
            if let Some(&(parent_face, parent_edge_local)) = welded_edge_to_parent.get(&edge) {
                dual_graph.push(DualEdge {
                    face_a: parent_face,
                    face_b: face,
                    welded_edge: edge,
                    face_a_local_edge: parent_edge_local,
                    face_b_local_edge: (point_a, point_b),
                });
            } else {
                welded_edge_to_parent.insert(edge, (face, (point_a, point_b)));
            }
        }
    }
    dual_graph
}

pub fn build_parent_tree(face_count: usize, dual_graph: &mut [DualEdge]) -> (Vec<ParentLink>, Vec<Vec<usize>>) {
    // dual_graph.sort_by(|left, right| right.fold_weight.partial_cmp(&left.fold_weight).unwrap());
    //TODO: biggest change for the anchored faces
    dual_graph.sort_by(|left, right| dual_edge_sorting_order(left).cmp(&dual_edge_sorting_order(right)));
    let mut dsu = DisjointSetUnion::new(face_count);
    let mut adjacency_list = vec![Vec::new(); face_count];

    for edge in dual_graph.iter().copied() {
        if dsu.union(edge.face_a, edge.face_b) {
            adjacency_list[edge.face_a].push((edge.face_b, edge));
            adjacency_list[edge.face_b].push((edge.face_a, edge));
        }
    }
    for adjacent_faces in &mut adjacency_list {
        adjacent_faces.sort_by_key(|&(face, edge)| (face, edge.welded_edge.vertex_a.id, edge.welded_edge.vertex_b.id));
    }
    let mut parent_links = vec![ParentLink::default(); face_count];
    let mut children = vec![Vec::new(); face_count];
    let mut seen = vec![false; face_count]; //TODO: stupid fucking parallel arrays again
    let mut face_queue = VecDeque::new();
    for id in 0..face_count {
        if seen[id] {
            continue;
        }
        seen[id] = true;
        face_queue.push_back(id);
        while let Some(current_face) = face_queue.pop_front() {
            for &(face, edge) in &adjacency_list[current_face] {
                if seen[face] {
                    continue;
                }
                seen[face] = true;
                let (parent_local_edge, child_local_edge) = if current_face == edge.face_a {
                    (edge.face_a_local_edge, edge.face_b_local_edge)
                } else {
                    (edge.face_b_local_edge, edge.face_a_local_edge)
                };
                parent_links[face] = ParentLink {
                    parent: Some(current_face),
                    parent_local_edge: Some(parent_local_edge),
                    child_local_edge: Some(child_local_edge),
                    welded_edge: Some(edge.welded_edge),
                };
                children[current_face].push(face);
                face_queue.push_back(face);
            }
        }
    }
    (parent_links, children)
}

pub fn align_to_welded_edge(
    parent_face_welded_vertices: &[WeldedVertex; 3],
    local_edge: (u8, u8),
    welded_edge: WeldedEdge,
) -> (u8, u8) {
    let local_vertex_a = local_edge.0 as usize;
    let local_vertex_b = local_edge.1 as usize;
    let welded_vertex_a = parent_face_welded_vertices[local_vertex_a];
    let welded_vertex_b = parent_face_welded_vertices[local_vertex_b];
    if welded_eq(welded_vertex_a, welded_edge.vertex_a) && welded_eq(welded_vertex_b, welded_edge.vertex_b) {
        (local_edge.0, local_edge.1)
    } else if welded_eq(welded_vertex_a, welded_edge.vertex_b) && welded_eq(welded_vertex_b, welded_edge.vertex_a) {
        (local_edge.1, local_edge.0)
    } else {
        panic!("local_edge does not match welded_edge (bad adjacency / welding).");
    }
}

pub fn align_child_to_parent(
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
    let parent_x_axis = parent_ab.normalize();
    // let parent_y_axis = Vec2::new(-parent_x_axis.y, parent_x_axis.x);
    let perpendicular_rotation = Vec2::new((PI * 0.5).cos(), 1.0);
    let parent_y_axis = parent_x_axis.rotate(perpendicular_rotation); //rotate PI/2

    let child_a = child_local_vertices[aligned_child_edge.0 as usize];
    let child_b = child_local_vertices[aligned_child_edge.1 as usize];
    let child_x_axis = (child_b - child_a).normalize();
    let cosine_rotation = child_x_axis.dot(parent_x_axis).clamp(-1.0, 1.0);
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

pub fn derive_local_plane_vertices(a: Vec3, b: Vec3, c: Vec3) -> [Vec2; 3] {
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

pub fn dual_edge_sorting_order(edge: &DualEdge) -> (u32, u32, usize, usize) {
    let welded_edge_vertex_a = edge.welded_edge.vertex_a.id;
    let welded_edge_vertex_b = edge.welded_edge.vertex_b.id;
    let lesser_face = edge.face_a.min(edge.face_b);
    let greater_face = edge.face_a.max(edge.face_b);
    (welded_edge_vertex_a, welded_edge_vertex_b, lesser_face, greater_face)
}

pub fn anchor_welded_face(face: [Vec2; 3], welded_face: &[WeldedVertex; 3]) -> [Vec2; 3] {
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

pub fn collect_subtree_faces(root_face: usize, children: &[Vec<usize>]) -> Vec<usize> {
    let mut subtree = Vec::new();
    let mut face_stack = vec![root_face];
    while let Some(face) = face_stack.pop() {
        subtree.push(face);
        for &child_face in &children[face] {
            face_stack.push(child_face);
        }
    }
    subtree
}

pub fn signed_dihedral_between(
    parent_id: usize,
    child_id: usize,
    parent_edge_local: (u8, u8),
    welded: &WeldedMesh,
) -> f32 {
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

pub fn align_to_original_pose(unfolded_faces_lifted: &mut [[Vec3; 3]], welded_mesh: &WeldedMesh, root: usize) {
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
pub fn fold_unfold_time(i_time: f32, period: f32) -> f32 {
    let u = (i_time / period).fract();
    if u <= 0.5 {
        u * 2.0
    } else {
        2.0 - 2.0 * u
    }
}

pub struct BillowWaveParameters {
    /// peak hinge rotation in radians (e.g., 0.2 ~ 11.5°)
    pub amplitude_radians: f32,
    /// wavelength measured along the tree distance in the unfolded plane (same units as your coordinates)
    pub wavelength_in_unfolded_space: f32,
    /// wave cycles per second (1.0 = one full cycle every second)
    pub speed_cycles_per_second: f32,
    /// constant phase offset in radians
    pub phase_offset_radians: f32,
    /// optional attenuation per breadth-first layer away from the root (0.0 = no attenuation)
    pub depth_attenuation_per_breadth_first_step: f32,
    /// optional multiplier applied to emphasize long hinges (0.0 = ignore hinge length, 1.0 = linear)
    pub hinge_length_weight: f32,
}

impl Default for BillowWaveParameters {
    fn default() -> Self {
        Self {
            amplitude_radians: 0.2,
            wavelength_in_unfolded_space: 1.5,
            speed_cycles_per_second: 0.6,
            phase_offset_radians: 0.0,
            depth_attenuation_per_breadth_first_step: 0.0,
            hinge_length_weight: 0.0,
        }
    }
}

fn fit_unfolded_faces_xy_to_zoom_scale_preserve_z(unfolded_faces: &mut [[Vec3; 3]]) {
    let mut min_x = f32::MAX;
    let mut min_y = f32::MAX;
    let mut max_x = f32::MIN;
    let mut max_y = f32::MIN;
    for f in 0..unfolded_faces.len() {
        for i in 0..3 {
            let v = unfolded_faces[f][i];
            min_x = min_x.min(v.x);
            max_x = max_x.max(v.x);
            min_y = min_y.min(v.y);
            max_y = max_y.max(v.y);
        }
    }

    let center_x = 0.5 * (min_x + max_x);
    let center_y = 0.5 * (min_y + max_y);
    let step_x = max_x - min_x;
    let step_y = max_y - min_y;
    let step = ZOOM_SCALE / step_x.max(step_y).max(1e-8);
    for f in 0..unfolded_faces.len() {
        for i in 0..3 {
            unfolded_faces[f][i].x = (unfolded_faces[f][i].x - center_x) * step;
            unfolded_faces[f][i].y = (unfolded_faces[f][i].y - center_y) * step;
        }
    }
}

pub fn billow_unfolded(
    mesh: &mut WeakMesh,
    i_time: f32,
    fit_xy_before_animation: bool,
    params: &BillowWaveParameters,
) -> Mesh {
    let (welded_mesh, parent_links, children, mut unfolded_faces, root_faces) = prepare_mesh_for_folding(mesh);
    if fit_xy_before_animation {
        fit_unfolded_faces_to_zoom_scale(&mut unfolded_faces);
    }
    for &root in &root_faces {
        apply_hinge_rotation_with_equation(
            root,
            &children,
            &parent_links,
            &mut unfolded_faces,
            &welded_mesh,
            i_time,
            &|context, t| {
                let two_pi = std::f32::consts::TAU; // 2π
                let k = two_pi / params.wavelength_in_unfolded_space;
                let omega = two_pi * params.speed_cycles_per_second;
                let base_phase = k * context.graph_distance_from_parent - omega * t + params.phase_offset_radians;
                let depth_atten = (-params.depth_attenuation_per_breadth_first_step
                    * context.breadth_first_depth_from_parent as f32)
                    .exp();

                let length_weight = if params.hinge_length_weight > 0.0 {
                    let l = context.hinge_edge_length_in_unfolded_plane.max(1e-6);
                    (1.0 + params.hinge_length_weight * l).sqrt()
                } else {
                    1.0
                };

                params.amplitude_radians * base_phase.sin() * depth_atten * length_weight
            },
        );
    }

    build_unfolded_mesh(&unfolded_faces, &welded_mesh)
}

pub struct WhipPulseParameters {
    /// peak hinge rotation in radians at the pulse crest (e.g., 0.25 ~ 14.3°)
    pub maximum_rotation_radians: f32,
    /// how fast the pulse front advances measured in unfolded-space distance units per second
    pub pulse_front_speed_units_per_second: f32,
    /// width of the pulse (the spatial sigma of the gaussian envelope), in unfolded-space units
    pub pulse_width_sigma_in_unfolded_space: f32,
    /// exponential attenuation per breadth-first layer (0.0 = none)
    pub depth_attenuation_per_breadth_first_step: f32,
    /// optional emphasis for long hinges (0 = ignore, 1 = linear-ish boost)
    pub hinge_length_weight: f32,
    /// optional launch delay in seconds (can be 0.0)
    pub launch_delay_seconds: f32,
}

impl Default for WhipPulseParameters {
    fn default() -> Self {
        Self {
            maximum_rotation_radians: 0.25,
            pulse_front_speed_units_per_second: 1.2,
            pulse_width_sigma_in_unfolded_space: 0.35,
            depth_attenuation_per_breadth_first_step: 0.0,
            hinge_length_weight: 0.0,
            launch_delay_seconds: 0.0,
        }
    }
}

#[inline]
fn gaussian_whip_envelope(distance_along_tree: f32, time_seconds: f32, params: &WhipPulseParameters) -> f32 {
    let elapsed = (time_seconds - params.launch_delay_seconds).max(0.0);
    let front_pos = elapsed * params.pulse_front_speed_units_per_second;
    let sigma = params.pulse_width_sigma_in_unfolded_space.max(1e-6);
    let x = (distance_along_tree - front_pos) / sigma; // 0 near the front
    (-x * x).exp() // gaussian peak at the front, decays behind/ahead
}

pub fn whip_pulse_unfolded(
    mesh: &mut WeakMesh,
    i_time: f32,
    fit_xy_before_animation: bool,
    params: &WhipPulseParameters,
) -> Mesh {
    let (welded_mesh, parent_links, children, mut unfolded_faces, root_faces) = prepare_mesh_for_folding(mesh);
    if fit_xy_before_animation {
        fit_unfolded_faces_xy_to_zoom_scale_preserve_z(&mut unfolded_faces);
    }

    for &root in &root_faces {
        apply_hinge_rotation_with_equation(
            root,
            &children,
            &parent_links,
            &mut unfolded_faces,
            &welded_mesh,
            i_time,
            &|context, t| {
                let mut angle = gaussian_whip_envelope(context.graph_distance_from_parent, t, params)
                    * params.maximum_rotation_radians;
                if params.depth_attenuation_per_breadth_first_step > 0.0 {
                    angle *= (-params.depth_attenuation_per_breadth_first_step
                        * context.breadth_first_depth_from_parent as f32)
                        .exp();
                }
                if params.hinge_length_weight > 0.0 {
                    let l = context.hinge_edge_length_in_unfolded_plane.max(1e-6);
                    angle *= (1.0 + params.hinge_length_weight * l).sqrt();
                }
                angle
            },
        );
    }
    build_unfolded_mesh(&unfolded_faces, &welded_mesh)
}

pub struct PeriodicWhipParameters {
    /// peak hinge rotation in radians (e.g., 0.35 ~ 20°)
    pub amplitude_radians: f32,
    /// wavelength along unfolded-space graph distance (same units as your XY)
    pub wavelength_in_unfolded_space: f32,
    /// cycles per second (temporal), higher = faster repetition
    pub speed_cycles_per_second: f32,
    /// constant phase offset (radians)
    pub phase_offset_radians: f32,

    /// shapes the pulse front/back. 1.0 = sine; >1.0 = “snappier” whip
    pub front_sharpness_exponent: f32,
    /// if true: half-wave rectification (only scrunch → relax → scrunch ...),
    /// if false: symmetric back-and-forth (fold one way then the other).
    pub use_half_wave_rectification: bool,

    /// optional attenuation per BFS layer from the root (0 = none)
    pub depth_attenuation_per_breadth_first_step: f32,
    /// optional hinge-length emphasis (0 = none, 1 ~ linear-ish boost)
    pub hinge_length_weight: f32,
    /// delay before the first crest starts moving
    pub launch_delay_seconds: f32,
}

impl Default for PeriodicWhipParameters {
    fn default() -> Self {
        Self {
            amplitude_radians: 0.35,
            wavelength_in_unfolded_space: 1.2,
            speed_cycles_per_second: 0.8,
            phase_offset_radians: 0.0,
            front_sharpness_exponent: 1.75,
            use_half_wave_rectification: true,
            depth_attenuation_per_breadth_first_step: 0.0,
            hinge_length_weight: 0.0,
            launch_delay_seconds: 0.0,
        }
    }
}

#[inline]
fn shaped_periodic_whip_profile(phase: f32, exponent: f32, half_wave: bool) -> f32 {
    // base sinusoid
    let s = phase.sin();
    // optional half-wave rectification: scrunch (positive) then relax to zero (no negative unfold)
    let h = if half_wave { s.max(0.0) } else { s };
    // power shaping for snappier front and quicker release
    let e = exponent.max(1.0);
    h.signum() * h.abs().powf(e)
}

pub fn periodic_whip_unfolded(
    mesh: &mut WeakMesh,
    i_time: f32,
    fit_xy_before_animation: bool,
    params: &PeriodicWhipParameters,
) -> Mesh {
    // build unfolded model once (your pipeline)
    let (welded_mesh, parent_links, children, mut unfolded_faces, root_faces) = prepare_mesh_for_folding(mesh);

    if fit_xy_before_animation {
        fit_unfolded_faces_xy_to_zoom_scale_preserve_z(&mut unfolded_faces);
    }

    let two_pi = std::f32::consts::TAU;
    let k = two_pi / params.wavelength_in_unfolded_space.max(1e-6);
    let omega = two_pi * params.speed_cycles_per_second;

    for &root in &root_faces {
        apply_hinge_rotation_with_equation(
            root,
            &children,
            &parent_links,
            &mut unfolded_faces,
            &welded_mesh,
            i_time,
            &|context, t| {
                // traveling wave phase along the tree distance (unfolded-space)
                let elapsed = (t - params.launch_delay_seconds).max(0.0);
                let phase = k * context.graph_distance_from_parent - omega * elapsed + params.phase_offset_radians;

                // shape it: sine → rectified (optional) → power-shaped “whip”
                let mut profile = shaped_periodic_whip_profile(
                    phase,
                    params.front_sharpness_exponent,
                    params.use_half_wave_rectification,
                );

                // attenuation with BFS depth
                if params.depth_attenuation_per_breadth_first_step > 0.0 {
                    profile *= (-params.depth_attenuation_per_breadth_first_step
                        * context.breadth_first_depth_from_parent as f32)
                        .exp();
                }

                // hinge length emphasis (longer hinges = bigger kick)
                if params.hinge_length_weight > 0.0 {
                    let l = context.hinge_edge_length_in_unfolded_plane.max(1e-6);
                    profile *= (1.0 + params.hinge_length_weight * l).sqrt();
                }

                params.amplitude_radians * profile
            },
        );
    }

    build_unfolded_mesh(&unfolded_faces, &welded_mesh)
}
