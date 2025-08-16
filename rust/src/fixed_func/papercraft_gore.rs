use crate::fixed_func::constants::TWO_PI;
use raylib::math::glam::Vec3;
use raylib::models::{Mesh, WeakMesh};
use std::collections::HashMap;
use std::f32::consts::PI;
use std::mem::{swap, zeroed};
use std::ptr::null_mut;
use std::slice::from_raw_parts;

pub const GORE_COUNT: usize = 1;
pub const MERIDIAN_BAND_FRACTION: f32 = 0.1;
pub const TARGET_MAX_EXTENT: f32 = 1.9;
pub const ANGLE_LIMIT: f32 = f32::INFINITY;
pub const SPHERICAL_VERTICALITY_RING_BUCKETS_SCALAR: f32 = 1.0;
pub const AZIMUTH_QUANTIZATION_SCALAR: f32 = 4096.0;

#[derive(Copy, Clone, Debug)]
struct SharedEdge {
    face_a: usize,
    face_b: usize,
    edge: (u16, u16),
    angle_fold_over_edge: f32,
}

pub fn unfold_gore(mesh: &mut WeakMesh) -> Mesh {
    let triangle_count = mesh.triangleCount as usize;
    let vertex_count = mesh.vertexCount as usize;
    let src_vertices = unsafe { from_raw_parts(mesh.vertices, vertex_count * 3) };
    let src_indices = unsafe { from_raw_parts(mesh.indices, triangle_count * 3) };
    let src_texcoords = unsafe { from_raw_parts(mesh.texcoords, vertex_count * 2) };

    let (spherical_verticality_polar_angle_per_vertex, depth_azimuth_per_vertex) =
        compute_spherical_verticality_and_depth_angles(src_vertices);
    // this matches exactly the order of the indexed vertices and corresponds a vertex to a ring bucket
    // (think of a bucket like a strip wrapped horizontally around the sphere at differing heights to kind of classify groupings of vertices
    let verticality_ring_ids_per_vertex =
        group_vertices_by_vertical_rings_wrapping_sphere(&spherical_verticality_polar_angle_per_vertex);

    //TODO: wouldnt it be better to just make the whole data structure of all these indexed and categorized vertex metadata?
    let closest_vertex_to_observer_per_ring =
        compute_closest_vertex_to_observer_per_ring(&verticality_ring_ids_per_vertex, &depth_azimuth_per_vertex);

    let full_meridian_gore_angles: Vec<f32> = (0..GORE_COUNT)
        .map(|gore_index| gore_index as f32 * (TWO_PI / GORE_COUNT as f32))
        .collect();
    let half_meridian_gore_subdivision = (PI / GORE_COUNT as f32) * MERIDIAN_BAND_FRACTION;
    let shared_edges = compute_shared_edges(src_indices, src_vertices);

    let mut parent_face_per_target_face: Vec<usize> = (0..triangle_count).collect();
    let mut is_parent_face: Vec<bool> = vec![true; triangle_count];
    let mut num_of_children_per_parent_face: Vec<u8> = vec![0; triangle_count];
    let mut edge_path_child_to_parent: Vec<Option<(usize, (u16, u16))>> = vec![None; triangle_count];

    for (i, shared_edge) in &mut shared_edges.iter().enumerate() {
        // println!("cutting decisions for shared_edge {:?} at {}", shared_edge, i);
        if should_cut_edge(
            shared_edge,
            &verticality_ring_ids_per_vertex,
            &closest_vertex_to_observer_per_ring,
            &depth_azimuth_per_vertex,
            &full_meridian_gore_angles,
            half_meridian_gore_subdivision,
        ) {
            // println!("CUT OCCURRED");
            continue;
        }
        let parent_of_face_a =
            find_and_update_parent_face_for_target_face(&mut parent_face_per_target_face, shared_edge.face_a);
        // println!("face a's parent face is: {} ", parent_of_face_a);
        let parent_of_face_b =
            find_and_update_parent_face_for_target_face(&mut parent_face_per_target_face, shared_edge.face_b);
        // println!("face b's parent face is: {} ", parent_of_face_b);
        if parent_of_face_a == parent_of_face_b {
            // println!("FULLY LOOPED AND CUT AT PARENT");
            continue;
        }
        edge_path_child_to_parent[shared_edge.face_b] = Some((shared_edge.face_a, shared_edge.edge));
        is_parent_face[shared_edge.face_b] = false;
        let mut grandparent_of_face_a =
            find_and_update_parent_face_for_target_face(&mut parent_face_per_target_face, parent_of_face_a);
        let mut grandparent_of_face_b =
            find_and_update_parent_face_for_target_face(&mut parent_face_per_target_face, parent_of_face_b);
        if grandparent_of_face_a == grandparent_of_face_b {
            // println!("FULLY LOOPED AND CUT AT GRANDPARENT");
            continue;
        }
        if num_of_children_per_parent_face[grandparent_of_face_a]
            < num_of_children_per_parent_face[grandparent_of_face_b]
        {
            swap(&mut grandparent_of_face_a, &mut grandparent_of_face_b);
        }
        parent_face_per_target_face[grandparent_of_face_b] = grandparent_of_face_a;
        if num_of_children_per_parent_face[grandparent_of_face_a]
            == num_of_children_per_parent_face[grandparent_of_face_b]
        {
            num_of_children_per_parent_face[grandparent_of_face_a] += 1;
        }
    }

    let mut children_faces_per_parent_face: Vec<Vec<usize>> = vec![Vec::new(); triangle_count];
    for child_face in 0..triangle_count {
        if let Some((parent_face, edge)) = edge_path_child_to_parent[child_face] {
            children_faces_per_parent_face[parent_face].push(child_face);
        }
    }

    let mut parent_vertices_draw_space: Vec<[[f32; 2]; 3]> = vec![[[0.0; 2]; 3]; triangle_count];
    let mut is_placed: Vec<bool> = vec![false; triangle_count];
    let mut face_stack: Vec<usize> = Vec::new();

    let all_vertices_local_space_per_face = compute_triangle_vertices_local_space(src_vertices, src_indices);
    for i in 0..triangle_count {
        if is_parent_face[i] {
            parent_vertices_draw_space[i] = all_vertices_local_space_per_face[i];
            is_placed[i] = true;
            face_stack.push(i);

            while let Some(parent_face) = face_stack.pop() {
                for &child_face_index in &children_faces_per_parent_face[parent_face] {
                    if is_placed[child_face_index] {
                        continue;
                    }
                    if let Some((parent_face_index, edge)) = edge_path_child_to_parent[child_face_index] {
                        let parent_face_indices = &src_indices[parent_face_index * 3..parent_face_index * 3 + 3];
                        let child_face_indices = &src_indices[child_face_index * 3..child_face_index * 3 + 3];

                        let child_vertices_draw_space = convert_child_vertices_from_local_space_to_draw_space(
                            parent_face_indices,
                            child_face_indices,
                            &parent_vertices_draw_space[parent_face_index],
                            &all_vertices_local_space_per_face[child_face_index],
                            edge,
                        );

                        parent_vertices_draw_space[child_face_index] = child_vertices_draw_space;
                        is_placed[child_face_index] = true;
                        face_stack.push(child_face_index);
                    }
                }
            }
        }
    }

    let mut parent_face_of_face: Vec<usize> = vec![usize::MAX; triangle_count];
    for face in 0..triangle_count {
        let parent_face = traverse_edge_path_child_to_parent(face, &is_parent_face, &edge_path_child_to_parent);
        parent_face_of_face[face] = parent_face;
    }

    let mut draw_space_bounds_per_parent: HashMap<usize, ([f32; 2], [f32; 2])> = HashMap::new();
    for face in 0..triangle_count {
        let parent_face = parent_face_of_face[face];
        let draw_space_bounds_for_parent_and_children = draw_space_bounds_per_parent
            .entry(parent_face)
            .or_insert(([f32::MAX; 2], [f32::MIN; 2]));
        for vertex_index in 0..3 {
            let parent_vertex_index = parent_vertices_draw_space[face][vertex_index];
            if parent_vertex_index[0] < draw_space_bounds_for_parent_and_children.0[0] {
                draw_space_bounds_for_parent_and_children.0[0] = parent_vertex_index[0];
            }
            if parent_vertex_index[1] < draw_space_bounds_for_parent_and_children.0[1] {
                draw_space_bounds_for_parent_and_children.0[1] = parent_vertex_index[1];
            }
            if parent_vertex_index[0] > draw_space_bounds_for_parent_and_children.1[0] {
                draw_space_bounds_for_parent_and_children.1[0] = parent_vertex_index[0];
            }
            if parent_vertex_index[1] > draw_space_bounds_for_parent_and_children.1[1] {
                draw_space_bounds_for_parent_and_children.1[1] = parent_vertex_index[1];
            }
        }
    }

    let mut parent_ordering_in_draw_space: Vec<usize> = draw_space_bounds_per_parent.keys().copied().collect();
    parent_ordering_in_draw_space.sort_by(|parent_face_index_a, parent_face_index_b| {
        let bounds_a = draw_space_bounds_per_parent.get(parent_face_index_a).unwrap();
        let bounds_b = draw_space_bounds_per_parent.get(parent_face_index_b).unwrap();
        let draw_space_area_a = (bounds_a.1[0] - bounds_a.0[0]) * (bounds_a.1[1] - bounds_a.0[1]);
        let draw_space_area_b = (bounds_b.1[0] - bounds_b.0[0]) * (bounds_b.1[1] - bounds_b.0[1]);
        draw_space_area_b
            .partial_cmp(&draw_space_area_a)
            .unwrap()
            .then(parent_face_index_a.cmp(parent_face_index_b))
    });
    let mut draw_space_area = 0.0;
    for parent_face_index in &parent_ordering_in_draw_space {
        let (min_bounds, max_bounds) = draw_space_bounds_per_parent[parent_face_index];
        let draw_space_width = max_bounds[0] - min_bounds[0];
        let draw_space_height = max_bounds[1] - min_bounds[1];
        draw_space_area += draw_space_width * draw_space_height;
    }
    let page_width = draw_space_area.sqrt();

    let mut draw_space_origin_per_parent: HashMap<usize, [f32; 2]> = HashMap::new();
    let mut draw_space_cursor_x = 0.0;
    let mut draw_space_cursor_y = 0.0;
    let mut current_draw_space_y_origin = 0.0;

    for parent in &parent_ordering_in_draw_space {
        let (min_bounds, max_bounds) = draw_space_bounds_per_parent[parent];
        let draw_space_width = max_bounds[0] - min_bounds[0];
        let draw_space_height = max_bounds[1] - min_bounds[1];
        if draw_space_cursor_x > 0.0 && draw_space_cursor_x + draw_space_width > page_width {
            draw_space_cursor_x = 0.0;
            draw_space_cursor_y += current_draw_space_y_origin;
            current_draw_space_y_origin = 0.0;
        }
        let origin_for_parent = [draw_space_cursor_x - min_bounds[0], draw_space_cursor_y - min_bounds[1]];
        draw_space_origin_per_parent.insert(*parent, origin_for_parent);
        draw_space_cursor_x += draw_space_width;
        if draw_space_height > current_draw_space_y_origin {
            current_draw_space_y_origin = draw_space_height;
        }
    }
    let mut unfolded_vertices: Vec<f32> = Vec::with_capacity(triangle_count * 9);
    let mut unfolded_indices: Vec<u16> = Vec::with_capacity(triangle_count * 3);
    let mut unfolded_texcoords: Vec<f32> = Vec::with_capacity(triangle_count * 6);
    let mut src_index_to_draw_space_index: HashMap<(u32, usize), u16> = HashMap::new();

    for face_index in 0..triangle_count {
        let parent_face_index = parent_face_of_face[face_index];
        let draw_space_origin = draw_space_origin_per_parent[&parent_face_index];
        let triangle = &src_indices[face_index * 3..face_index * 3 + 3];
        for index in 0..3 {
            let vertex_index = triangle[index] as usize;
            let new_vertex_index = *src_index_to_draw_space_index
                .entry((vertex_index as u32, parent_face_index))
                .or_insert_with(|| {
                    let draw_space_vertex = parent_vertices_draw_space[face_index][index];
                    unfolded_vertices.extend_from_slice(&[
                        draw_space_vertex[0] + draw_space_origin[0],
                        draw_space_vertex[1] + draw_space_origin[1],
                        0.0,
                    ]);
                    unfolded_texcoords
                        .extend_from_slice(&[src_texcoords[vertex_index * 2 + 0], src_texcoords[vertex_index * 2 + 1]]);
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
    let page_width_span = max_x - min_x;
    let page_height_span = max_y - min_y;
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
                    edge: edge_ab,
                    angle_fold_over_edge,
                });
            } else {
                edges_to_triangle_indices_map.insert(edge_ab, index);
            }
        }
    }
    // TODO: I dont like this sorting shit, i want all the triangles to be near eachother fine
    // TODO: THIS IS WHERE THE FUCKING UNION SHIT COMES IN DOESNT IT, IDK WHAT THE FUCK
    shared_edges.sort_by(|left, right| {
        left.angle_fold_over_edge
            .partial_cmp(&right.angle_fold_over_edge)
            .unwrap()
            .then_with(|| left.edge.cmp(&right.edge))
            .then(left.face_a.cmp(&right.face_a))
            .then(left.face_b.cmp(&right.face_b))
    });
    shared_edges
}

