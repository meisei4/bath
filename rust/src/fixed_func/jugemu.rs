use crate::fixed_func::topology::{observed_line_of_sight, rotate_vertices_in_plane, Topology};
use raylib::camera::Camera3D;
use raylib::color::Color;
use raylib::drawing::RaylibDraw3D;
use raylib::ffi::{rlSetLineWidth, rlSetPointSize};
use raylib::math::{Vector2, Vector3};
use raylib::models::{Model, RaylibMesh, RaylibModel, WeakMesh};
use std::ffi::c_int;

pub fn draw_observed_axes(rl3d: &mut impl RaylibDraw3D, observer: &Camera3D) {
    let (observed_line_of_sight, observed_right, observed_up) = observed_orthonormal_basis_vectors(observer);
    unsafe { rlSetLineWidth(1.5) };
    rl3d.draw_line3D(observer.position, observer.position + observed_right, Color::PURPLE);
    rl3d.draw_line3D(observer.position, observer.position + observed_up, Color::DARKSEAGREEN);
    rl3d.draw_line3D(
        observer.position,
        observer.position + observed_line_of_sight,
        Color::DEEPSKYBLUE,
    );
}

pub fn map_frustum_to_ndc_cube(
    rl3d: &mut impl RaylibDraw3D,
    screen_w: i32,
    screen_h: i32,
    observer: &Camera3D,
    near_clip_plane: f32,
    far_clip_plane: f32,
    src_model: &Model,
    ndc_model: &mut Model,
    model_pos: Vector3,
    model_scale: Vector3,
    mesh_rotation: f32,
) {
    let aspect = screen_w as f32 / screen_h as f32;
    let observed_line_of_sight = observed_line_of_sight(observer);
    let center_near_clip_plane = observer.position + observed_line_of_sight * near_clip_plane;
    let half_fovy = observer.fovy.to_radians() * 0.5;
    let half_height_near_clip_plane = near_clip_plane * half_fovy.tan();

    ///ANISOTROPIC VIEW
    // let half_width_near_clip_plane = half_height_near_clip_plane * aspect;
    // let half_depth_ndc_cube = 0.5 * (far_clip_plane - near_clip_plane);
    // let ndc_cube_center = center_near_clip_plane + observed_line_of_sight * half_depth_ndc_cube;

    ///ISOTROPIC DIDACTIC VIEW
    let half_width_near_clip_plane = half_height_near_clip_plane;
    let half_depth_ndc_cube = half_height_near_clip_plane;
    let ndc_cube_center = center_near_clip_plane + observed_line_of_sight * half_depth_ndc_cube;
    draw_ndc_cube(
        rl3d,
        observer,
        ndc_cube_center,
        half_width_near_clip_plane,
        half_height_near_clip_plane,
        half_depth_ndc_cube,
        Color::SKYBLUE,
    );
    update_world_to_ndc_mapped_mesh(
        &mut ndc_model.meshes_mut()[0],
        &src_model.meshes()[0],
        screen_w,
        screen_h,
        observer,
        near_clip_plane,
        far_clip_plane,
        model_pos,
        model_scale,
        mesh_rotation,
    );
}

pub fn update_world_to_ndc_mapped_mesh(
    ndc_mesh: &mut WeakMesh,
    world_mesh: &WeakMesh,
    screen_w: i32,
    screen_h: i32,
    observer: &Camera3D,
    near_clip_plane: f32,
    far_clip_plane: f32,
    model_pos: Vector3,
    model_scale: Vector3,
    mesh_rotation: f32,
) {
    let aspect = screen_w as f32 / screen_h as f32;
    let observed_line_of_sight = observed_line_of_sight(observer);

    let center_near_clip_plane = observer.position + observed_line_of_sight * near_clip_plane;
    let half_fovy = observer.fovy.to_radians() * 0.5;
    let half_height_near_clip_plane = near_clip_plane * half_fovy.tan();

    ///ANISOTROPIC VIEW
    // let half_width_near_clip_plane = half_height_near_clip_plane * aspect;
    // let half_depth_ndc_cube = 0.5 * (far_clip_plane - near_clip_plane);
    // let ndc_cube_center = center_near_clip_plane + observed_line_of_sight * half_depth_ndc_cube;

    ///ISOTROPIC DIDACTIC VIEW
    let half_width_near_clip_plane = half_height_near_clip_plane;
    let half_depth_ndc_cube = half_height_near_clip_plane;
    let ndc_cube_center = center_near_clip_plane + observed_line_of_sight * half_depth_ndc_cube;

    let world_coords = world_mesh.vertices();
    let ndc_coords = ndc_mesh.vertices_mut();
    let ndc_vertex_count = world_coords.len().min(ndc_coords.len());
    for i in 0..ndc_vertex_count {
        // let world_coord = world_coords[i].clone();
        let world_coord = apply_model_translate_rotate_scale(world_coords[i], model_pos, model_scale, mesh_rotation);
        let ndc_coord = world_coord_to_ndc_coord(aspect, observer, near_clip_plane, far_clip_plane, world_coord);
        let scaled_ndc_coord = scale_ndc_coord_by_near_clip_plane(
            observer,
            ndc_cube_center,
            half_width_near_clip_plane,
            half_height_near_clip_plane,
            half_depth_ndc_cube,
            ndc_coord,
        );
        // ndc_coords[i] = scaled_ndc_coord;
        ndc_coords[i] =
            apply_inverse_model_translate_rotate_scale(scaled_ndc_coord, model_pos, model_scale, mesh_rotation);
    }
    ndc_mesh.vertexCount = ndc_vertex_count as c_int;
}

