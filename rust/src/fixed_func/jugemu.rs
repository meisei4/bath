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

pub const PERSPECTIVE_CORRECT: bool = true;
pub const DEPTH_TEST_ON: bool = true;

pub fn draw_near_plane_software_raster(
    rl3d: &mut impl RaylibDraw3D,
    screen_w: i32,
    screen_h: i32,
    observer: &Camera3D,
    near_clip_plane: f32,
    mesh: &WeakMesh,
    model_position: Vector3,
    model_scale: Vector3,
    mesh_rotation_radians: f32,
    step: i32,
) {
    let observation_basis = observed_orthonormal_basis_vectors(observer);
    let depth_buffer_len = (screen_w * screen_h) as usize;
    let mut depth_buffer = vec![f32::INFINITY; depth_buffer_len];
    let triangles: Vec<[usize; 3]> = mesh.triangles().collect();
    let colors = mesh.colors().unwrap();
    let vertices = mesh.vertices();
    for [vertex_a, vertex_b, vertex_c] in triangles {
        let mut triangle = [vertices[vertex_a], vertices[vertex_b], vertices[vertex_c]];
        rotate_vertices_in_plane(&mut triangle, mesh_rotation_radians);
        let world_triangle = triangle.map(|vertex| model_position + vertex * model_scale);
        //TODO: this is only for getting x,y and depth data, it doesnt project anything. make that more clear
        let projection_a =
            perspective_project_stage_data(screen_w, screen_h, observer, near_clip_plane, world_triangle[0]);
        let projection_b =
            perspective_project_stage_data(screen_w, screen_h, observer, near_clip_plane, world_triangle[1]);
        let projection_c =
            perspective_project_stage_data(screen_w, screen_h, observer, near_clip_plane, world_triangle[2]);
        rasterize_triangle(
            rl3d,
            screen_w,
            screen_h,
            observer,
            observation_basis,
            near_clip_plane,
            [projection_a, projection_b, projection_c],
            [vertex_a, vertex_b, vertex_c],
            colors,
            &mut depth_buffer,
            step,
        );
    }
}

pub fn perspective_project_stage_data(
    screen_w: i32,
    screen_h: i32,
    observer: &Camera3D,
    near_clip_plane: f32,
    world_coord: Vector3,
) -> (f32, f32, f32) {
    let aspect = screen_w as f32 / screen_h as f32;
    let (observed_line_of_sight, observed_right, observed_up) = observed_orthonormal_basis_vectors(observer);
    let center_near_clip_plane = observer.position + observed_line_of_sight * near_clip_plane;
    let half_height_near_clip_plane = near_clip_plane * (observer.fovy.to_radians() * 0.5).tan();
    let half_width_near_clip_plane = half_height_near_clip_plane * aspect;
    let ray_from_world_to_observer = world_coord - observer.position;
    let depth_z = ray_from_world_to_observer.dot(observed_line_of_sight);
    let (x_ndc, y_ndc) = if PERSPECTIVE_CORRECT {
        let intersection_point = near_plane_intersection(observer, near_clip_plane, world_coord);
        let clip_plane_vector = intersection_point - center_near_clip_plane;
        (
            clip_plane_vector.dot(observed_right) / half_width_near_clip_plane,
            clip_plane_vector.dot(observed_up) / half_height_near_clip_plane,
        )
    } else {
        (
            //TODO: this idk its still the same as what PERSPECTIVE CORRECT DRAWS PRETTY MUCH IDK HOW TO NOT
            // TRY SKIPPING NORMALIZE, BUT MOVE TO A SEPARATE SPACE OR SOMETHING
            ray_from_world_to_observer.normalize_or_zero().dot(observed_right) / half_width_near_clip_plane,
            ray_from_world_to_observer.normalize_or_zero().dot(observed_up) / half_height_near_clip_plane,
        )
    };

    let pixel_x = (x_ndc * 0.5 + 0.5) * screen_w as f32;
    let pixel_y = (1.0 - (y_ndc * 0.5 + 0.5)) * screen_h as f32;
    (pixel_x, pixel_y, depth_z)
}

