use crate::fixed_func::silhouette::{draw_inverted_hull_guassian_silhouette_stack, FOVY, MODEL_POS, MODEL_SCALE};
use crate::fixed_func::topology::{debug_draw_triangles, Topology};
use crate::render::raylib::RaylibRenderer;
use raylib::camera::Camera3D;
use raylib::color::Color;
use raylib::consts::CameraProjection;
use raylib::drawing::{RaylibDraw, RaylibDraw3D, RaylibMode3DExt};
use raylib::ffi::rlViewport;
use raylib::math::Vector3;
use raylib::models::{Model, RaylibModel};

pub fn happo_giri_setup() -> (Vec<Camera3D>, Vec<&'static str>) {
    let diag: f32 = 2.0_f32 / 2.0_f32.sqrt();
    let cameras: Vec<Camera3D> = vec![
        create_camera(0.0, 0.0, 2.0),     // front
        create_camera(0.0, 0.0, -2.0),    // back
        create_camera(-2.0, 0.0, 0.0),    // left
        create_camera(2.0, 0.0, 0.0),     // right
        create_camera(diag, 0.0, diag),   // front-right
        create_camera(-diag, 0.0, diag),  // front-left
        create_camera(diag, 0.0, -diag),  // back-right
        create_camera(-diag, 0.0, -diag), // back-left
    ];
    let labels: Vec<&'static str> = vec![
        "front",
        "back",
        "left",
        "right",
        "front-right",
        "front-left",
        "back-right",
        "back-left",
    ];
    (cameras, labels)
}

pub fn happo_giri_draw(
    render: &mut RaylibRenderer,
    cameras: &[Camera3D],
    labels: &[&'static str],
    grid_columns: i32,
    grid_rows: i32,
    target_model: &Model,
    inverted_hull: Option<&Model>,
    mesh_rotation: f32,
) {
    let screen_w = render.handle.get_screen_width();
    let screen_h = render.handle.get_screen_height();
    let grid_w = screen_w / grid_columns;
    let grid_h = screen_h / grid_rows;
    let aspect = grid_w as f32 / grid_h as f32;
    let mut draw_handle = render.handle.begin_drawing(&render.thread);
    draw_handle.clear_background(Color::BLACK);
    for view_index in 0..8 {
        let column_index = (view_index as i32) % grid_columns;
        let row_index = (view_index as i32) / grid_columns;
        let row_index_inverse = (grid_rows - 1) - row_index;
        let viewport_x = column_index * grid_w;
        let viewport_y = row_index_inverse * grid_h;
        unsafe {
            rlViewport(viewport_x, viewport_y, grid_w, grid_w);
        }
        draw_handle.draw_mode3D(cameras[view_index], |mut rl3d| {
            rl3d.draw_model_ex(
                target_model,
                MODEL_POS,
                Vector3::Y,
                mesh_rotation.to_degrees(),
                MODEL_SCALE,
                Color::BLUE,
            );
            rl3d.draw_model_wires_ex(
                target_model,
                MODEL_POS,
                Vector3::Y,
                mesh_rotation.to_degrees(),
                MODEL_SCALE,
                Color::RED,
            );
            rl3d.draw_model_points_ex(
                target_model,
                MODEL_POS,
                Vector3::Y,
                mesh_rotation.to_degrees(),
                MODEL_SCALE,
                Color::GREEN,
            );
        });
        let topology = Topology::build_topology(&target_model.meshes()[0])
            .welded_vertices()
            .triangles()
            .welded_vertices_per_triangle()
            .neighbors_per_triangle()
            .vertices_per_triangle()
            .front_triangles(mesh_rotation, &cameras[view_index])
            .back_triangles()
            .silhouette_triangles()
            .build();
        if let Some(triangle_set) = topology.silhouette_triangles_snapshot.as_ref() {
            debug_draw_triangles(
                cameras[view_index],
                &mut draw_handle,
                &topology,
                mesh_rotation,
                &triangle_set,
                // None,
                Some(Color::new(255, 32, 32, 90)),
                true,
                32,
            );
        }
        if let Some(inverted_hull) = inverted_hull {
            draw_handle.draw_mode3D(cameras[view_index], |mut rl3d| {
                draw_inverted_hull_guassian_silhouette_stack(&mut rl3d, inverted_hull, mesh_rotation);
            });
        }
        unsafe {
            rlViewport(0, 0, screen_w, screen_h);
        }
        let label_pos_x = column_index * grid_w;
        let label_pos_y = row_index * grid_h;
        draw_handle.draw_text(labels[view_index], label_pos_x, label_pos_y, 14, Color::WHITE);
    }
}

pub fn create_camera(x: f32, y: f32, z: f32) -> Camera3D {
    Camera3D {
        position: Vector3::new(x, y, z),
        target: Vector3::ZERO,
        up: Vector3::Y,
        fovy: FOVY,
        projection: CameraProjection::CAMERA_ORTHOGRAPHIC,
    }
}
