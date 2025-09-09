use crate::fixed_func::dsu::{build_dual_graph, build_parent_tree, collect_subtree_triangles, ParentLink};
use crate::fixed_func::topology::{
    edge_opposing_vertex, lift_dimension, rotate_point_about_axis, triangle_normal, welded_eq, Topology, WeldedEdge,
    WeldedMesh, WeldedVertex,
};
use raylib::math::glam::{Vec2, Vec3};
use raylib::math::{Vector2, Vector3};
use raylib::models::{Mesh, WeakMesh};
use std::collections::VecDeque;
use std::f32::consts::PI;

pub const ZOOM_SCALE: f32 = 2.0;
const FOLD_DURATION_SEC: f32 = 5.0;
const FOLD_UNFOLD_DURATION: f32 = FOLD_DURATION_SEC * 2.0;

pub struct HingeAngleContext {
    pub parent_triangle: usize,
    pub child_triangle: usize,
    pub parent_local_edge: (u8, u8),
    pub welded_edge: WeldedEdge,
    pub original_signed_dihedral_angle: f32,
    pub breadth_first_depth_from_parent: u32,
    pub hinge_edge_length_in_unfolded_plane: f32,
    pub graph_distance_from_parent: f32,
}

pub fn fold(_thread_borrow: &raylib::RaylibThread, mesh: &mut WeakMesh, i_time: f32, repeat_fold_unfold: bool) -> Mesh {
    let fold_progress = if repeat_fold_unfold {
        fold_unfold_time(i_time, FOLD_UNFOLD_DURATION)
    } else {
        (i_time / FOLD_DURATION_SEC).clamp(0.0, 1.0)
    };
    let (welded_mesh, parent_links, children, mut lifted_triangles, parent_triangles) = prepare_mesh_for_folding(mesh);
    for &parent_triangle in &parent_triangles {
        apply_hinge_rotation_with_equation(
            parent_triangle,
            &children,
            &parent_links,
            &mut lifted_triangles,
            &welded_mesh,
            i_time,
            &|context, _t| context.original_signed_dihedral_angle * fold_progress,
        );
        align_to_original_pose(&mut lifted_triangles, &welded_mesh, parent_triangle);
    }
    build_unfolded_mesh(_thread_borrow, &lifted_triangles, &welded_mesh)
}

pub fn unfold(_thread_borrow: &raylib::RaylibThread, mesh: &mut WeakMesh) -> Mesh {
    let (welded_mesh, parent_links, children, mut lifted_triangles, parent_triangles) = prepare_mesh_for_folding(mesh);
    for &parent_triangle in &parent_triangles {
        apply_hinge_rotation_with_equation(
            parent_triangle,
            &children,
            &parent_links,
            &mut lifted_triangles,
            &welded_mesh,
            0.0,
            &|_ctx, _t| 0.0,
        );
    }
    fit_unfolded_triangles_to_zoom_scale(&mut lifted_triangles);
    build_unfolded_mesh(_thread_borrow, &lifted_triangles, &welded_mesh)
}

fn apply_hinge_rotation_with_equation(
    parent_triangle: usize,
    children_triangles: &[Vec<usize>],
    parent_links: &[ParentLink],
    lifted_triangles: &mut [[Vec3; 3]],
    welded_mesh: &WeldedMesh,
    i_time: f32,
    hinge_angle_equation: &dyn Fn(&HingeAngleContext, f32) -> f32,
) {
    let mut queue = VecDeque::new();
    queue.push_back((parent_triangle, 0u32, 0.0f32));

    while let Some((current_parent, depth_from_parent, dist_from_parent)) = queue.pop_front() {
        for &child_triangle in &children_triangles[current_parent] {
            let link = parent_links[child_triangle];
            let parent_edge = link.parent_local_edge.unwrap();
            let parent_a = lifted_triangles[current_parent][parent_edge.0 as usize];
            let parent_b = lifted_triangles[current_parent][parent_edge.1 as usize];
            let hinge_len = (parent_b - parent_a).length();
            let breadth_first_depth_from_parent = depth_from_parent + 1;
            let graph_distance_from_parent = dist_from_parent + hinge_len;
            let original_signed_dihedral_angle = {
                let parent_id = current_parent;
                let child_id = child_triangle;
                signed_dihedral_between(parent_id, child_id, parent_edge, welded_mesh)
            };

            if hinge_len > 1e-8 {
                let context = HingeAngleContext {
                    parent_triangle: current_parent,
                    child_triangle,
                    parent_local_edge: parent_edge,
                    welded_edge: link.welded_edge.unwrap(),
                    original_signed_dihedral_angle,
                    breadth_first_depth_from_parent,
                    hinge_edge_length_in_unfolded_plane: hinge_len,
                    graph_distance_from_parent,
                };

                let angle = hinge_angle_equation(&context, i_time);
                let subtree = collect_subtree_triangles(child_triangle, children_triangles);
                for triangle in subtree {
                    for vertex_index in 0..3 {
                        lifted_triangles[triangle][vertex_index] = rotate_point_about_axis(
                            lifted_triangles[triangle][vertex_index],
                            (parent_a, parent_b),
                            angle,
                        );
                    }
                }
            }
            queue.push_back((
                child_triangle,
                breadth_first_depth_from_parent,
                graph_distance_from_parent,
            ));
        }
    }
}

