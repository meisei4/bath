use asset_payload::SPHERE_PATH;
use bath::fixed_func::papercraft::unfold;
use bath::fixed_func::silhouette::{collect_deformed_mesh_samples, FOVY};
use bath::fixed_func::silhouette::{interpolate_between_deformed_meshes, MODEL_POS, MODEL_SCALE};
use bath::fixed_func::silhouette::{ANGULAR_VELOCITY, TIME_BETWEEN_SAMPLES};
use bath::fixed_func::topology::debug_draw_triangles;
use bath::render::raylib::RaylibRenderer;
use bath::render::raylib_util::N64_WIDTH;
use bath::render::renderer::Renderer;
use raylib::camera::Camera3D;
use raylib::color::Color;
use raylib::consts::CameraProjection;
use raylib::drawing::{RaylibDraw, RaylibDraw3D, RaylibMode3DExt};
use raylib::math::Vector3;
use raylib::models::RaylibModel;

fn main() {
    let mut i_time = 0.0f32;
    let mut _mesh_rotation = 0.0f32;
    let mut render = RaylibRenderer::init(N64_WIDTH, N64_WIDTH);

    let observer = Camera3D {
        position: Vector3::new(0.0, 0.0, 2.0),
        target: Vector3::ZERO,
        up: Vector3::Y,
        fovy: FOVY,
        projection: CameraProjection::CAMERA_ORTHOGRAPHIC,
    };
    let mut wire_model = render.handle.load_model(&render.thread, SPHERE_PATH).unwrap();
    //TODO: this is still ugly here:
    let mesh_samples = collect_deformed_mesh_samples(&mut render);
    interpolate_between_deformed_meshes(&mut wire_model, i_time, &mesh_samples);
    while !render.handle.window_should_close() {
        i_time += render.handle.get_frame_time();
        _mesh_rotation -= ANGULAR_VELOCITY * render.handle.get_frame_time();
        let duration = mesh_samples.len() as f32 * TIME_BETWEEN_SAMPLES;
        let time = i_time % duration;
        let frame = time / TIME_BETWEEN_SAMPLES;
        let _current_frame = frame.floor() as usize % mesh_samples.len();
        interpolate_between_deformed_meshes(
            &mut wire_model,
            i_time,
            // (_current_frame as f32 * TIME_BETWEEN_SAMPLES).floor(),
            &mesh_samples,
        );
        // let unfolded_mesh = unsafe { fold(&render.thread, &mut wire_model.meshes_mut()[0], i_time, true).make_weak() };
        let unfolded_mesh = unsafe { unfold(&render.thread, &mut wire_model.meshes_mut()[0]).make_weak() };
        let unfolded_model = render
            .handle
            .load_model_from_mesh(&render.thread, unfolded_mesh.clone())
            .unwrap();
        let mut draw_handle = render.handle.begin_drawing(&render.thread);
        draw_handle.clear_background(Color::BLACK);
        draw_handle.draw_mode3D(observer, |mut rl3d| {
            rl3d.draw_model_wires_ex(&unfolded_model, MODEL_POS, Vector3::Y, 0.0, MODEL_SCALE, Color::WHITE);
        });

        let all_triangles: Vec<usize> = (0..unfolded_mesh.triangleCount as usize).collect();
        debug_draw_triangles(
            observer,
            &mut draw_handle,
            &unfolded_mesh,
            // mesh_rotation,
            0.0,
            &*all_triangles,
            None,
            true,
        );
    }
}