fn compute_spherical_verticality_and_depth_angles(vertices: &[f32]) -> (Vec<f32>, Vec<f32>) {
    let vertex_count = vertices.len() / 3;
    // angle between east/right-hand vector going counter clockwise (or just the whole fucking XZ plane?)
    let mut verticality_polar_angle_per_vertex = Vec::with_capacity(vertex_count);
    //angle between observer line of site vector going around the vertical y axis (which way is the vector order spinning?
    let mut depth_azimuth_per_vertex = Vec::with_capacity(vertex_count);
    for i in 0..vertex_count {
        let x = vertices[i * 3 + 0];
        let y = vertices[i * 3 + 1];
        let z = vertices[i * 3 + 2];
        let origin_to_vertex_radial_magnitude = (x.powi(2) + y.powi(2) + z.powi(2)).sqrt(); //.max(1e-9);
                                                                                            // looks like the whole XZ plane (soi imagine this as the like angle that goes up and around the XY plane
                                                                                            // (providing the vertical angle/height derivation of where the vertex is in y pretty much)
        let verticality_polar_angle = (y / origin_to_vertex_radial_magnitude).clamp(-1.0, 1.0).acos();
        // the depth of the vertex ( imagine this as the kind of depth angle with how close the vertex is to the observer)
        let mut depth_azimuth = z.atan2(x);
        depth_azimuth = wrap_2pi(depth_azimuth);

        verticality_polar_angle_per_vertex.push(verticality_polar_angle);
        depth_azimuth_per_vertex.push(depth_azimuth);
    }
    (verticality_polar_angle_per_vertex, depth_azimuth_per_vertex)
}

