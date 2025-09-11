use crate::fixed_func::topology::observed_line_of_sight;
use raylib::camera::Camera3D;
use raylib::color::Color;
use raylib::drawing::RaylibDraw3D;
use raylib::ffi::rlSetLineWidth;
use raylib::math::Vector3;

pub const RAY_TO_NDC_MATH_NEAR_PLANE_DISTANCE: f32 = 1.0;
pub const PROXY_NEAR_PLANE_GRID_STEP_PIXELS: i32 = 24;

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

pub fn draw_proxy_near_plane(
    rl3d: &mut impl RaylibDraw3D,
    observer: &Camera3D,
    aspect: f32,
    proxy_near_plane_distance: f32,
    screen_width_pixels: i32,
    screen_height_pixels: i32,
    grid_step_pixels: i32,
) {
    let grid_color = Color {
        a: 140,
        ..Color::SKYBLUE
    };
    draw_near_plane_pixel_grid(
        rl3d,
        observer,
        aspect,
        proxy_near_plane_distance,
        screen_width_pixels,
        screen_height_pixels,
        grid_step_pixels,
        grid_color,
    );

    draw_near_plane_ndc_axes(rl3d, observer, aspect, proxy_near_plane_distance, 0.2);
}

pub fn draw_near_plane_ndc_axes(
    rl3d: &mut impl RaylibDraw3D,
    observer: &Camera3D,
    aspect: f32,
    near_clip_plane: f32,
    axis_length_fraction_of_half_extent: f32,
) {
    let (observed_line_of_sight, observed_right, observed_up) = observed_orthonormal_basis_vectors(observer);

    let near_center = observer.position + observed_line_of_sight * near_clip_plane;

    let half_vertical_extent = (observer.fovy.to_radians() * 0.5).tan() * near_clip_plane;
    let half_horizontal_extent = half_vertical_extent * aspect;

    let u_extent = observed_right * (half_horizontal_extent * axis_length_fraction_of_half_extent);
    let v_extent = observed_up * (half_vertical_extent * axis_length_fraction_of_half_extent);

    rl3d.draw_line3D(near_center - u_extent, near_center + u_extent, Color::DARKBLUE);
    rl3d.draw_line3D(near_center - v_extent, near_center + v_extent, Color::DARKBLUE);
}