fn prepare_mesh_for_folding(
    mesh: &mut WeakMesh,
) -> (WeldedMesh, Vec<ParentLink>, Vec<Vec<usize>>, Vec<[Vec3; 3]>, Vec<usize>) {
    let welded_mesh = Topology::build_topology(mesh)
        .welded_vertices()
        .triangles()
        .welded_vertices_per_triangle()
        .neighbors_per_triangle()
        .vertices_per_triangle()
        .texcoords_per_triangle()
        .build_welded_mesh();
    let triangle_count = welded_mesh.welded_vertices_per_triangle.len();
    let mut dual_graph = build_dual_graph(&welded_mesh);
    let (parent_links, children_triangles) = build_parent_tree(triangle_count, &mut dual_graph);
    let mut local_vertices_per_triangle = Vec::with_capacity(triangle_count);
    for triangle_index in 0..triangle_count {
        let [vertex_a, vertex_b, vertex_c] = welded_mesh.vertices_per_triangle[triangle_index];
        local_vertices_per_triangle.push(derive_local_plane_vertices(vertex_a, vertex_b, vertex_c));
    }

    let mut unfolded_triangles = vec![[Vec2::ZERO; 3]; triangle_count];
    let mut is_already_unfolded = vec![false; triangle_count];
    for triangle in 0..triangle_count {
        if is_already_unfolded[triangle] {
            continue;
        }
        unfolded_triangles[triangle] = anchor_welded_triangle(
            local_vertices_per_triangle[triangle],
            &welded_mesh.welded_vertices_per_triangle[triangle],
        );
        is_already_unfolded[triangle] = true;
        let mut triangle_stack = vec![triangle];
        while let Some(parent_triangle) = triangle_stack.pop() {
            for &child_triangle in &children_triangles[parent_triangle] {
                if is_already_unfolded[child_triangle] {
                    continue;
                }
                let parent_link = parent_links[child_triangle];
                let aligned_child = align_child_to_parent(
                    &local_vertices_per_triangle[child_triangle],
                    &welded_mesh.welded_vertices_per_triangle[child_triangle],
                    &unfolded_triangles[parent_triangle],
                    &welded_mesh.welded_vertices_per_triangle[parent_triangle],
                    parent_link.parent_local_edge.unwrap(),
                    parent_link.child_local_edge.unwrap(),
                    parent_link.welded_edge.unwrap(),
                );
                unfolded_triangles[child_triangle] = aligned_child;
                is_already_unfolded[child_triangle] = true;
                triangle_stack.push(child_triangle);
            }
        }
    }

    let mut lifted_triangles = vec![[Vec3::ZERO; 3]; triangle_count];
    for triangle in 0..triangle_count {
        lifted_triangles[triangle][0] = lift_dimension(unfolded_triangles[triangle][0]);
        lifted_triangles[triangle][1] = lift_dimension(unfolded_triangles[triangle][1]);
        lifted_triangles[triangle][2] = lift_dimension(unfolded_triangles[triangle][2]);
    }

    let parent_triangles: Vec<usize> = (0..triangle_count)
        .filter(|&f| parent_links[f].parent.is_none())
        .collect();

    (
        welded_mesh,
        parent_links,
        children_triangles,
        lifted_triangles,
        parent_triangles,
    )
}