fn rasterize_triangle(
    rl3d: &mut impl RaylibDraw3D,
    screen_w: i32,
    screen_h: i32,
    observer: &Camera3D,
    observation_basis: (Vector3, Vector3, Vector3),
    near_clip_plane: f32,
    projected_triangle: [(f32, f32, f32); 3],
    vertex_indices: [usize; 3],
    colors: &[Color],
    depth_buffer: &mut [f32],
    step: i32,
) {
    let aspect = screen_w as f32 / screen_h as f32;
    let (observed_line_of_sight, observed_right, observed_up) = observation_basis;

    let center_near_clip_plane = observer.position + observed_line_of_sight * near_clip_plane;
    let half_fovy = observer.fovy.to_radians() * 0.5;
    let half_height_near_clip_plane = near_clip_plane * half_fovy.tan();
    let half_width_near_clip_plane = half_height_near_clip_plane * aspect;

    let pixel_x_half_step = observed_right * (half_width_near_clip_plane / screen_w as f32);
    let pixel_y_half_step = observed_up * (half_height_near_clip_plane / screen_h as f32);

    let pixels: [Vector2; 3] = projected_triangle.map(|(x, y, _)| Vector2::new(x, y));
    let depths: [f32; 3] = projected_triangle.map(|(_, _, z)| z);
    let inverse_depths: [f32; 3] = depths.map(|z| 1.0 / z);

    let full_signed_barycentric_weight_component =
        full_signed_barycentric_weight_component(pixels[0], pixels[1], pixels[2]);
    let (min_x, max_x, min_y, max_y) = rasterization_bounding_box(&pixels, screen_w, screen_h);

    let start_x = (min_x / step) * step;
    let start_y = (min_y / step) * step;

    let mut raster_pixel_y = start_y;
    while raster_pixel_y <= max_y {
        let sample_center_y = raster_pixel_y as f32 + 0.5;

        let mut raster_pixel_x = start_x;
        while raster_pixel_x <= max_x {
            let sample_center_x = raster_pixel_x as f32 + 0.5;
            let sample_pixel = Vector2::new(sample_center_x, sample_center_y);

            if let Some(barycentric_weights) =
                barycentric_weights_for_sample(sample_pixel, pixels, full_signed_barycentric_weight_component)
            {
                let depth = interpolate_depth(barycentric_weights, inverse_depths);
                let buffer_index = (raster_pixel_y as usize) * (screen_w as usize) + (raster_pixel_x as usize);

                if depth_test(depth, depth_buffer, buffer_index) {
                    let raster_color = interpolate_color(barycentric_weights, vertex_indices, colors);

                    let x_ndc = (sample_pixel.x / screen_w as f32) * 2.0 - 1.0;
                    let y_ndc = 1.0 - (sample_pixel.y / screen_h as f32) * 2.0;

                    let near_plane_origin = center_near_clip_plane
                        + observed_right * (x_ndc * half_width_near_clip_plane)
                        + observed_up * (y_ndc * half_height_near_clip_plane);

                    let quad_corner_a = near_plane_origin - pixel_x_half_step - pixel_y_half_step;
                    let quad_corner_b = near_plane_origin + pixel_x_half_step - pixel_y_half_step;
                    let quad_corner_c = near_plane_origin + pixel_x_half_step + pixel_y_half_step;
                    let quad_corner_d = near_plane_origin - pixel_x_half_step + pixel_y_half_step;

                    rl3d.draw_triangle3D(quad_corner_a, quad_corner_b, quad_corner_c, raster_color);
                    rl3d.draw_triangle3D(quad_corner_a, quad_corner_c, quad_corner_d, raster_color);
                }
            }

            raster_pixel_x += step;
        }
        raster_pixel_y += step;
    }
}