fn group_vertices_by_vertical_rings_wrapping_sphere(
    spherical_verticality_polar_angle_per_vertex: &[f32],
) -> Vec<usize> {
    let mut verticality_ring_buckets: Vec<i32> = spherical_verticality_polar_angle_per_vertex
        .iter()
        .map(|&spherical_verticality_polar_angle| {
            (spherical_verticality_polar_angle * SPHERICAL_VERTICALITY_RING_BUCKETS_SCALAR).round() as i32
        })
        .collect();

    let mut ring_id_per_vertex = vec![0usize; spherical_verticality_polar_angle_per_vertex.len()];
    verticality_ring_buckets.sort(); //TODO: what the fuck is this sorting -- required for the stupid binary search bullshit
    for (i, &spherical_verticality_polar_angle) in spherical_verticality_polar_angle_per_vertex.iter().enumerate() {
        let bucket = (spherical_verticality_polar_angle * SPHERICAL_VERTICALITY_RING_BUCKETS_SCALAR).round() as i32;
        let ring_id = verticality_ring_buckets
            .binary_search(&bucket)
            .expect("bucket must exist");
        ring_id_per_vertex[i] = ring_id;
    }
    // TODO: this is somehow now in the same order as the fucking vertices themselves indexed
    ring_id_per_vertex
}