fn fit_unfolded_triangles_to_zoom_scale(unfolded_triangles: &mut [[Vec3; 3]]) {
    for triangle in 0..unfolded_triangles.len() {
        for vertex_index in 0..3 {
            unfolded_triangles[triangle][vertex_index].z = 0.0;
        }
    }
    let mut min_x = f32::MAX;
    let mut min_y = f32::MAX;
    let mut max_x = f32::MIN;
    let mut max_y = f32::MIN;
    for triangle in 0..unfolded_triangles.len() {
        for vertex_index in 0..3 {
            let vertex = unfolded_triangles[triangle][vertex_index];
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

    for triangle in 0..unfolded_triangles.len() {
        for vertex_index in 0..3 {
            unfolded_triangles[triangle][vertex_index].x =
                (unfolded_triangles[triangle][vertex_index].x - center_x) * step;
            unfolded_triangles[triangle][vertex_index].y =
                (unfolded_triangles[triangle][vertex_index].y - center_y) * step;
        }
    }
}

fn build_unfolded_mesh(
    _thread_borrow: &raylib::RaylibThread,
    unfolded_triangles: &[[Vec3; 3]],
    welded_mesh: &WeldedMesh,
) -> Mesh {
    let triangle_count = unfolded_triangles.len();
    let mut vertices = Vec::with_capacity(triangle_count * 3);
    let mut texcoords = Vec::with_capacity(triangle_count * 3);
    let mut indices = Vec::with_capacity(triangle_count * 3);
    for triangle in 0..triangle_count {
        for vertex_index in 0..3 {
            let vertex = unfolded_triangles[triangle][vertex_index];
            vertices.push(Vector3::new(vertex.x, vertex.y, vertex.z));
            let texcoord = welded_mesh.texcoords_per_triangle[triangle][vertex_index];
            texcoords.push(Vector2::new(texcoord.x, texcoord.y));
            indices.push((vertices.len() - 1) as u16);
        }
    }
    Mesh::init_mesh(&vertices)
        .texcoords(&texcoords)
        .indices(&indices)
        .build(_thread_borrow)
        .unwrap()
}

pub fn align_to_welded_edge(
    parent_triangle_welded_vertices: &[WeldedVertex; 3],
    local_edge: (u8, u8),
    welded_edge: WeldedEdge,
) -> (u8, u8) {
    let local_vertex_a = local_edge.0 as usize;
    let local_vertex_b = local_edge.1 as usize;
    let welded_vertex_a = parent_triangle_welded_vertices[local_vertex_a];
    let welded_vertex_b = parent_triangle_welded_vertices[local_vertex_b];
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
    let mut aligned_child_triangle = [aligned_child_a, aligned_child_b, aligned_child_c];
    let parent_c = parent_vertices[edge_opposing_vertex(aligned_parent_edge) as usize];
    let child_c = aligned_child_triangle[edge_opposing_vertex(aligned_child_edge) as usize];
    let parent_sign = parent_ab.perp_dot(parent_c - parent_a);
    let child_sign = parent_ab.perp_dot(child_c - parent_a);
    //cases: 0) either negative -> negative. 1) Both positive = positive -> flip the child 2) both negative = positive -> flip the child
    // if (parent_sign * child_sign).is_sign_positive() {
    if (parent_sign * child_sign) > 1e-6 {
        for child_vertex in &mut aligned_child_triangle {
            let parent_offset = *child_vertex - parent_a;
            let x_aligned_magnitude = parent_offset.dot(parent_x_axis);
            let y_aligned_magnitude = parent_offset.dot(parent_y_axis);
            let x_aligned_component = parent_x_axis * x_aligned_magnitude;
            let y_aligned_component = parent_y_axis * -y_aligned_magnitude;
            *child_vertex = parent_a + x_aligned_component + y_aligned_component;
        }
    }
    aligned_child_triangle
}

pub fn derive_local_plane_vertices(a: Vec3, b: Vec3, c: Vec3) -> [Vec2; 3] {
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

pub fn anchor_welded_triangle(triangle: [Vec2; 3], welded_triangle: &[WeldedVertex; 3]) -> [Vec2; 3] {
    // base edge stability rule: edge between the two SMALLEST welded vertices
    let mut indices = [0, 1, 2];
    indices.sort_by_key(|&i| welded_triangle[i].id);
    let smallest_index = indices[0];
    let second_smallest_index = indices[1];
    let largest_index = indices[2];

    let a = triangle[smallest_index];
    let b = triangle[second_smallest_index];
    let c = triangle[largest_index];

    let ab = b - a;
    let ab_x_magnitude = ab.length();
    if ab_x_magnitude <= 0.0 {
        return triangle;
    }
    let x_axis = ab.normalize_or_zero();
    let perpendicular_rotation = Vec2::new((PI * 0.5).cos(), 1.0);
    let y_axis = x_axis.rotate(perpendicular_rotation);
    // let y_axis = Vec2::new(-x_axis.y, x_axis.x);
    let ac = c - a;
    let ac_x_magnitude = ac.dot(x_axis);
    let ac_y_magnitude = ac.dot(y_axis);

    let stable_a = Vec2::new(0.0, 0.0);
    let stable_b = Vec2::new(ab_x_magnitude, 0.0);
    let stable_c = Vec2::new(ac_x_magnitude, ac_y_magnitude);
    let mut stable_welded_triangle = [Vec2::ZERO; 3];
    stable_welded_triangle[smallest_index] = stable_a;
    stable_welded_triangle[second_smallest_index] = stable_b;
    stable_welded_triangle[largest_index] = stable_c;
    stable_welded_triangle
}

pub fn signed_dihedral_between(
    parent_id: usize,
    child_id: usize,
    parent_edge_local: (u8, u8),
    welded: &WeldedMesh,
) -> f32 {
    let [parent_a_world, parent_b_world, parent_c_world] = welded.vertices_per_triangle[parent_id];
    let [child_a, child_b, child_c] = welded.vertices_per_triangle[child_id];
    let parent_triangle_normal = triangle_normal(parent_a_world, parent_b_world, parent_c_world);
    let child_triangle_normal = triangle_normal(child_a, child_b, child_c);
    let parent_triangle_world = [parent_a_world, parent_b_world, parent_c_world];
    let parent_a_local = parent_triangle_world[parent_edge_local.0 as usize];
    let parent_b_local = parent_triangle_world[parent_edge_local.1 as usize];
    let x_axis = (parent_b_local - parent_a_local).normalize_or_zero();

    let cosine = parent_triangle_normal.dot(child_triangle_normal).clamp(-1.0, 1.0);
    let dihedral_angle = cosine.acos();
    let sine = x_axis.dot(parent_triangle_normal.cross(child_triangle_normal));
    // if sine >= 0.0 {
    //     dihedral_angle
    // } else {
    //     -dihedral_angle
    // }
    dihedral_angle * sine.signum()
}

pub fn align_to_original_pose(unfolded_triangles_lifted: &mut [[Vec3; 3]], welded_mesh: &WeldedMesh, root: usize) {
    let [unfolded_a, unfolded_b, unfolded_c] = unfolded_triangles_lifted[root];
    let unfolded_x_axis = (unfolded_b - unfolded_a).normalize_or_zero();
    let unfolded_z_axis = triangle_normal(unfolded_a, unfolded_b, unfolded_c);
    let unfolded_y_axis = unfolded_z_axis.cross(unfolded_x_axis).normalize_or_zero();

    let [folded_a, folded_b, folded_c] = welded_mesh.vertices_per_triangle[root];
    let folded_x_axis = (folded_b - folded_a).normalize_or_zero();
    let folded_z_axis = triangle_normal(folded_a, folded_b, folded_c);
    let folded_y_axis = folded_z_axis.cross(folded_x_axis).normalize_or_zero();

    for unfolded_triangle in 0..unfolded_triangles_lifted.len() {
        for i in 0..3 {
            let unfolded_edge = unfolded_triangles_lifted[unfolded_triangle][i] - unfolded_a;
            let unfolded_x_magnitude = unfolded_edge.dot(unfolded_x_axis);
            let unfolded_y_magnitude = unfolded_edge.dot(unfolded_y_axis);
            let unfolded_z_magnitude = unfolded_edge.dot(unfolded_z_axis);
            unfolded_triangles_lifted[unfolded_triangle][i] = folded_a
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
