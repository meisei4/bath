use asset_payload::SPHERE_PATH;
use bath::fixed_func::jugemu::apply_barycentric_palette;
use bath::fixed_func::silhouette::{ANGULAR_VELOCITY, FOVY_PERSPECTIVE, MODEL_SCALE};
use bath::render::raylib::RaylibRenderer;
use bath::render::raylib_util::N64_WIDTH;
use bath::render::renderer::Renderer;
use raylib::camera::Camera3D;
use raylib::color::Color;
use raylib::consts::CameraProjection;
use raylib::drawing::{RaylibDraw, RaylibDraw3D, RaylibMode3DExt};
use raylib::math::Vector3;
use raylib::models::RaylibModel;

pub const MODEL_POS_BACK: Vector3 = Vector3::new(0.0, 0.0, -2.0);
pub const OBSERVER_POS: Vector3 = Vector3::new(0.0, 0.0, 2.0);

fn main() {
    let mut mesh_rotation = 0.0f32;
    let mut render = RaylibRenderer::init(N64_WIDTH, N64_WIDTH);

    let main_observer = Camera3D {
        position: OBSERVER_POS,
        target: Vector3::ZERO,
        up: Vector3::Y,
        fovy: FOVY_PERSPECTIVE,
        projection: CameraProjection::CAMERA_PERSPECTIVE,
    };

    let mut main_model = render.handle.load_model(&render.thread, SPHERE_PATH).unwrap();
    apply_barycentric_palette(&mut main_model.meshes_mut()[0]);
    while !render.handle.window_should_close() {
        mesh_rotation -= ANGULAR_VELOCITY * render.handle.get_frame_time();

        let mut draw_handle = render.handle.begin_drawing(&render.thread);
        draw_handle.clear_background(Color::BLACK);

        draw_handle.draw_mode3D(main_observer, |mut rl3d| {
            rl3d.draw_model_ex(
                &main_model,
                MODEL_POS_BACK,
                Vector3::Y,
                mesh_rotation.to_degrees(),
                MODEL_SCALE,
                Color::WHITE,
            );
            // unsafe { rlSetLineWidth(2.0) };
            // rl3d.draw_model_wires_ex(
            //     &main_model,
            //     MODEL_POS_BACK,
            //     Vector3::Y,
            //     mesh_rotation.to_degrees(),
            //     MODEL_SCALE,
            //     Color::RED,
            // );
            // unsafe { rlSetPointSize(6.0) };
            // rl3d.draw_model_points_ex(
            //     &main_model,
            //     MODEL_POS_BACK,
            //     Vector3::Y,
            //     mesh_rotation.to_degrees(),
            //     MODEL_SCALE,
            //     Color::GREEN,
            // );
        });
    }
}
