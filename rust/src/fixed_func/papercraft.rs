use raylib::math::glam::Vec3;
use raylib::models::{Mesh, WeakMesh};
use std::collections::HashMap;
use std::f32::consts::PI;
use std::mem::zeroed;
use std::ptr::null_mut;
use std::slice::from_raw_parts;

pub const GORE_COUNT: usize = 0;
pub const MERIDIAN_BAND_FRACTION: f32 = 0.2;
pub const PADDING: f32 = 0.0;
pub const PAGE_WIDTH: f32 = 0.0;
pub const TARGET_MAX_EXTENT: f32 = 1.9;
pub const ANGLE_LIMIT: f32 = f32::INFINITY;
pub const RING_BUCKET_SCALE: f32 = 1e5;

pub fn unfold_sphere_like(mesh: &mut WeakMesh) -> Mesh {
    ensure_indices_exist(mesh);
    let triangle_count = mesh.triangleCount as usize;
    let vertex_count = mesh.vertexCount as usize;
    // if triangle_count == 0 {
    //     return unsafe { zeroed() };
    // }
    let src_vertices = unsafe { from_raw_parts(mesh.vertices, vertex_count * 3) };
    let src_indices = unsafe { from_raw_parts(mesh.indices, triangle_count * 3) };
    let src_texcoords = unsafe { from_raw_parts(mesh.texcoords, vertex_count * 2) };

    let (polar_angle_per_vertex, azimuth_per_vertex) = compute_spherical_angles(src_vertices);
    let ring_id_per_vertex = compute_ring_ids_by_polar_angle(&polar_angle_per_vertex);

    let preferred_meridian_angles: Vec<f32> = (0..GORE_COUNT)
        .map(|gore_index| gore_index as f32 * (2.0 * PI / GORE_COUNT as f32))
        .collect();
    let half_meridian_band_angle = (PI / GORE_COUNT as f32) * MERIDIAN_BAND_FRACTION;

    let local_triangle_corners = compute_local_triangle_corners(src_vertices, src_indices);
    let min_azimuth_vertex_per_ring = compute_min_azimuth_vertex_per_ring(&ring_id_per_vertex, &azimuth_per_vertex);

    let mut shared_edges = compute_shared_edges(src_indices, src_vertices);

    let mut triangle_index_edge_pairs: Vec<Option<(usize, u16, u16)>> = vec![None; triangle_count];
    let mut is_chart_root_face: Vec<bool> = vec![true; triangle_count];
    let mut unique_triangle_indices: Vec<usize> = (0..triangle_count).collect();
    let mut union_find_rank: Vec<u8> = vec![0; triangle_count];

    for edge in &mut shared_edges {
        if should_cut_edge(
            edge,
            &ring_id_per_vertex,
            &min_azimuth_vertex_per_ring,
            &azimuth_per_vertex,
            &preferred_meridian_angles,
            half_meridian_band_angle,
        ) {
            continue;
        }
        let root_a = what_the_fuck(&mut unique_triangle_indices, edge.face_a);
        let root_b = what_the_fuck(&mut unique_triangle_indices, edge.face_b);
        if root_a == root_b {
            continue;
        }
        triangle_index_edge_pairs[edge.face_b] = Some((edge.face_a, edge.edge_start_index, edge.edge_end_index));
        is_chart_root_face[edge.face_b] = false;
        what_the_fuck_2(&mut unique_triangle_indices, &mut union_find_rank, root_a, root_b);
    }

    let mut children_faces_per_face: Vec<Vec<usize>> = vec![Vec::new(); triangle_count];
    for face_index in 0..triangle_count {
        if let Some((parent_face, _, _)) = triangle_index_edge_pairs[face_index] {
            children_faces_per_face[parent_face].push(face_index);
        }
    }

    let mut placed_corners_per_face: Vec<[[f32; 2]; 3]> = vec![[[0.0; 2]; 3]; triangle_count];
    let mut placement_done_per_face: Vec<bool> = vec![false; triangle_count];
    let mut face_stack: Vec<usize> = Vec::new();

    for i in 0..triangle_count {
        if is_chart_root_face[i] {
            placed_corners_per_face[i] = local_triangle_corners[i];
            placement_done_per_face[i] = true;
            face_stack.push(i);

            while let Some(current_face) = face_stack.pop() {
                for &child_face in &children_faces_per_face[current_face] {
                    if placement_done_per_face[child_face] {
                        continue;
                    }
                    if let Some((parent_face, shared_vertex_a, shared_vertex_b)) = triangle_index_edge_pairs[child_face]
                    {
                        let parent_triangle_indices = &src_indices[parent_face * 3..parent_face * 3 + 3];
                        let child_triangle_indices = &src_indices[child_face * 3..child_face * 3 + 3];

                        let placed_child = place_child_triangle(
                            parent_triangle_indices,
                            child_triangle_indices,
                            &placed_corners_per_face[parent_face],
                            &local_triangle_corners[child_face],
                            shared_vertex_a,
                            shared_vertex_b,
                        );

                        placed_corners_per_face[child_face] = placed_child;
                        placement_done_per_face[child_face] = true;
                        face_stack.push(child_face);
                    }
                }
            }
        }
    }

    let mut chart_root_face_of: Vec<usize> = vec![usize::MAX; triangle_count];
    for i in 0..triangle_count {
        let mut root = i;
        while !is_chart_root_face[root] {
            root = triangle_index_edge_pairs[root].unwrap().0;
        }
        chart_root_face_of[i] = root;
    }

    let mut chart_bounds_by_root: HashMap<usize, ([f32; 2], [f32; 2])> = HashMap::new();
    for i in 0..triangle_count {
        let chart_root = chart_root_face_of[i];
        let entry = chart_bounds_by_root
            .entry(chart_root)
            .or_insert(([f32::MAX; 2], [f32::MIN; 2]));
        for corner in 0..3 {
            let p = placed_corners_per_face[i][corner];
            if p[0] < entry.0[0] {
                entry.0[0] = p[0];
            }
            if p[1] < entry.0[1] {
                entry.0[1] = p[1];
            }
            if p[0] > entry.1[0] {
                entry.1[0] = p[0];
            }
            if p[1] > entry.1[1] {
                entry.1[1] = p[1];
            }
        }
    }

    let mut chart_order: Vec<usize> = chart_bounds_by_root.keys().copied().collect();
    chart_order.sort_by(|a, b| {
        let bounds_a = chart_bounds_by_root.get(a).unwrap();
        let bounds_b = chart_bounds_by_root.get(b).unwrap();
        let area_a = (bounds_a.1[0] - bounds_a.0[0]) * (bounds_a.1[1] - bounds_a.0[1]);
        let area_b = (bounds_b.1[0] - bounds_b.0[0]) * (bounds_b.1[1] - bounds_b.0[1]);
        area_b.partial_cmp(&area_a).unwrap().then(a.cmp(b))
    });

    let page_width = if PAGE_WIDTH <= 0.0 {
        let mut total_area = 0.0;
        for root in &chart_order {
            let (min_bounds, max_bounds) = chart_bounds_by_root[root];
            let chart_width = (max_bounds[0] - min_bounds[0]) + PADDING;
            let chart_height = (max_bounds[1] - min_bounds[1]) + PADDING;
            total_area += chart_width * chart_height;
        }
        total_area.sqrt().max(1e-3)
    } else {
        PAGE_WIDTH
    };

    let mut chart_offset_by_root: HashMap<usize, [f32; 2]> = HashMap::new();
    let mut layout_cursor_x = 0.0;
    let mut layout_cursor_y = 0.0;
    let mut current_row_height = 0.0;

    for chart_root in &chart_order {
        let (min_bounds, max_bounds) = chart_bounds_by_root[chart_root];
        let chart_width = max_bounds[0] - min_bounds[0];
        let chart_height = max_bounds[1] - min_bounds[1];
        let required_width = chart_width + PADDING;
        if layout_cursor_x > 0.0 && layout_cursor_x + required_width > page_width {
            layout_cursor_x = 0.0;
            layout_cursor_y += current_row_height + PADDING;
            current_row_height = 0.0;
        }

        chart_offset_by_root.insert(
            *chart_root,
            [layout_cursor_x - min_bounds[0], layout_cursor_y - min_bounds[1]],
        );
        layout_cursor_x += required_width;
        if chart_height > current_row_height {
            current_row_height = chart_height;
        }
    }
    let mut unfolded_vertices: Vec<f32> = Vec::with_capacity(triangle_count * 9);
    let mut unfolded_indices: Vec<u16> = Vec::with_capacity(triangle_count * 3);
    let mut unfolded_texcoords: Vec<f32> = Vec::with_capacity(triangle_count * 6);
    let mut mapping_source_vertex_and_chart_to_index: HashMap<(u32, usize), u16> = HashMap::new();

    for i in 0..triangle_count {
        let chart_root = chart_root_face_of[i];
        let chart_offset = chart_offset_by_root[&chart_root];
        let triangle_indices = &src_indices[i * 3..i * 3 + 3];
        for corner in 0..3 {
            let source_vertex_id = triangle_indices[corner] as usize;
            let map_key = (source_vertex_id as u32, chart_root);
            let new_vertex_index = *mapping_source_vertex_and_chart_to_index
                .entry(map_key)
                .or_insert_with(|| {
                    let placed_point = placed_corners_per_face[i][corner];
                    unfolded_vertices.extend_from_slice(&[
                        placed_point[0] + chart_offset[0],
                        placed_point[1] + chart_offset[1],
                        0.0,
                    ]);
                    unfolded_texcoords.extend_from_slice(&[
                        src_texcoords[source_vertex_id * 2 + 0],
                        src_texcoords[source_vertex_id * 2 + 1],
                    ]);
                    (unfolded_vertices.len() / 3 - 1) as u16
                });
            unfolded_indices.push(new_vertex_index);
        }
    }

    let (mut min_x, mut max_x, mut min_y, mut max_y) = (f32::MAX, f32::MIN, f32::MAX, f32::MIN);
    for i in (0..unfolded_vertices.len()).step_by(3) {
        let x = unfolded_vertices[i + 0];
        let y = unfolded_vertices[i + 1];
        if x < min_x {
            min_x = x;
        }
        if x > max_x {
            max_x = x;
        }
        if y < min_y {
            min_y = y;
        }
        if y > max_y {
            max_y = y;
        }
    }
    let page_width_span = (max_x - min_x).max(1e-6);
    let page_height_span = (max_y - min_y).max(1e-6);
    let scale_factor = TARGET_MAX_EXTENT / page_width_span.max(page_height_span);
    let center_x = 0.5 * (min_x + max_x);
    let center_y = 0.5 * (min_y + max_y);

    for i in (0..unfolded_vertices.len()).step_by(3) {
        unfolded_vertices[i + 0] = (unfolded_vertices[i + 0] - center_x) * scale_factor;
        unfolded_vertices[i + 1] = (unfolded_vertices[i + 1] - center_y) * scale_factor;
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

#[inline]
fn normalize_or(vector: Vec3) -> Vec3 {
    //TODO: this is checking if its greater than 0??
    if vector.length() > 0.0 {
        vector / vector.length()
    } else {
        Vec3::ZERO
    }
}

#[inline]
fn get_vertex_from_vertex_index(vertices: &[f32], vertex_index: usize) -> Vec3 {
    Vec3::new(
        vertices[vertex_index * 3 + 0],
        vertices[vertex_index * 3 + 1],
        vertices[vertex_index * 3 + 2],
    )
}

fn ensure_indices_exist(mesh: &mut WeakMesh) {
    if mesh.indices.is_null() {
        let vertex_count = mesh.vertexCount as usize;
        let indices: Vec<u16> = (0..vertex_count as u32).map(|i| i as u16).collect();
        mesh.indices = Box::leak(indices.into_boxed_slice()).as_mut_ptr();
        mesh.triangleCount = (vertex_count / 3) as i32;
    }
}

#[inline]
fn wrap_2pi(mut a: f32) -> f32 {
    if a < 0.0 {
        a += 2.0 * PI;
    }
    if a >= 2.0 * PI {
        a -= 2.0 * PI;
    }
    a
}

#[inline]
fn circular_mean(a: f32, b: f32) -> f32 {
    let distance = (a - b).abs();
    if distance <= PI {
        return wrap_2pi((a + b) * 0.5);
    }
    if a > b {
        wrap_2pi(((a - 2.0 * PI) + b) * 0.5)
    } else {
        wrap_2pi((a + (b - 2.0 * PI)) * 0.5)
    }
}

#[inline]
fn circular_distance(a: f32, b: f32) -> f32 {
    let mut distance = (a - b).abs();
    if distance > PI {
        distance = 2.0 * PI - distance;
    }
    distance
}

#[inline]
fn quantize_angle_for_decisions(angle: f32) -> f32 {
    (angle * 4096.0).round() / 4096.0
}

#[derive(Copy, Clone)]
struct SharedEdge {
    face_a: usize,
    face_b: usize,
    edge_start_index: u16,
    edge_end_index: u16,
    angle_fold_over_edge: f32,
}

fn compute_spherical_angles(vertices: &[f32]) -> (Vec<f32>, Vec<f32>) {
    let vertex_count = vertices.len() / 3;
    let mut polar_angle_per_vertex = Vec::with_capacity(vertex_count);
    let mut azimuth_per_vertex = Vec::with_capacity(vertex_count);
    for i in 0..vertex_count {
        let x = vertices[i * 3 + 0];
        let y = vertices[i * 3 + 1];
        let z = vertices[i * 3 + 2];
        let radius = (x * x + y * y + z * z).sqrt().max(1e-9);

        let polar_angle = (y / radius).clamp(-1.0, 1.0).acos();
        let mut azimuth = z.atan2(x);
        azimuth = wrap_2pi(azimuth);

        polar_angle_per_vertex.push(polar_angle);
        azimuth_per_vertex.push(azimuth);
    }
    (polar_angle_per_vertex, azimuth_per_vertex)
}

fn compute_ring_ids_by_polar_angle(polar_angle_per_vertex: &[f32]) -> Vec<usize> {
    let mut unique_buckets: Vec<i32> = polar_angle_per_vertex
        .iter()
        .map(|&a| (a * RING_BUCKET_SCALE).round() as i32)
        .collect();
    unique_buckets.sort(); //TODO: what the fuck is this sorting

    let mut ring_id_per_vertex = vec![0usize; polar_angle_per_vertex.len()];
    for (i, &angle) in polar_angle_per_vertex.iter().enumerate() {
        let bucket = (angle * RING_BUCKET_SCALE).round() as i32;
        let ring_id = unique_buckets.binary_search(&bucket).expect("bucket must exist");
        ring_id_per_vertex[i] = ring_id;
    }
    ring_id_per_vertex
}

fn compute_min_azimuth_vertex_per_ring(ring_id_per_vertex: &[usize], azimuth_per_vertex: &[f32]) -> Vec<Option<usize>> {
    let max_ring_id = *ring_id_per_vertex.iter().max().unwrap_or(&0);
    let mut min_azimuth_vertex_per_ring: Vec<Option<usize>> = vec![None; max_ring_id + 1];
    for i in 0..ring_id_per_vertex.len() {
        let ring_id = ring_id_per_vertex[i];
        if let Some(current_min) = min_azimuth_vertex_per_ring[ring_id] {
            if azimuth_per_vertex[i] < azimuth_per_vertex[current_min] {
                min_azimuth_vertex_per_ring[ring_id] = Some(i);
            }
        } else {
            min_azimuth_vertex_per_ring[ring_id] = Some(i);
        }
    }
    min_azimuth_vertex_per_ring
}

fn compute_local_triangle_corners(vertices: &[f32], indices: &[u16]) -> Vec<[[f32; 2]; 3]> {
    let triangle_count = indices.len() / 3;
    let mut local_triangle_corners: Vec<[[f32; 2]; 3]> = Vec::with_capacity(triangle_count);
    for i in 0..triangle_count {
        let tri = &indices[i * 3..i * 3 + 3];
        let a = tri[0] as usize;
        let b = tri[1] as usize;
        let c = tri[2] as usize;

        let position_a = get_vertex_from_vertex_index(vertices, a);
        let position_b = get_vertex_from_vertex_index(vertices, b);
        let position_c = get_vertex_from_vertex_index(vertices, c);

        let edge_ab = position_b - position_a;
        let axis_right = edge_ab.normalize_or_zero();
        let axis_normal = edge_ab.cross(position_c - position_a).normalize_or_zero();
        let axis_up = axis_normal.cross(axis_right);

        let local_a = [0.0, 0.0];
        let local_b = [edge_ab.length(), 0.0];
        let vector_ac = position_c - position_a;
        let local_c = [vector_ac.dot(axis_right), vector_ac.dot(axis_up)];

        local_triangle_corners.push([local_a, local_b, local_c]);
    }
    local_triangle_corners
}

fn face_normal(vertices: &[f32], triangle: &[u16]) -> Vec3 {
    let vertex_a = get_vertex_from_vertex_index(vertices, triangle[0] as usize);
    let vertex_b = get_vertex_from_vertex_index(vertices, triangle[1] as usize);
    let vertex_c = get_vertex_from_vertex_index(vertices, triangle[2] as usize);
    (vertex_b - vertex_a).cross(vertex_c - vertex_a).normalize_or_zero()
}

fn compute_shared_edges(indices: &[u16], vertices: &[f32]) -> Vec<SharedEdge> {
    let triangle_count = indices.len() / 3; // this is because indices is giant list of per vertex mappings to triangles
    let mut shared_edges: Vec<SharedEdge> = Vec::new();
    let mut edges_to_triangle_indices_map: HashMap<(u16, u16), usize> = HashMap::new();

    for index in 0..triangle_count {
        let current_triangle = &indices[index * 3..index * 3 + 3];
        for j in 0..3 {
            let vertex_index_a = current_triangle[j];
            let vertex_index_b = current_triangle[(j + 1) % 3];
            let edge_ab = if vertex_index_a < vertex_index_b {
                (vertex_index_a, vertex_index_b)
            } else {
                (vertex_index_b, vertex_index_a)
            };

            if let Some(&adjacent_triangle_index) = edges_to_triangle_indices_map.get(&edge_ab) {
                let current_triangle_reget = &indices[index * 3..index * 3 + 3];
                let adjacent_triangle = &indices[adjacent_triangle_index * 3..adjacent_triangle_index * 3 + 3];
                let current_triangle_face_normal = face_normal(vertices, current_triangle_reget);
                let adjacent_triangle_face_normal = face_normal(vertices, adjacent_triangle);
                //TODO: why the fuck do we have to clamp here?
                let angle_fold_over_edge = current_triangle_face_normal
                    .dot(adjacent_triangle_face_normal)
                    .clamp(-1.0, 1.0)
                    .acos();
                shared_edges.push(SharedEdge {
                    face_a: adjacent_triangle_index,
                    face_b: index,
                    edge_start_index: edge_ab.0,
                    edge_end_index: edge_ab.1,
                    angle_fold_over_edge,
                });
            } else {
                edges_to_triangle_indices_map.insert(edge_ab, index);
            }
        }
    }
    // TODO: I dont like this sorting shit, i want all the triangles to be near eachother fine
    // shared_edges.sort_by(|left, right| {
    //     left.angle_fold_over_edge
    //         .partial_cmp(&right.angle_fold_over_edge)
    //         .unwrap()
    //         .then_with(|| {
    //             (left.edge_start_index, left.edge_end_index).cmp(&(right.edge_start_index, right.edge_end_index))
    //         })
    //         .then(left.face_a.cmp(&right.face_a))
    //         .then(left.face_b.cmp(&right.face_b))
    // });
    shared_edges
}

fn should_cut_edge(
    edge: &SharedEdge,
    ring_id_per_vertex: &[usize],
    min_azimuth_vertex_per_ring: &[Option<usize>],
    azimuth_per_vertex: &[f32],
    preferred_meridian_angles: &[f32],
    half_meridian_band_angle: f32,
) -> bool {
    let vertex_index_a = edge.edge_start_index as usize;
    let vertex_index_b = edge.edge_end_index as usize;
    let circular_mean = circular_mean(azimuth_per_vertex[vertex_index_a], azimuth_per_vertex[vertex_index_b]);
    let avg_azimuth = quantize_angle_for_decisions(circular_mean);
    let near_preferred_meridian = preferred_meridian_angles
        .iter()
        .any(|&m| circular_distance(avg_azimuth, quantize_angle_for_decisions(m)) < half_meridian_band_angle);

    let same_ring = ring_id_per_vertex[vertex_index_a] == ring_id_per_vertex[vertex_index_b];
    let at_ring_split = same_ring
        && min_azimuth_vertex_per_ring[ring_id_per_vertex[vertex_index_a]]
            .map(|min_vertex| min_vertex == vertex_index_a || min_vertex == vertex_index_b)
            .unwrap_or(false);

    near_preferred_meridian || at_ring_split || edge.angle_fold_over_edge > ANGLE_LIMIT
}

fn place_child_triangle(
    parent_triangle_indices: &[u16],
    child_triangle_indices: &[u16],
    placed_corners_parent: &[[f32; 2]; 3],
    local_corners_child: &[[f32; 2]; 3],
    shared_vertex_a: u16,
    shared_vertex_b: u16,
) -> [[f32; 2]; 3] {
    let mut parent_anchor_a = [0.0; 2];
    let mut parent_anchor_b = [0.0; 2];
    for corner in 0..3 {
        if parent_triangle_indices[corner] == shared_vertex_a {
            parent_anchor_a = placed_corners_parent[corner];
        }
        if parent_triangle_indices[corner] == shared_vertex_b {
            parent_anchor_b = placed_corners_parent[corner];
        }
    }

    let mut child_local_anchor_a = [0.0; 2];
    let mut child_local_anchor_b = [0.0; 2];
    for corner in 0..3 {
        let source_vertex_id = child_triangle_indices[corner];
        if source_vertex_id == shared_vertex_a {
            child_local_anchor_a = local_corners_child[corner];
        } else if source_vertex_id == shared_vertex_b {
            child_local_anchor_b = local_corners_child[corner];
        }
    }

    let child_local_edge = [
        child_local_anchor_b[0] - child_local_anchor_a[0],
        child_local_anchor_b[1] - child_local_anchor_a[1],
    ];
    let parent_edge_vector = [
        parent_anchor_b[0] - parent_anchor_a[0],
        parent_anchor_b[1] - parent_anchor_a[1],
    ];

    let child_local_edge_length = (child_local_edge[0] * child_local_edge[0]
        + child_local_edge[1] * child_local_edge[1])
        .sqrt()
        .max(1e-12);
    let parent_edge_length = (parent_edge_vector[0] * parent_edge_vector[0]
        + parent_edge_vector[1] * parent_edge_vector[1])
        .sqrt()
        .max(1e-12);

    let child_local_edge_direction = [
        child_local_edge[0] / child_local_edge_length,
        child_local_edge[1] / child_local_edge_length,
    ];
    let parent_edge_direction = [
        parent_edge_vector[0] / parent_edge_length,
        parent_edge_vector[1] / parent_edge_length,
    ];

    let cosine = child_local_edge_direction[0] * parent_edge_direction[0]
        + child_local_edge_direction[1] * parent_edge_direction[1];
    let sine = child_local_edge_direction[0] * parent_edge_direction[1]
        - child_local_edge_direction[1] * parent_edge_direction[0];
    let scale = parent_edge_length / child_local_edge_length;

    let mut placed_child_corners = [[0.0; 2]; 3];
    for corner in 0..3 {
        let local_point = local_corners_child[corner];
        let local_x = (local_point[0] - child_local_anchor_a[0]) * scale;
        let local_y = (local_point[1] - child_local_anchor_a[1]) * scale;

        let rotated_x = local_x * cosine - local_y * sine;
        let rotated_y = local_x * sine + local_y * cosine;

        placed_child_corners[corner] = [parent_anchor_a[0] + rotated_x, parent_anchor_a[1] + rotated_y];
    }
    placed_child_corners
}

fn what_the_fuck(unique_triangle_indices: &mut [usize], triangle_index: usize) -> usize {
    if unique_triangle_indices[triangle_index] == triangle_index {
        triangle_index
    } else {
        let next_triangle_index = what_the_fuck(unique_triangle_indices, unique_triangle_indices[triangle_index]);
        unique_triangle_indices[triangle_index] = next_triangle_index;
        next_triangle_index
    }
}

fn what_the_fuck_2(parent: &mut [usize], rank: &mut [u8], a: usize, b: usize) {
    let mut root_a = what_the_fuck(parent, a);
    let mut root_b = what_the_fuck(parent, b);
    if root_a == root_b {
        return;
    }
    if rank[root_a] < rank[root_b] {
        std::mem::swap(&mut root_a, &mut root_b);
    }
    parent[root_b] = root_a;
    if rank[root_a] == rank[root_b] {
        rank[root_a] += 1;
    }
}
