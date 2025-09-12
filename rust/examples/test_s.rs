use asset_payload::SPHERE_PATH;
use bath::fixed_func::silhouette::FOVY_ORTHOGRAPHIC;
use bath::fixed_func::silhouette::{ANGULAR_VELOCITY, MODEL_POS, MODEL_SCALE, SCALE_TWEAK};
use bath::fixed_func::topology::observed_line_of_sight;
use bath::render::raylib::RaylibRenderer;
use bath::render::raylib_util::N64_WIDTH;
use bath::render::renderer::Renderer;
use raylib::camera::Camera3D;
use raylib::color::Color;
use raylib::consts::CameraProjection;
use raylib::drawing::{RaylibDraw, RaylibDraw3D, RaylibMode3DExt};
use raylib::ffi::{rlSetLineWidth, rlSetPointSize};
use raylib::math::Vector3;

fn main() {
    let mut i_time = 0.0f32;
    let mut mesh_rotation = 0.0f32;
    let mut render = RaylibRenderer::init(N64_WIDTH, N64_WIDTH);
    let main_observer = Camera3D {
        position: Vector3::new(0.0, 0.0, 2.0),
        target: Vector3::ZERO,
        up: Vector3::Y,
        fovy: FOVY_ORTHOGRAPHIC,
        projection: CameraProjection::CAMERA_ORTHOGRAPHIC,
    };
    let mut main_model = render.handle.load_model(&render.thread, SPHERE_PATH).unwrap();
    //TODO: figure out how to do this better lmao
    // main_model.materials_mut()[0].set_material_texture(MATERIAL_MAP_ALBEDO, silhouette_texture);
    let observed_los = observed_line_of_sight(&main_observer);
    while !render.handle.window_should_close() {
        i_time += render.handle.get_frame_time();
        mesh_rotation -= ANGULAR_VELOCITY * render.handle.get_frame_time();
        let mut draw_handle = render.handle.begin_drawing(&render.thread);
        draw_handle.clear_background(Color::BLACK);
        draw_handle.draw_mode3D(main_observer, |mut rl3d| {
            // rl3d.draw_model_ex(
            //     &main_model,
            //     MODEL_POS,
            //     Vector3::Y,
            //     mesh_rotation.to_degrees(),
            //     MODEL_SCALE * SCALE_TWEAK,
            //     Color::BLUE,
            // );
            unsafe { rlSetLineWidth(5.0) };
            rl3d.draw_model_wires_ex(
                &main_model,
                MODEL_POS,
                Vector3::Y,
                mesh_rotation.to_degrees(),
                MODEL_SCALE * SCALE_TWEAK,
                Color::RED,
            );
            unsafe { rlSetPointSize(20.0) };
            rl3d.draw_model_points_ex(
                &main_model,
                MODEL_POS,
                Vector3::Y,
                mesh_rotation.to_degrees(),
                MODEL_SCALE * SCALE_TWEAK,
                Color::GREEN,
            );
        });
    }
}
