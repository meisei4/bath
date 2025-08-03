use asset_payload::SPHERE_PATH;
use bath::render::raylib::RaylibRenderer;
use bath::render::raylib_util::N64_WIDTH;
use bath::render::renderer::Renderer;
use raylib::camera::Camera3D;
use raylib::color::Color;
use raylib::consts::CameraProjection;

use bath::fixed_func::silhouette::{
    bake_samples_for_single_mesh_no_rotation, blend_into_mesh, ANGULAR_VELOCITY, MODEL_POS, MODEL_SCALE,
};
use raylib::drawing::{RaylibDraw, RaylibDraw3D, RaylibMode3DExt};
use raylib::math::Vector3;
use raylib::models::RaylibModel;

const DT: f32 = 0.1;
const NUM_SAMPLES: usize = 40;
fn main() {
    let mut i_time = 0.0f32;
    let mut mesh_rotation = 0.0f32;
    let mut render = RaylibRenderer::init(N64_WIDTH, N64_WIDTH);
    let observer = Camera3D {
        position: Vector3::new(0.0, 0.0, 2.0),
        target: Vector3::ZERO,
        up: Vector3::Y,
        fovy: 2.0,
        projection: CameraProjection::CAMERA_ORTHOGRAPHIC,
    };
    let screen_w = render.handle.get_screen_width();
    let screen_h = render.handle.get_screen_height();
    let sample_vertices = bake_samples_for_single_mesh_no_rotation(DT, NUM_SAMPLES, screen_w, screen_h, &mut render);
    let mut model = render.handle.load_model(&render.thread, SPHERE_PATH).unwrap();
    while !render.handle.window_should_close() {
        i_time += render.handle.get_frame_time();
        mesh_rotation -= ANGULAR_VELOCITY * render.handle.get_frame_time();
        // mesh_rotation = 0.0;
        blend_into_mesh(&mut model.meshes_mut()[0], &sample_vertices, i_time, DT);
        let mut draw_handle = render.handle.begin_drawing(&render.thread);
        draw_handle.clear_background(Color::BLACK);
        let mut rl3d = draw_handle.begin_mode3D(observer);
        rl3d.draw_model_wires_ex(
            &model,
            MODEL_POS,
            Vector3::Y,
            mesh_rotation.to_degrees(),
            MODEL_SCALE,
            Color::WHITE,
        );
    }
}