pub fn draw_near_plane_markers(
    rl3d: &mut impl RaylibDraw3D,
    observer: &Camera3D,
    near_clip_plane: f32,
    model_position: Vector3,
    model_scale: Vector3,
    mesh_rotation_radians: f32,
) {
    let cosine = mesh_rotation_radians.cos();
    let sine = mesh_rotation_radians.sin();

    let local_points = [
        Vector3::new(1.0, 0.0, 0.0),
        Vector3::new(-1.0, 0.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
        Vector3::new(0.0, -1.0, 0.0),
        Vector3::new(0.0, 0.0, 1.0),
        Vector3::new(0.0, 0.0, -1.0),
    ];

    for local in local_points.iter() {
        let scaled = Vector3::new(
            local.x * model_scale.x,
            local.y * model_scale.y,
            local.z * model_scale.z,
        );

        let rotated = Vector3::new(
            scaled.x * cosine + scaled.z * sine,
            scaled.y,
            -scaled.x * sine + scaled.z * cosine,
        );

        let world_point = model_position + rotated;

        if let Some(intersection_point) = near_plane_intersection(observer, near_clip_plane, world_point) {
            unsafe { rlSetLineWidth(1.0) }
            rl3d.draw_line3D(world_point, intersection_point, Color::YELLOW);
            rl3d.draw_point3D(intersection_point, Color::GOLD)
        }
    }
}

pub fn near_plane_intersection(observer: &Camera3D, near_clip_plane: f32, world_point: Vector3) -> Option<Vector3> {
    let (observed_line_of_sight, _observed_right, _observed_up) = observed_orthonormal_basis_vectors(observer);
    let vector_from_camera_to_point = world_point - observer.position;
    let forward_component = vector_from_camera_to_point.dot(observed_line_of_sight);
    if forward_component <= 0.0 {
        return None;
    }
    let interpolation_factor = near_clip_plane / forward_component;
    if interpolation_factor <= 0.0 {
        return None;
    }
    Some(observer.position + vector_from_camera_to_point * interpolation_factor)
}

pub fn draw_near_plane_pixel_grid(
    rl3d: &mut impl RaylibDraw3D,
    observer: &Camera3D,
    aspect: f32,
    near_clip_plane: f32,
    screen_width_pixels: i32,
    screen_height_pixels: i32,
    pixel_step: i32,
    color: Color,
) {
    let (observed_line_of_sight, observed_right, observed_up) = observed_orthonormal_basis_vectors(observer);
    let near_center = observer.position + observed_line_of_sight * near_clip_plane;

    let half_vertical_angle_radians = observer.fovy.to_radians() * 0.5;
    let top = near_clip_plane * half_vertical_angle_radians.tan();
    let right = top * aspect;

    let mut x_pixel = 0;
    while x_pixel <= screen_width_pixels {
        let x_ndc = ((x_pixel as f32 + 0.5) / screen_width_pixels as f32) * 2.0 - 1.0;
        let a = near_center + observed_right * (x_ndc * right) + observed_up * (-1.0 * top);
        let b = near_center + observed_right * (x_ndc * right) + observed_up * (1.0 * top);
        rl3d.draw_line3D(a, b, color);
        x_pixel += pixel_step;
    }
    let mut y_pixel = 0;
    while y_pixel <= screen_height_pixels {
        let y_ndc = ((y_pixel as f32 + 0.5) / screen_height_pixels as f32) * 2.0 - 1.0;
        let a = near_center + observed_up * (y_ndc * top) + observed_right * (-1.0 * right);
        let b = near_center + observed_up * (y_ndc * top) + observed_right * (1.0 * right);
        rl3d.draw_line3D(a, b, color);
        y_pixel += pixel_step;
    }
}

pub fn project_world_to_near_plane_pixels(
    observer: &Camera3D,
    aspect: f32,
    near_clip_plane: f32,
    screen_width_pixels: i32,
    screen_height_pixels: i32,
    world_point: Vector3,
) -> Option<(i32, i32)> {
    let (observed_line_of_sight, observed_right, observed_up) = observed_orthonormal_basis_vectors(observer);

    let vector_from_camera_to_point = world_point - observer.position;
    let forward_component = vector_from_camera_to_point.dot(observed_line_of_sight);
    if forward_component <= 0.0 {
        return None;
    }

    let intersection_point = observer.position + vector_from_camera_to_point * (near_clip_plane / forward_component);

    let near_center = observer.position + observed_line_of_sight * near_clip_plane;
    let delta_on_plane = intersection_point - near_center;

    let half_vertical_angle_radians = observer.fovy.to_radians() * 0.5;
    let top = near_clip_plane * half_vertical_angle_radians.tan();
    let right = top * aspect;

    let x_normalized_device = delta_on_plane.dot(observed_right) / right;
    let y_normalized_device = delta_on_plane.dot(observed_up) / top;

    if x_normalized_device < -1.0
        || x_normalized_device > 1.0
        || y_normalized_device < -1.0
        || y_normalized_device > 1.0
    {
        return None;
    }

    let pixel_x = ((x_normalized_device * 0.5 + 0.5) * screen_width_pixels as f32).floor() as i32;
    let pixel_y = ((1.0 - (y_normalized_device * 0.5 + 0.5)) * screen_height_pixels as f32).floor() as i32;

    Some((pixel_x, pixel_y))
}

pub fn draw_frustum(
    rl3d: &mut impl RaylibDraw3D,
    observer: &Camera3D,
    aspect: f32,
    near_clip_plane: f32,
    far_clip_plane: f32,
) {
    let points = compute_frustum_corners(observer, aspect, near_clip_plane, far_clip_plane);
    let near_top_left = points[0];
    let near_top_right = points[1];
    let near_bottom_right = points[2];
    let near_bottom_left = points[3];

    let far_top_left = points[4];
    let far_top_right = points[5];
    let far_bottom_right = points[6];
    let far_bottom_left = points[7];
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

    let near_clip_plane_center = observer.position + observed_line_of_sight * near_clip_plane;
    let far_clip_plane_center = observer.position + observed_line_of_sight * far_clip_plane;

    let half_fovy = observer.fovy.to_radians() * 0.5;
    let near_half_height = near_clip_plane * half_fovy.tan();
    let near_half_width = near_half_height * aspect;

    let far_half_height = far_clip_plane * half_fovy.tan();
    let far_half_width = far_half_height * aspect;

    let near_top_left = near_clip_plane_center + observed_up * near_half_height - observed_right * near_half_width;
    let near_top_right = near_clip_plane_center + observed_up * near_half_height + observed_right * near_half_width;
    let near_bottom_right = near_clip_plane_center - observed_up * near_half_height + observed_right * near_half_width;
    let near_bottom_left = near_clip_plane_center - observed_up * near_half_height - observed_right * near_half_width;

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

#[inline]
pub fn observed_orthonormal_basis_vectors(observer: &Camera3D) -> (Vector3, Vector3, Vector3) {
    let observed_line_of_sight = observed_line_of_sight(observer);
    let observed_right = observed_line_of_sight.cross(observer.up).normalize_or_zero();
    let observed_up = observed_right.cross(observed_line_of_sight).normalize_or_zero();
    (observed_line_of_sight, observed_right, observed_up)
}