fn compute_closest_vertex_to_observer_per_ring(
    ring_id_per_vertex: &[usize],
    depth_azimuths_per_vertex: &[f32],
) -> Vec<Option<usize>> {
    let max_ring_id = *ring_id_per_vertex.iter().max().unwrap();
    let number_of_rings = max_ring_id + 1; // the rings are indexed by 0, and so another way would be to just somehow get the fucking size of the unique ring ids

    //TODO: this option stuff is to allow for there to at least be SOME (even if its NONE to begin with) closest vertex per ring id
    let mut indicies_of_closest_vertex_to_observer_per_ring: Vec<Option<usize>> = vec![None; number_of_rings];
    for i in 0..ring_id_per_vertex.len() {
        let ring_id = ring_id_per_vertex[i];
        if let Some(current_closest_vertex_for_ring_id_at_i) = indicies_of_closest_vertex_to_observer_per_ring[ring_id]
        {
            if depth_azimuths_per_vertex[i] < depth_azimuths_per_vertex[current_closest_vertex_for_ring_id_at_i] {
                indicies_of_closest_vertex_to_observer_per_ring[ring_id] = Some(i);
            }
        } else {
            indicies_of_closest_vertex_to_observer_per_ring[ring_id] = Some(i);
        }
    }
    //at some ring id the closest vertex is vertex at index blah blah blah
    indicies_of_closest_vertex_to_observer_per_ring
}