#[inline]
pub fn world_coord_to_ndc_coord(
    aspect: f32,
    observer: &Camera3D,
    near_clip_plane: f32,
    far_clip_plane: f32,
    world_coord: Vector3,
) -> (f32, f32, f32) {
    let (observed_line_of_sight, observed_right, observed_up) = observed_orthonormal_basis_vectors(observer);
    let half_fovy = observer.fovy.to_radians() * 0.5;
    //TODO: everywhere that these calculations are made we need to make it consistent for the didactic visuals
    let half_height_near_clip_plane = near_clip_plane * half_fovy.tan();

    /// ANISOTROPIC
    // let half_width_near_clip_plane = half_height_near_clip_plane * aspect;

    /// ISOTROPIC DIDACTIC
    let half_width_near_clip_plane = half_height_near_clip_plane;

    let ray_from_world_to_observer = world_coord - observer.position;
    let signed_depth_component = ray_from_world_to_observer.dot(observed_line_of_sight); // depth here is now POSITIVE/FORWARD from camera (no longer world context of -z los)
    let center_near_clip_plane = observer.position + observed_line_of_sight * near_clip_plane;
    let intersection_coord = near_plane_intersection(observer, near_clip_plane, world_coord);
    let clip_plane_vector = intersection_coord - center_near_clip_plane;
    let x_ndc = clip_plane_vector.dot(observed_right) / half_width_near_clip_plane;
    let y_ndc = clip_plane_vector.dot(observed_up) / half_height_near_clip_plane;

    /// OTHOGRAPHIC Z?
    // let z_ndc_linear_orthographic = 2.0 * (signed_depth_component - near_clip_plane) / (far_clip_plane - near_clip_plane) - 1.0;
    // (x_ndc, y_ndc, z_ndc_linear_orthographic)

    ///PERSPECTIVE CORRECT Z?
    let z_ndc_perspective_correct = ((far_clip_plane + near_clip_plane)
        - (2.0 * far_clip_plane * near_clip_plane) / signed_depth_component)
        / (far_clip_plane - near_clip_plane);
    (x_ndc, y_ndc, z_ndc_perspective_correct)
}

#[inline]
fn scale_ndc_coord_by_near_clip_plane(
    observer: &Camera3D,
    center_near_clip_plane: Vector3,
    half_width_near_clip_plane: f32,
    half_height_near_clip_plane: f32,
    half_depth_ndc_cube: f32,
    (x, y, z): (f32, f32, f32),
) -> Vector3 {
    let (observed_line_of_sight, observed_right, observed_up) = observed_orthonormal_basis_vectors(observer);
    center_near_clip_plane
        + observed_right * (x * half_width_near_clip_plane)
        + observed_up * (y * half_height_near_clip_plane)
        + observed_line_of_sight * (z * half_depth_ndc_cube)
}

