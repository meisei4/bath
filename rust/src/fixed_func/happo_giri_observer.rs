use crate::fixed_func::silhouette::{
    draw_inverted_hull_guassian_silhouette_stack, FOVY_ORTHOGRAPHIC, MODEL_POS, MODEL_SCALE,
};
use crate::fixed_func::topology::{debug_draw_triangles, Topology};
use crate::render::raylib::RaylibRenderer;
use raylib::camera::Camera3D;
use raylib::color::Color;
use raylib::consts::CameraProjection;
use raylib::drawing::{RaylibDraw, RaylibDraw3D, RaylibMode3DExt};
use raylib::ffi::{
    rlGetMatrixModelview, rlGetMatrixProjection, rlSetMatrixModelview, rlSetMatrixProjection, rlViewport,
};
use raylib::math::{Matrix, Vector3};
use raylib::models::{Model, RaylibModel};
use std::ops::Mul;

pub const CAMERA_DISTANCE: f32 = 2.0;
pub fn happo_giri_setup() -> (Vec<Camera3D>, Vec<&'static str>) {
    let diag: f32 = CAMERA_DISTANCE / CAMERA_DISTANCE.sqrt();
    let cameras: Vec<Camera3D> = vec![
        create_camera(0.0, 0.0, CAMERA_DISTANCE),  // front
        create_camera(0.0, 0.0, -CAMERA_DISTANCE), // back
        create_camera(-CAMERA_DISTANCE, 0.0, 0.0), // left
        create_camera(CAMERA_DISTANCE, 0.0, 0.0),  // right
        create_camera(diag, 0.0, diag),            // front-right
        create_camera(-diag, 0.0, diag),           // front-left
        create_camera(diag, 0.0, -diag),           // back-right
        create_camera(-diag, 0.0, -diag),          // back-left
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
    let render_w = render.handle.get_render_width();
    let render_h = render.handle.get_render_height();
    let grid_w = screen_w / grid_columns;
    let grid_h = screen_h / grid_rows;
    eprintln!(
        "screen={}x{}  render={}x{}  (HiDPI? {:?})",
        screen_w,
        screen_h,
        render_w,
        render_h,
        render.handle.get_window_scale_dpi()
    );

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
        eprintln!(
            "cell[{view_index}] viewport = x={}, y={}, w={}, h={}  -> aspect_cell={:.4}",
            viewport_x,
            viewport_y,
            grid_w,
            grid_w,
            (grid_w as f32) / (grid_w as f32)
        );

        let camera = cameras[view_index];
        print_cam("cell camera", &camera);
        draw_handle.draw_mode3D(cameras[view_index], |mut rl3d| {
            let aspect = grid_w as f32 / grid_h as f32;
            let fb_aspect = render_w as f32 / render_h as f32;
            let ortho_width = camera.fovy; // raylib: fovy = width
            let ortho_height = ortho_width / aspect; // = width / 1.0 = width
            let l = -0.5 * ortho_width;
            let r = 0.5 * ortho_width;
            let b = -0.5 * ortho_height;
            let t = 0.5 * ortho_height;
            // let proj = Matrix::perspective(FOVY_PERSPECTIVE.to_radians(), aspect,  0.01, 1000.0);
            let proj = Matrix::ortho(l, r, b, t, 0.01, 1000.0);
            let cancel_x = Matrix::scale(fb_aspect, 1.0, 1.0);
            let proj_fixed = Matrix::mul(proj, cancel_x);
            let view = Matrix::look_at(camera.position, camera.target, camera.up);
            unsafe { rlSetMatrixProjection(proj_fixed.into()) };
            unsafe { rlSetMatrixModelview(view.into()) };
            print_matrix_set("proj SET", proj_fixed);
            print_matrix_set("view SET", view);
            let proj_now = unsafe { rlGetMatrixProjection() };
            let view_now = unsafe { rlGetMatrixModelview() };
            print_matrix_rl("proj RL (after set)", proj_now.into());
            print_matrix_rl("view RL (after set)", view_now.into());
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
        fovy: FOVY_ORTHOGRAPHIC,
        projection: CameraProjection::CAMERA_ORTHOGRAPHIC,
    }
    // Camera3D {
    //     position: Vector3::new(x, y, z),
    //     target: Vector3::ZERO,
    //     up: Vector3::Y,
    //     fovy: FOVY_PERSPECTIVE,
    //     projection: CameraProjection::CAMERA_PERSPECTIVE,
    // }
}
fn print_matrix_set(tag: &str, m: Matrix) {
    let a = m.to_array();
    eprintln!(
        "{tag} (SET):\n\
         [{:>8.4} {:>8.4} {:>8.4} {:>8.4}]\n\
         [{:>8.4} {:>8.4} {:>8.4} {:>8.4}]\n\
         [{:>8.4} {:>8.4} {:>8.4} {:>8.4}]\n\
         [{:>8.4} {:>8.4} {:>8.4} {:>8.4}]",
        a[0], a[4], a[8], a[12], a[1], a[5], a[9], a[13], a[2], a[6], a[10], a[14], a[3], a[7], a[11], a[15],
    );
}

fn print_matrix_rl(tag: &str, m: Matrix) {
    eprintln!(
        "{tag} (RL):\n\
         [{:>8.4} {:>8.4} {:>8.4} {:>8.4}]\n\
         [{:>8.4} {:>8.4} {:>8.4} {:>8.4}]\n\
         [{:>8.4} {:>8.4} {:>8.4} {:>8.4}]\n\
         [{:>8.4} {:>8.4} {:>8.4} {:>8.4}]",
        m.m0, m.m4, m.m8, m.m12, m.m1, m.m5, m.m9, m.m13, m.m2, m.m6, m.m10, m.m14, m.m3, m.m7, m.m11, m.m15,
    );
}

fn print_cam(tag: &str, cam: &Camera3D) {
    let mode = match cam.projection {
        CameraProjection::CAMERA_PERSPECTIVE => "PERSPECTIVE",
        CameraProjection::CAMERA_ORTHOGRAPHIC => "ORTHOGRAPHIC",
    };
    eprintln!(
        "{tag}: mode={mode}  fovy={:.3}  pos=({:.3},{:.3},{:.3})  target=({:.3},{:.3},{:.3})  up=({:.3},{:.3},{:.3})",
        cam.fovy,
        cam.position.x,
        cam.position.y,
        cam.position.z,
        cam.target.x,
        cam.target.y,
        cam.target.z,
        cam.up.x,
        cam.up.y,
        cam.up.z
    );
}