fn should_cut_edge(
    shared_edge: &SharedEdge,
    verticality_ring_ids_per_vertex: &[usize],
    closest_vertex_to_observer_per_ring: &[Option<usize>],
    depth_azimuth_per_vertex: &[f32],
    full_meridian_gore_angles: &[f32],
    half_meridian_gore_subdivision: f32,
) -> bool {
    let vertex_index_a = shared_edge.edge.0 as usize;
    let vertex_index_b = shared_edge.edge.1 as usize;
    let avg_depth_azimuth_between_ab = avg_bounded_by_2pi(
        depth_azimuth_per_vertex[vertex_index_a],
        depth_azimuth_per_vertex[vertex_index_b],
    );
    let quantized_avg_depth_azimuth_between_ab = quantize_azimuth(avg_depth_azimuth_between_ab);
    let within_one_half_meridian_gore_subdivision_of_a_full_meridian_gore_angle =
        full_meridian_gore_angles.iter().any(|&gore_angle| {
            distance_bounded_by_2pi(quantized_avg_depth_azimuth_between_ab, quantize_azimuth(gore_angle))
                < half_meridian_gore_subdivision
        });

    let within_the_same_spherical_verticality_ring =
        verticality_ring_ids_per_vertex[vertex_index_a] == verticality_ring_ids_per_vertex[vertex_index_b];
    let either_vertex_a_or_b_are_touching_the_ring_verticality_divide_line = within_the_same_spherical_verticality_ring
        && closest_vertex_to_observer_per_ring[verticality_ring_ids_per_vertex[vertex_index_a]]
            .map(|closest_vertex_for_ring_containing_vertex_a| {
                closest_vertex_for_ring_containing_vertex_a == vertex_index_a
                    || closest_vertex_for_ring_containing_vertex_a == vertex_index_b
            })
            .unwrap_or(false);

    within_one_half_meridian_gore_subdivision_of_a_full_meridian_gore_angle
        || either_vertex_a_or_b_are_touching_the_ring_verticality_divide_line
}