#[inline]
fn apply_model_translate_rotate_scale(
    model_coord: Vector3,
    model_pos: Vector3,
    model_scale: Vector3,
    mesh_rotation: f32,
) -> Vector3 {
    let scaled_x = model_coord.x * model_scale.x;
    let scaled_y = model_coord.y * model_scale.y;
    let scaled_z = model_coord.z * model_scale.z;
    let (sine, cosine) = mesh_rotation.sin_cos();
    let rotated_x = cosine * scaled_x + sine * scaled_z;
    let rotated_z = -sine * scaled_x + cosine * scaled_z;
    Vector3 {
        x: rotated_x + model_pos.x,
        y: scaled_y + model_pos.y,
        z: rotated_z + model_pos.z,
    }
}

#[inline]
fn apply_inverse_model_translate_rotate_scale(
    world_coord: Vector3,
    world_pos: Vector3,
    world_scale: Vector3,
    mesh_rotation: f32,
) -> Vector3 {
    let translated_x = world_coord.x - world_pos.x;
    let translated_y = world_coord.y - world_pos.y;
    let translated_z = world_coord.z - world_pos.z;
    let (sine, cosine) = (-mesh_rotation).sin_cos();
    let rotated_x = cosine * translated_x + sine * translated_z;
    let rotated_z = -sine * translated_x + cosine * translated_z;
    Vector3 {
        x: rotated_x / world_scale.x,
        y: translated_y / world_scale.y,
        z: rotated_z / world_scale.z,
    }
}

fn draw_ndc_cube(
    rl3d: &mut impl RaylibDraw3D,
    observer: &Camera3D,
    center_near_clip_plane: Vector3,
    half_width_near_clip_plane: f32,
    half_height_near_clip_plane: f32,
    half_depth_ndc_cube: f32,
    edge_color: Color,
) {
    let (observed_line_of_sight, observed_right, observed_up) = observed_orthonormal_basis_vectors(observer);
    let mut ndc_cube_corners = [Vector3::ZERO; 8];
    let mut i = 0;
    for &z_ndc in &[-1.0, 1.0] {
        for &y_ndc in &[-1.0, 1.0] {
            for &x_ndc in &[-1.0, 1.0] {
                ndc_cube_corners[i] = center_near_clip_plane
                    + observed_right * (x_ndc * half_width_near_clip_plane)
                    + observed_up * (y_ndc * half_height_near_clip_plane)
                    + observed_line_of_sight * (z_ndc * half_depth_ndc_cube);
                i += 1;
            }
        }
    }
    let near_top_left = ndc_cube_corners[3];
    let near_top_right = ndc_cube_corners[2];
    let near_bottom_right = ndc_cube_corners[0];
    let near_bottom_left = ndc_cube_corners[1];
    let far_top_left = ndc_cube_corners[7];
    let far_top_right = ndc_cube_corners[6];
    let far_bottom_right = ndc_cube_corners[4];
    let far_bottom_left = ndc_cube_corners[5];

    unsafe { rlSetLineWidth(1.0) };
    rl3d.draw_line3D(near_top_left, near_top_right, edge_color);
    rl3d.draw_line3D(near_top_right, near_bottom_right, edge_color);
    rl3d.draw_line3D(near_bottom_right, near_bottom_left, edge_color);
    rl3d.draw_line3D(near_bottom_left, near_top_left, edge_color);
    rl3d.draw_line3D(far_top_left, far_top_right, edge_color);
    rl3d.draw_line3D(far_top_right, far_bottom_right, edge_color);
    rl3d.draw_line3D(far_bottom_right, far_bottom_left, edge_color);
    rl3d.draw_line3D(far_bottom_left, far_top_left, edge_color);
    rl3d.draw_line3D(near_top_left, far_top_left, edge_color);
    rl3d.draw_line3D(near_top_right, far_top_right, edge_color);
    rl3d.draw_line3D(near_bottom_right, far_bottom_right, edge_color);
    rl3d.draw_line3D(near_bottom_left, far_bottom_left, edge_color);
}

