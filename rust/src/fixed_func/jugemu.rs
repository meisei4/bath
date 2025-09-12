use crate::fixed_func::topology::{observed_line_of_sight, rotate_vertices_in_plane, Topology};
use raylib::camera::Camera3D;
use raylib::color::Color;
use raylib::drawing::RaylibDraw3D;
use raylib::ffi::{rlSetLineWidth, rlSetPointSize};
use raylib::math::{Vector2, Vector3};
use raylib::models::{Model, RaylibMesh, RaylibModel, WeakMesh};
use std::collections::HashSet;
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

pub fn draw_frustum(
    rl3d: &mut impl RaylibDraw3D,
    observer: &Camera3D,
    aspect: f32,
    near_clip_plane: f32,
    far_clip_plane: f32,
) {
    let frustum_corners = compute_frustum_corners(observer, aspect, near_clip_plane, far_clip_plane);
    let near_top_left = frustum_corners[0];
    let near_top_right = frustum_corners[1];
    let near_bottom_right = frustum_corners[2];
    let near_bottom_left = frustum_corners[3];

    let far_top_left = frustum_corners[4];
    let far_top_right = frustum_corners[5];
    let far_bottom_right = frustum_corners[6];
    let far_bottom_left = frustum_corners[7];
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

pub fn compute_frustum_corners(
    observer: &Camera3D,
    aspect: f32,
    near_clip_plane: f32,
    far_clip_plane: f32,
) -> [Vector3; 8] {
    let (observed_line_of_sight, observed_right, observed_up) = observed_orthonormal_basis_vectors(observer);

    let center_near_clip_plane = observer.position + observed_line_of_sight * near_clip_plane;
    let far_clip_plane_center = observer.position + observed_line_of_sight * far_clip_plane;

    let half_fovy = observer.fovy.to_radians() * 0.5;
    let half_height_near = near_clip_plane * half_fovy.tan();
    let half_width_near = half_height_near * aspect;

    let far_half_height = far_clip_plane * half_fovy.tan();
    let far_half_width = far_half_height * aspect;

    let near_top_left = center_near_clip_plane + observed_up * half_height_near - observed_right * half_width_near;
    let near_top_right = center_near_clip_plane + observed_up * half_height_near + observed_right * half_width_near;
    let near_bottom_right = center_near_clip_plane - observed_up * half_height_near + observed_right * half_width_near;
    let near_bottom_left = center_near_clip_plane - observed_up * half_height_near - observed_right * half_width_near;

    let far_top_left = far_clip_plane_center + observed_up * far_half_height - observed_right * far_half_width;
    let far_top_right = far_clip_plane_center + observed_up * far_half_height + observed_right * far_half_width;
    let far_bottom_right = far_clip_plane_center - observed_up * far_half_height + observed_right * far_half_width;
    let far_bottom_left = far_clip_plane_center - observed_up * far_half_height - observed_right * far_half_width;

    [
        near_top_left,
        near_top_right,
        near_bottom_right,
        near_bottom_left,
        far_top_left,
        far_top_right,
        far_bottom_right,
        far_bottom_left,
    ]
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
    mesh_rotation_radians: f32,
    topology: &Topology,
) {
    let front_triangles = topology.front_triangles_snapshot.as_ref().unwrap();
    let silhouette_triangles = topology.silhouette_triangles_snapshot.clone().unwrap();
    let triangle_set: HashSet<usize> = front_triangles.difference(&silhouette_triangles).copied().collect();
    let vertices_per_triangle = topology.vertices_per_triangle_snapshot.as_ref().unwrap();

    let mut intersection_coordinates = Vec::new();

    for triangle_id in triangle_set {
        let mut vertices = vertices_per_triangle[triangle_id];
        rotate_vertices_in_plane(&mut vertices, mesh_rotation_radians);

        for vertex in vertices.iter() {
            let scaled = vertex * model_scale;
            let world_coord = model_position + scaled;

            let intersection_coord = near_plane_intersection(observer, near_clip_plane, world_coord);
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
    replace_mesh_vertices(&mut intersection_model.meshes_mut()[0], &intersection_coordinates);
    unsafe { rlSetPointSize(3.0) };
    rl3d.draw_model_points(&intersection_model, Vector3::ZERO, 1.0, Color::GOLD);
    // rl3d.draw_model_wires(&intersection_model, Vector3::ZERO, 1.0, Color::WHITE);
}

pub fn draw_near_plane_software_raster(
    rl3d: &mut impl RaylibDraw3D,
    observer: &Camera3D,
    aspect: f32,
    near_clip_plane: f32,
    screen_w: i32,
    screen_h: i32,
    mesh: &WeakMesh,
    model_position: Vector3,
    model_scale: Vector3,
    mesh_rotation_radians: f32,
    step: i32,
) {
    let (observed_line_of_sight, observed_right, observed_up) = observed_orthonormal_basis_vectors(observer);
    let center_near_clip_plane = observer.position + observed_line_of_sight * near_clip_plane;
    let half_fovy = observer.fovy.to_radians() * 0.5;
    let half_height_near_clip_plane = near_clip_plane * half_fovy.tan();
    let half_width_near_clip_plane = half_height_near_clip_plane * aspect;
    let half_dx_world = observed_right * (half_width_near_clip_plane / screen_w as f32);
    let half_dy_world = observed_up * (half_height_near_clip_plane / screen_h as f32);
    let depth_buffer_len = (screen_w * screen_h) as usize;
    let mut depth_buffer = vec![f32::INFINITY; depth_buffer_len];
    //TODO: why is this Vec instead of something simpler lor like clearer?
    let triangles: Vec<[usize; 3]> = mesh.triangles().collect();
    let colors = mesh.colors().unwrap(); //TODO: maybe ensure idk
    let vertices = mesh.vertices();
    for [vertex_a, vertex_b, vertex_c] in triangles {
        let mut triangle = [vertices[vertex_a], vertices[vertex_b], vertices[vertex_c]];
        rotate_vertices_in_plane(&mut triangle, mesh_rotation_radians);
        let world_triangle = triangle.map(|vertex| model_position + vertex * model_scale);
        let projection_a = project_stage(observer, aspect, near_clip_plane, screen_w, screen_h, world_triangle[0]);
        let projection_b = project_stage(observer, aspect, near_clip_plane, screen_w, screen_h, world_triangle[1]);
        let projection_c = project_stage(observer, aspect, near_clip_plane, screen_w, screen_h, world_triangle[2]);
        rasterize_triangle(
            rl3d,
            [projection_a, projection_b, projection_c],
            [vertex_a, vertex_b, vertex_c],
            colors,
            screen_w,
            screen_h,
            step,
            &mut depth_buffer,
            center_near_clip_plane,
            observed_right,
            observed_up,
            half_width_near_clip_plane,
            half_height_near_clip_plane,
            half_dx_world,
            half_dy_world,
        );
    }
}

pub fn project_stage(
    observer: &Camera3D,
    aspect: f32,
    near_clip_plane: f32,
    screen_w: i32,
    screen_h: i32,
    world_coord: Vector3,
) -> (f32, f32, f32) {
    let (observed_line_of_sight, observed_right, observed_up) = observed_orthonormal_basis_vectors(observer);
    let center_near_clip_plane = observer.position + observed_line_of_sight * near_clip_plane;
    let half_height_near_clip_plane = near_clip_plane * (observer.fovy.to_radians() * 0.5).tan();
    let half_width_near_clip_plane = half_height_near_clip_plane * aspect;
    let ray_from_world_to_observer = world_coord - observer.position;
    let depth_z = ray_from_world_to_observer.dot(observed_line_of_sight);
    let intersection_point = near_plane_intersection(observer, near_clip_plane, world_coord);
    let clip_plane_vector = intersection_point - center_near_clip_plane;
    let ndc_x = clip_plane_vector.dot(observed_right) / half_width_near_clip_plane;
    let ndc_y = clip_plane_vector.dot(observed_up) / half_height_near_clip_plane;
    let pixel_x = (ndc_x * 0.5 + 0.5) * screen_w as f32;
    let pixel_y = (1.0 - (ndc_y * 0.5 + 0.5)) * screen_h as f32;
    (pixel_x, pixel_y, depth_z)
}

fn rasterize_triangle(
    rl3d: &mut impl RaylibDraw3D,
    projected_triangle: [(f32, f32, f32); 3],
    vertex_indices: [usize; 3],
    colors: &[Color],
    screen_w: i32,
    screen_h: i32,
    step: i32,
    depth_buffer: &mut [f32],
    center_near_clip_plane: Vector3,
    observed_right: Vector3,
    observed_up: Vector3,
    half_width_near_clip_plane: f32,
    half_height_near_clip_plane: f32,
    half_dx_world: Vector3,
    half_dy_world: Vector3,
) {
    let (pixel_x_a, pixel_y_a, depth_z_a) = projected_triangle[0];
    let (pixel_x_b, pixel_y_b, depth_z_b) = projected_triangle[1];
    let (pixel_x_c, pixel_y_c, depth_z_c) = projected_triangle[2];

    let pixel_a = Vector2::new(pixel_x_a, pixel_y_a);
    let pixel_b = Vector2::new(pixel_x_b, pixel_y_b);
    let pixel_c = Vector2::new(pixel_x_c, pixel_y_c);
    let triangle_area = barymetric_vomit(pixel_a, pixel_b, pixel_c);

    let mut min_pixel_x = pixel_x_a.min(pixel_x_b).min(pixel_x_c).floor() as i32;
    let mut max_pixel_x = pixel_x_a.max(pixel_x_b).max(pixel_x_c).ceil() as i32;
    let mut min_pixel_y = pixel_y_a.min(pixel_y_b).min(pixel_y_c).floor() as i32;
    let mut max_pixel_y = pixel_y_a.max(pixel_y_b).max(pixel_y_c).ceil() as i32;

    min_pixel_x = min_pixel_x.clamp(0, screen_w - 1);
    max_pixel_x = max_pixel_x.clamp(0, screen_w - 1);
    min_pixel_y = min_pixel_y.clamp(0, screen_h - 1);
    max_pixel_y = max_pixel_y.clamp(0, screen_h - 1);

    let depth_divide_a = 1.0 / depth_z_a;
    let depth_divide_b = 1.0 / depth_z_b;
    let depth_divide_c = 1.0 / depth_z_c;

    let start_x = (min_pixel_x / step) * step;
    let start_y = (min_pixel_y / step) * step;

    let mut raster_pixel_y = start_y;
    while raster_pixel_y <= max_pixel_y {
        let sample_center_y = raster_pixel_y as f32 + 0.5;
        let mut raster_pixel_x = start_x;
        while raster_pixel_x <= max_pixel_x {
            let sample_center_x = raster_pixel_x as f32 + 0.5;
            let sample_pixel = Vector2::new(sample_center_x, sample_center_y);

            let edge_value_ab_c = barymetric_vomit(pixel_b, pixel_c, sample_pixel);
            let edge_value_bc_a = barymetric_vomit(pixel_c, pixel_a, sample_pixel);
            let edge_value_ca_b = barymetric_vomit(pixel_a, pixel_b, sample_pixel);

            if (edge_value_ab_c * triangle_area >= 0.0)
                && (edge_value_bc_a * triangle_area >= 0.0)
                && (edge_value_ca_b * triangle_area >= 0.0)
            {
                let barycentric_w_a = edge_value_ab_c / triangle_area;
                let barycentric_w_b = edge_value_bc_a / triangle_area;
                let barycentric_w_c = edge_value_ca_b / triangle_area;

                let depth = barycentric_w_a * depth_divide_a
                    + barycentric_w_b * depth_divide_b
                    + barycentric_w_c * depth_divide_c;
                let depth_camera_space = 1.0 / depth;

                let buffer_index = (raster_pixel_y as usize) * (screen_w as usize) + (raster_pixel_x as usize);

                if depth_camera_space < depth_buffer[buffer_index] {
                    depth_buffer[buffer_index] = depth_camera_space;

                    let [vertex_a, vertex_b, vertex_c] = vertex_indices;
                    let red = barycentric_w_a * (colors[vertex_a].r as f32)
                        + barycentric_w_b * (colors[vertex_b].r as f32)
                        + barycentric_w_c * (colors[vertex_c].r as f32);
                    let green = barycentric_w_a * (colors[vertex_a].g as f32)
                        + barycentric_w_b * (colors[vertex_b].g as f32)
                        + barycentric_w_c * (colors[vertex_c].g as f32);
                    let blue = barycentric_w_a * (colors[vertex_a].b as f32)
                        + barycentric_w_b * (colors[vertex_b].b as f32)
                        + barycentric_w_c * (colors[vertex_c].b as f32);

                    let raster_color = Color {
                        r: red.clamp(0.0, 255.0) as u8,
                        g: green.clamp(0.0, 255.0) as u8,
                        b: blue.clamp(0.0, 255.0) as u8,
                        a: 255,
                    };

                    let x_ndc = (sample_pixel.x / screen_w as f32) * 2.0 - 1.0;
                    let y_ndc = 1.0 - (sample_pixel.y / screen_h as f32) * 2.0;

                    let world_pixel_center_on_near_plane = center_near_clip_plane
                        + observed_right * (x_ndc * half_width_near_clip_plane)
                        + observed_up * (y_ndc * half_height_near_clip_plane);

                    let quad_raster_pixel_a = world_pixel_center_on_near_plane - half_dx_world - half_dy_world;
                    let quad_raster_pixel_b = world_pixel_center_on_near_plane + half_dx_world - half_dy_world;
                    let quad_raster_pixel_c = world_pixel_center_on_near_plane + half_dx_world + half_dy_world;
                    let quad_raster_pixel_d = world_pixel_center_on_near_plane - half_dx_world + half_dy_world;

                    rl3d.draw_triangle3D(
                        quad_raster_pixel_a,
                        quad_raster_pixel_b,
                        quad_raster_pixel_c,
                        raster_color,
                    );
                    rl3d.draw_triangle3D(
                        quad_raster_pixel_a,
                        quad_raster_pixel_c,
                        quad_raster_pixel_d,
                        raster_color,
                    );
                }
            }
            raster_pixel_x += step;
        }
        raster_pixel_y += step;
    }
}

pub fn near_plane_intersection(observer: &Camera3D, near_clip_plane: f32, world_coord: Vector3) -> Vector3 {
    let observed_line_of_sight = observed_line_of_sight(observer);
    let vector_from_camera_to_world_coord = world_coord - observer.position;
    //TODO: I need to rename this as another example of the dot product's importance... i forget it already jesus
    let forward_component = vector_from_camera_to_world_coord.dot(observed_line_of_sight);
    let interpolation_factor = near_clip_plane / forward_component;
    observer.position + vector_from_camera_to_world_coord * interpolation_factor
}

#[inline]
pub fn observed_orthonormal_basis_vectors(observer: &Camera3D) -> (Vector3, Vector3, Vector3) {
    let observed_line_of_sight = observed_line_of_sight(observer);
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

#[inline]
pub fn barymetric_vomit(pixel_a: Vector2, pixel_b: Vector2, pixel_c: Vector2) -> f32 {
    (pixel_c.x - pixel_a.x) * (pixel_b.y - pixel_a.y) - (pixel_c.y - pixel_a.y) * (pixel_b.x - pixel_a.x)
}

/// Oriented 2×area of triangle ABC in *screen space* (pixels).
/// This is the **barycentric denominator**. Its sign encodes winding (CW/CCW).
#[inline]
pub fn barycentric_denominator_area(
    screen_vertex_a: Vector2,
    screen_vertex_b: Vector2,
    screen_vertex_c: Vector2,
) -> f32 {
    (screen_vertex_c.x - screen_vertex_a.x) * (screen_vertex_b.y - screen_vertex_a.y)
        - (screen_vertex_c.y - screen_vertex_a.y) * (screen_vertex_b.x - screen_vertex_a.x)
}

/// Oriented 2×area of triangle (edge_start, edge_end, sample_pixel).
/// This is a **barycentric numerator** and a **half-space test** for coverage.
/// Use (B,C,P) for the weight of A, (C,A,P) for B, (A,B,P) for C.
#[inline]
pub fn barycentric_numerator_area(edge_start: Vector2, edge_end: Vector2, sample_pixel: Vector2) -> f32 {
    (sample_pixel.x - edge_start.x) * (edge_end.y - edge_start.y)
        - (sample_pixel.y - edge_start.y) * (edge_end.x - edge_start.x)
}

//TODO: make this a more common update mesh function somehow? to avoid having ot do load and unloads of model/meshes constantly in opengl1.1
pub fn replace_mesh_vertices(dst_mesh: &mut WeakMesh, src_vertices: &[Vector3]) {
    let dst_vertices = dst_mesh.vertices_mut();
    let src_vertex_count = src_vertices.len(); //TODO: dont try to write more vertices that dst has ofc?
    for i in 0..src_vertex_count {
        dst_vertices[i] = src_vertices[i];
    }
    dst_mesh.vertexCount = src_vertex_count as c_int;
}