fn find_and_update_parent_face_for_target_face(parent_face_per_target_face: &mut [usize], target_face: usize) -> usize {
    if parent_face_per_target_face[target_face] == target_face {
        //face owns itself
        target_face
    } else {
        let parent_face = find_and_update_parent_face_for_target_face(
            parent_face_per_target_face,
            parent_face_per_target_face[target_face],
        );
        parent_face_per_target_face[target_face] = parent_face;
        parent_face
    }
}

fn compute_triangle_vertices_local_space(vertices: &[f32], indices: &[u16]) -> Vec<[[f32; 2]; 3]> {
    let triangle_count = indices.len() / 3;
    let mut triangle_vertices_local_space: Vec<[[f32; 2]; 3]> = Vec::with_capacity(triangle_count);
    for i in 0..triangle_count {
        let triangle = &indices[i * 3..i * 3 + 3];

        let vertex_a_index = triangle[0] as usize;
        let vertex_b_index = triangle[1] as usize;
        let vertex_c_index = triangle[2] as usize;

        let vertex_a = get_vertex_from_vertex_index(vertices, vertex_a_index);
        let vertex_b = get_vertex_from_vertex_index(vertices, vertex_b_index);
        let vertex_c = get_vertex_from_vertex_index(vertices, vertex_c_index);

        let vector_ac = vertex_c - vertex_a;
        let vector_ab = vertex_b - vertex_a;

        let y_up_normal = vector_ac.normalize_or_zero();
        let x_right_normal = vector_ab.normalize_or_zero();
        let z_depth_normal = x_right_normal.cross(y_up_normal); //?????

        let local_a = [0.0, 0.0];
        let local_b = [vector_ab.length(), 0.0];
        let local_c = [vector_ac.dot(x_right_normal), vector_ac.dot(y_up_normal)];

        triangle_vertices_local_space.push([local_a, local_b, local_c]);
    }
    triangle_vertices_local_space
}

fn convert_child_vertices_from_local_space_to_draw_space(
    parent_face_vertex_indices: &[u16],
    child_face_vertex_indices: &[u16],
    parent_vertices_draw_space: &[[f32; 2]; 3],
    child_vertices_local_space: &[[f32; 2]; 3],
    shared_edge: (u16, u16),
) -> [[f32; 2]; 3] {
    let mut parent_anchor_a = [0.0; 2];
    let mut parent_anchor_b = [0.0; 2];
    let mut child_anchor_a_local_space = [0.0; 2];
    let mut child_anchor_b_local_space = [0.0; 2];
    for vertex_index in 0..3 {
        if parent_face_vertex_indices[vertex_index] == shared_edge.0 {
            parent_anchor_a = parent_vertices_draw_space[vertex_index];
        }
        if parent_face_vertex_indices[vertex_index] == shared_edge.1 {
            parent_anchor_b = parent_vertices_draw_space[vertex_index];
        }
        if child_face_vertex_indices[vertex_index] == shared_edge.0 {
            child_anchor_a_local_space = child_vertices_local_space[vertex_index];
        }
        if child_face_vertex_indices[vertex_index] == shared_edge.1 {
            child_anchor_b_local_space = child_vertices_local_space[vertex_index];
        }
    }
    let child_edge_vector_local_space = [
        child_anchor_b_local_space[0] - child_anchor_a_local_space[0],
        child_anchor_b_local_space[1] - child_anchor_a_local_space[1],
    ];
    let parent_edge_vector_draw_space = [
        parent_anchor_b[0] - parent_anchor_a[0],
        parent_anchor_b[1] - parent_anchor_a[1],
    ];

    let child_edge_length_local_space =
        (child_edge_vector_local_space[0].powi(2) + child_edge_vector_local_space[1].powi(2)).sqrt();
    let parent_edge_length_draw_space =
        (parent_edge_vector_draw_space[0].powi(2) + parent_edge_vector_draw_space[1].powi(2)).sqrt();

    let child_local_edge_direction = [
        child_edge_vector_local_space[0] / child_edge_length_local_space,
        child_edge_vector_local_space[1] / child_edge_length_local_space,
    ];
    let parent_edge_direction = [
        parent_edge_vector_draw_space[0] / parent_edge_length_draw_space,
        parent_edge_vector_draw_space[1] / parent_edge_length_draw_space,
    ];

    let cosine = child_local_edge_direction[0] * parent_edge_direction[0]
        + child_local_edge_direction[1] * parent_edge_direction[1];
    let sine = child_local_edge_direction[0] * parent_edge_direction[1]
        - child_local_edge_direction[1] * parent_edge_direction[0];
    let scale = parent_edge_length_draw_space / child_edge_length_local_space;

    let mut child_vertices_in_draw_space = [[0.0; 2]; 3];
    for vertex_index in 0..3 {
        let local_xy = child_vertices_local_space[vertex_index];
        let local_x = (local_xy[0] - child_anchor_a_local_space[0]) * scale;
        let local_y = (local_xy[1] - child_anchor_a_local_space[1]) * scale;

        let rotated_x = local_x * cosine - local_y * sine;
        let rotated_y = local_x * sine + local_y * cosine;

        child_vertices_in_draw_space[vertex_index] = [parent_anchor_a[0] + rotated_x, parent_anchor_a[1] + rotated_y];
    }
    child_vertices_in_draw_space
}