pub fn draw_frustum(
    rl3d: &mut impl RaylibDraw3D,
    aspect: f32,
    observer: &Camera3D,
    near_clip_plane: f32,
    far_clip_plane: f32,
) {
    let (observed_line_of_sight, observed_right, observed_up) = observed_orthonormal_basis_vectors(observer);

    let center_near_clip_plane = observer.position + observed_line_of_sight * near_clip_plane;
    let center_far_clip_plane = observer.position + observed_line_of_sight * far_clip_plane;

    let half_fovy = observer.fovy.to_radians() * 0.5;
    let half_height_near_clip_plane = near_clip_plane * half_fovy.tan();
    let half_width_near_clip_plane = half_height_near_clip_plane * aspect;

    let far_half_height = far_clip_plane * half_fovy.tan();
    let far_half_width = far_half_height * aspect;

    let near_top_left = center_near_clip_plane + observed_up * half_height_near_clip_plane
        - observed_right * half_width_near_clip_plane;
    let near_top_right = center_near_clip_plane
        + observed_up * half_height_near_clip_plane
        + observed_right * half_width_near_clip_plane;
    let near_bottom_right = center_near_clip_plane - observed_up * half_height_near_clip_plane
        + observed_right * half_width_near_clip_plane;
    let near_bottom_left = center_near_clip_plane
        - observed_up * half_height_near_clip_plane
        - observed_right * half_width_near_clip_plane;

    let far_top_left = center_far_clip_plane + observed_up * far_half_height - observed_right * far_half_width;
    let far_top_right = center_far_clip_plane + observed_up * far_half_height + observed_right * far_half_width;
    let far_bottom_right = center_far_clip_plane - observed_up * far_half_height + observed_right * far_half_width;
    let far_bottom_left = center_far_clip_plane - observed_up * far_half_height - observed_right * far_half_width;
    unsafe { rlSetLineWidth(1.0) };
    rl3d.draw_line3D(near_top_left, near_top_right, Color::SKYBLUE);
    rl3d.draw_line3D(near_top_right, near_bottom_right, Color::SKYBLUE);
    rl3d.draw_line3D(near_bottom_right, near_bottom_left, Color::SKYBLUE);
    rl3d.draw_line3D(near_bottom_left, near_top_left, Color::SKYBLUE);

    rl3d.draw_line3D(far_top_left, far_top_right, Color::GRAY);
    rl3d.draw_line3D(far_top_right, far_bottom_right, Color::GRAY);
    rl3d.draw_line3D(far_bottom_right, far_bottom_left, Color::GRAY);
    rl3d.draw_line3D(far_bottom_left, far_top_left, Color::GRAY);

    rl3d.draw_line3D(near_top_left, far_top_left, Color::DARKBLUE);
    rl3d.draw_line3D(near_top_right, far_top_right, Color::DARKBLUE);
    rl3d.draw_line3D(near_bottom_right, far_bottom_right, Color::DARKBLUE);
    rl3d.draw_line3D(near_bottom_left, far_bottom_left, Color::DARKBLUE);

    let near_plane_color = Color {
        a: 100,
        ..Color::SKYBLUE
    };
    let far_plane_color = Color { a: 200, ..Color::GRAY };
    draw_quad(
        rl3d,
        near_top_left,
        near_top_right,
        near_bottom_right,
        near_bottom_left,
        near_plane_color,
    );
    draw_quad(
        rl3d,
        far_top_left,
        far_top_right,
        far_bottom_right,
        far_bottom_left,
        far_plane_color,
    );
}

pub fn draw_quad(rl3d: &mut impl RaylibDraw3D, a: Vector3, b: Vector3, c: Vector3, d: Vector3, color: Color) {
    rl3d.draw_triangle3D(a, b, c, color);
    rl3d.draw_triangle3D(a, c, d, color);
}

pub fn draw_near_plane_intersectional_disk_mesh(
    rl3d: &mut impl RaylibDraw3D,
    observer: &Camera3D,
    near_clip_plane: f32,
    intersection_model: &mut Model,
    model_position: Vector3,
    model_scale: Vector3,
    mesh_rotation: f32,
    topology: &Topology,
    reflect_y: bool,
) {
    let triangle_set = topology.front_triangles_snapshot.as_ref().unwrap();
    // let front_triangles = topology.front_triangles_snapshot.as_ref().unwrap();
    // let silhouette_triangles = topology.silhouette_triangles_snapshot.clone().unwrap();
    // let triangle_set: HashSet<usize> = front_triangles.difference(&silhouette_triangles).copied().collect();

    let vertex_count = &triangle_set.len() * 3;
    let vertices_per_triangle = topology.vertices_per_triangle_snapshot.as_ref().unwrap();

    let mut intersection_coordinates = Vec::new();
    //TODO: when just iterating over a reference to the hashset of the front triangles (no difference stuff with silhouettes) you need to copy?
    for triangle_id in triangle_set.iter().copied() {
        let mut vertices = vertices_per_triangle[triangle_id];
        rotate_vertices_in_plane(&mut vertices, mesh_rotation);

        for vertex in vertices.iter() {
            let scaled = vertex * model_scale;
            let world_coord = model_position + scaled;

            let intersection_coord = near_plane_intersection(observer, near_clip_plane, world_coord);
            let intersection_coord = if reflect_y {
                flip_y_in_near_clip_plane(intersection_coord, observer, near_clip_plane)
            } else {
                intersection_coord
            };

            unsafe { rlSetLineWidth(1.0) };
            rl3d.draw_line3D(
                world_coord,
                intersection_coord,
                Color {
                    a: 80,
                    ..Color::WHITESMOKE
                },
            );
            intersection_coordinates.push(intersection_coord);
        }
    }

    if !intersection_coordinates.is_empty() {
        replace_mesh_vertices(
            &mut intersection_model.meshes_mut()[0],
            &intersection_coordinates,
            vertex_count,
        );
    }
    unsafe { rlSetPointSize(3.0) };
    rl3d.draw_model_points(&intersection_model, Vector3::ZERO, 1.0, Color::GREEN);
    // rl3d.draw_model_wires(&intersection_model, Vector3::ZERO, 1.0, Color::WHITE);
}

