use asset_payload::SPHERE_PATH;
use bath::fixed_func::jugemu::{
    draw_frustum, draw_near_plane_projection_markers_for_model, draw_observed_axes,
    project_world_point_to_main_observer_pixels,
};
use bath::fixed_func::silhouette::{ANGULAR_VELOCITY, FOVY_PERSPECTIVE, MODEL_SCALE, SCALE_TWEAK};
use bath::render::raylib::RaylibRenderer;
use bath::render::raylib_util::N64_WIDTH;
use bath::render::renderer::Renderer;
use raylib::camera::Camera3D;
use raylib::color::Color;
use raylib::consts::CameraProjection;
use raylib::drawing::{RaylibDraw, RaylibDraw3D, RaylibMode3DExt};
use raylib::ffi::{rlSetLineWidth, rlSetPointSize};
use raylib::math::Vector3;

pub const MODEL_POS_BACK: Vector3 = Vector3::new(0.0, 0.0, -2.0);

fn main() {
    let mut i_time = 0.0f32;
    let mut mesh_rotation = 0.0f32;
    let mut render = RaylibRenderer::init(N64_WIDTH, N64_WIDTH);

    let near_clip_plane: f32 = 1.0;
    let far_clip_plane: f32 = 3.0;

    let main_observer = Camera3D {
        position: Vector3::new(0.0, 0.0, 2.0),
        target: Vector3::ZERO,
        up: Vector3::Y,
        fovy: FOVY_PERSPECTIVE,
        projection: CameraProjection::CAMERA_PERSPECTIVE,
    };

    let screen_width_pixels = render.handle.get_screen_width();
    let screen_height_pixels = render.handle.get_screen_height();
    let main_observer_aspect = screen_width_pixels as f32 / screen_height_pixels as f32;

    let jugemu = Camera3D {
        position: Vector3::new(3.0, 1.0, 3.0),
        target: Vector3::ZERO,
        up: Vector3::Y,
        fovy: FOVY_PERSPECTIVE,
        projection: CameraProjection::CAMERA_PERSPECTIVE,
    };

    let main_model = render.handle.load_model(&render.thread, SPHERE_PATH).unwrap();

    while !render.handle.window_should_close() {
        i_time += render.handle.get_frame_time();
        mesh_rotation -= ANGULAR_VELOCITY * render.handle.get_frame_time();

        let mut draw_handle = render.handle.begin_drawing(&render.thread);
        draw_handle.clear_background(Color::BLACK);

        draw_handle.draw_mode3D(jugemu, |mut rl3d| {
            unsafe { rlSetLineWidth(2.0) };
            rl3d.draw_model_wires_ex(
                &main_model,
                MODEL_POS_BACK,
                Vector3::Y,
                mesh_rotation.to_degrees(),
                MODEL_SCALE * SCALE_TWEAK,
                Color::RED,
            );
            unsafe { rlSetPointSize(6.0) };
            rl3d.draw_model_points_ex(
                &main_model,
                MODEL_POS_BACK,
                Vector3::Y,
                mesh_rotation.to_degrees(),
                MODEL_SCALE * SCALE_TWEAK,
                Color::GREEN,
            );

            draw_observed_axes(&mut rl3d, &main_observer);
            draw_frustum(
                &mut rl3d,
                &main_observer,
                main_observer_aspect,
                near_clip_plane,
                far_clip_plane,
            );
            draw_near_plane_projection_markers_for_model(
                &mut rl3d,
                &main_observer,
                near_clip_plane,
                MODEL_POS_BACK,
                MODEL_SCALE * SCALE_TWEAK,
                mesh_rotation,
            );
        });

        if let Some((pixel_x, pixel_y)) = project_world_point_to_main_observer_pixels(
            &main_observer,
            main_observer_aspect,
            near_clip_plane,
            screen_width_pixels,
            screen_height_pixels,
            MODEL_POS_BACK,
        ) {
            draw_handle.draw_pixel(pixel_x, pixel_y, Color::YELLOW);
        }
    }
}