#[inline]
pub fn depth_test(depth: f32, depth_buffer: &mut [f32], buffer_index: usize) -> bool {
    if !DEPTH_TEST_ON {
        return true;
    }
    if depth < depth_buffer[buffer_index] {
        depth_buffer[buffer_index] = depth;
        true
    } else {
        false
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
    let observed_line_of_sight = observed_line_of_sight(observer).normalize_or_zero();
    let observed_right = observed_line_of_sight.cross(observer.up).normalize_or_zero();
    let observed_up = observed_right.cross(observed_line_of_sight).normalize_or_zero();
    (observed_line_of_sight, observed_right, observed_up)
}

#[inline]
fn barycentric_weights_for_sample(
    sample_vertex: Vector2,
    vertices: [Vector2; 3],
    full_signed_barycentric_weight_component: f32,
) -> Option<[f32; 3]> {
    let [vertex_a, vertex_b, vertex_c] = vertices;
    let partial_signed_weight_components = [
        partial_signed_barycentric_weight_component((vertex_b, vertex_c), sample_vertex),
        partial_signed_barycentric_weight_component((vertex_c, vertex_a), sample_vertex),
        partial_signed_barycentric_weight_component((vertex_a, vertex_b), sample_vertex),
    ];

    if partial_signed_weight_components
        .iter()
        .all(|&weight_component| weight_component * full_signed_barycentric_weight_component >= 0.0)
    {
        Some([
            partial_signed_weight_components[0] / full_signed_barycentric_weight_component,
            partial_signed_weight_components[1] / full_signed_barycentric_weight_component,
            partial_signed_weight_components[2] / full_signed_barycentric_weight_component,
        ])
    } else {
        None
    }
}

fn interpolate_depth(barycentric_weights: [f32; 3], depth_divides: [f32; 3]) -> f32 {
    1.0 / barycentric_weights
        .iter()
        .zip(depth_divides.iter())
        .map(|(weight, depth_divide)| weight * depth_divide)
        .sum::<f32>()
}

fn interpolate_color(weights: [f32; 3], indices: [usize; 3], colors: &[Color]) -> Color {
    let color_channels = |color_val: fn(&Color) -> u8| {
        weights
            .iter()
            .zip(indices.iter())
            .map(|(weight, &vertex_index)| weight * color_val(&colors[vertex_index]) as f32)
            .sum::<f32>()
            .clamp(0.0, 255.0) as u8
    };

    Color {
        r: color_channels(|color| color.r),
        g: color_channels(|color| color.g),
        b: color_channels(|color| color.b),
        a: 255,
    }
}

/// Compute the screen-space rasterization bounding box of a triangle.
///
/// Mathematically:
/// - Take the min/max of the projected triangle’s screen-space vertex coordinates.
/// - Quantize (floor/ceil) those continuous values to integer pixel grid coordinates.
/// - Clamp to the framebuffer resolution to perform trivial clipping.
///
/// This defines the **axis-aligned bounding box (AABB)** in raster space that
/// bounds the triangle. The rasterizer will only scan over this box.
#[inline]
pub fn rasterization_bounding_box(pixels: &[Vector2; 3], screen_w: i32, screen_h: i32) -> (i32, i32, i32, i32) {
    // continuous min/max in screen space
    let (min_x, max_x) = pixels
        .iter()
        .map(|p| p.x)
        .fold((f32::INFINITY, f32::NEG_INFINITY), |(lo, hi), x| (lo.min(x), hi.max(x)));
    let (min_y, max_y) = pixels
        .iter()
        .map(|p| p.y)
        .fold((f32::INFINITY, f32::NEG_INFINITY), |(lo, hi), y| (lo.min(y), hi.max(y)));

    let (mut min_ix, mut max_ix) = (min_x.floor() as i32, max_x.ceil() as i32);
    let (mut min_iy, mut max_iy) = (min_y.floor() as i32, max_y.ceil() as i32);

    min_ix = min_ix.clamp(0, screen_w - 1);
    max_ix = max_ix.clamp(0, screen_w - 1);
    min_iy = min_iy.clamp(0, screen_h - 1);
    max_iy = max_iy.clamp(0, screen_h - 1);

    (min_ix, max_ix, min_iy, max_iy)
}

#[inline]
pub fn barymetric_vomit(pixel_a: Vector2, pixel_b: Vector2, pixel_c: Vector2) -> f32 {
    (pixel_c.x - pixel_a.x) * (pixel_b.y - pixel_a.y) - (pixel_c.y - pixel_a.y) * (pixel_b.x - pixel_a.x)
}

/// Oriented 2×area of triangle ABC in *screen space* (pixels).
/// This is the **barycentric denominator**. Its sign encodes winding (CW/CCW).
#[inline]
pub fn full_signed_barycentric_weight_component(
    screen_vertex_a: Vector2,
    screen_vertex_b: Vector2,
    screen_vertex_c: Vector2,
) -> f32 {
    (screen_vertex_c.x - screen_vertex_a.x) * (screen_vertex_b.y - screen_vertex_a.y)
        - (screen_vertex_c.y - screen_vertex_a.y) * (screen_vertex_b.x - screen_vertex_a.x)
}

/// Oriented 2×area of triangle (edge_start, edge_end, sample_pixel).
/// This is a **barycentric numerator** and a **half-space test** for coverage.
/// Use ((B,C),P) for the weight of A, ((C,A),P) for B, ((A,B),P) for C.
pub fn partial_signed_barycentric_weight_component(edge: (Vector2, Vector2), sample_pixel: Vector2) -> f32 {
    let (edge_start, edge_end) = edge;
    (sample_pixel.x - edge_start.x) * (edge_end.y - edge_start.y)
        - (sample_pixel.y - edge_start.y) * (edge_end.x - edge_start.x)
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

//TODO: make this a more common update mesh function somehow? to avoid having ot do load and unloads of model/meshes constantly in opengl1.1
pub fn replace_mesh_vertices(dst_mesh: &mut WeakMesh, src_vertices: &[Vector3]) {
    let dst_vertices = dst_mesh.vertices_mut();
    let src_vertex_count = src_vertices.len(); //TODO: dont try to write more vertices that dst has ofc?
    for i in 0..src_vertex_count {
        dst_vertices[i] = src_vertices[i];
    }
    dst_mesh.vertexCount = src_vertex_count as c_int;
}