#[inline]
fn flip_y_in_near_clip_plane(intersection_coord: Vector3, observer: &Camera3D, near_clip_plane: f32) -> Vector3 {
    let (observed_line_of_sight, observed_right, observed_up) = observed_orthonormal_basis_vectors(observer);
    let center_near_clip_plane = observer.position + observed_line_of_sight * near_clip_plane;

    let intersection_coord_to_clip_plane_origin = intersection_coord - center_near_clip_plane;
    let intersection_coord_x_component = intersection_coord_to_clip_plane_origin.dot(observed_right);
    let intersection_coord_y_component = intersection_coord_to_clip_plane_origin.dot(observed_up);
    center_near_clip_plane + observed_right * intersection_coord_x_component
        - observed_up * intersection_coord_y_component
}

pub fn near_plane_intersection(observer: &Camera3D, near_clip_plane: f32, world_coord: Vector3) -> Vector3 {
    let observed_line_of_sight = observed_line_of_sight(observer);
    let ray_from_world_to_observer = world_coord - observer.position;
    let signed_depth_component = ray_from_world_to_observer.dot(observed_line_of_sight); // depth here is now POSITIVE/FORWARD from camera (no longer world context of -z los)
    let depth_interpolation = near_clip_plane / signed_depth_component;
    observer.position + ray_from_world_to_observer * depth_interpolation
}

#[inline]
pub fn observed_orthonormal_basis_vectors(observer: &Camera3D) -> (Vector3, Vector3, Vector3) {
    let observed_line_of_sight = observed_line_of_sight(observer).normalize_or_zero();
    let observed_right = observed_line_of_sight.cross(observer.up).normalize_or_zero();
    let observed_up = observed_right.cross(observed_line_of_sight).normalize_or_zero();
    (observed_line_of_sight, observed_right, observed_up)
}

pub fn apply_barycentric_palette(mesh: &mut WeakMesh) {
    let triangles: Vec<[usize; 3]> = mesh.triangles().collect();
    //TODO: I do not like this madness
    let colors = mesh.ensure_colors().unwrap();
    for [a, b, c] in &triangles {
        colors[*a] = Color::RED;
        colors[*b] = Color::GREEN;
        colors[*c] = Color::BLUE;
    }
    let texcoords = mesh.ensure_texcoords().unwrap();
    for [a, b, c] in &triangles {
        texcoords[*a] = Vector2::new(1.0, 0.0);
        texcoords[*b] = Vector2::new(0.0, 1.0);
        texcoords[*c] = Vector2::new(0.0, 0.0);
    }
}

//TODO: make this a more common update mesh function somehow? to avoid having to do load and unloads of model/meshes constantly in opengl1.1
pub fn replace_mesh_vertices(dst_mesh: &mut WeakMesh, src_vertices: &[Vector3], capacity: usize) {
    if src_vertices.is_empty() {
        return;
    }
    let dst_vertex_count = src_vertices.len().min(capacity);
    dst_mesh.vertexCount = dst_vertex_count as c_int;
    let dst_vertices = dst_mesh.vertices_mut();
    for (i, src_vertex) in src_vertices.iter().take(dst_vertex_count).enumerate() {
        dst_vertices[i] = *src_vertex;
    }
}