pub fn traverse_edge_path_child_to_parent(
    start_face: usize,
    is_parent_face: &Vec<bool>,
    edge_path_child_to_parent: &Vec<Option<(usize, (u16, u16))>>,
) -> usize {
    let mut face = start_face;
    while !is_parent_face[face] {
        face = edge_path_child_to_parent[face].unwrap().0;
    }
    face
}

#[inline]
fn get_vertex_from_vertex_index(vertices: &[f32], vertex_index: usize) -> Vec3 {
    Vec3::new(
        vertices[vertex_index * 3 + 0],
        vertices[vertex_index * 3 + 1],
        vertices[vertex_index * 3 + 2],
    )
}

#[inline]
fn wrap_2pi(mut angle_in_radians: f32) -> f32 {
    if angle_in_radians < 0.0 {
        angle_in_radians += TWO_PI;
    }
    if angle_in_radians >= TWO_PI {
        angle_in_radians -= TWO_PI;
    }
    angle_in_radians
}

#[inline]
fn avg_bounded_by_2pi(angle_a: f32, angle_b: f32) -> f32 {
    let distance_apart_on_circle = (angle_a - angle_b).abs();
    //NOTE: if the angles are at most a half circle appart (the MAX distance on the sphere), then wrap there avg over 2PI
    if distance_apart_on_circle <= PI {
        return wrap_2pi((angle_a + angle_b) * 0.5);
    }
    if angle_a > angle_b {
        wrap_2pi(((angle_a - TWO_PI) + angle_b) * 0.5)
    } else {
        wrap_2pi((angle_a + (angle_b - TWO_PI)) * 0.5)
    }
}

#[inline]
fn distance_bounded_by_2pi(a: f32, b: f32) -> f32 {
    let mut distance = (a - b).abs();
    if distance > PI {
        distance = TWO_PI - distance;
    }
    distance
}

fn face_normal(vertices: &[f32], triangle: &[u16]) -> Vec3 {
    let vertex_a = get_vertex_from_vertex_index(vertices, triangle[0] as usize);
    let vertex_b = get_vertex_from_vertex_index(vertices, triangle[1] as usize);
    let vertex_c = get_vertex_from_vertex_index(vertices, triangle[2] as usize);
    (vertex_b - vertex_a).cross(vertex_c - vertex_a).normalize_or_zero()
}

pub fn quantize_azimuth(azimuth: f32) -> f32 {
    (azimuth * AZIMUTH_QUANTIZATION_SCALAR).round() / AZIMUTH_QUANTIZATION_SCALAR
}
