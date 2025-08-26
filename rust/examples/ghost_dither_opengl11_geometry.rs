use asset_payload::SPHERE_PATH;
use bath::fixed_func::papercraft::fold;
use bath::fixed_func::silhouette::generate_mesh_and_texcoord_samples_from_silhouette;
use bath::fixed_func::silhouette_constants::{ANGULAR_VELOCITY, MODEL_POS, MODEL_SCALE, TIME_BETWEEN_SAMPLES};
use bath::fixed_func::silhouette_interpolation::{
    interpolate_mesh_and_texcoord_samples, lerp_intermediate_mesh_samples_to_single_mesh,
};
use bath::fixed_func::silhouette_util::debug_indices;
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
    let mut mesh_rotation = 0.0f32;
    let mut render = RaylibRenderer::init(N64_WIDTH, N64_WIDTH);
    let observer = Camera3D {
        position: Vector3::new(0.0, 0.0, 2.0),
        target: Vector3::ZERO,
        up: Vector3::Y,
        fovy: 2.0,
        projection: CameraProjection::CAMERA_ORTHOGRAPHIC,
    };
    let mut wire_model = render.handle.load_model(&render.thread, SPHERE_PATH).unwrap();
    let (mesh_samples, texcoord_samples) = generate_mesh_and_texcoord_samples_from_silhouette(&mut render);
    lerp_intermediate_mesh_samples_to_single_mesh(
        i_time,
        &mesh_samples,
        &texcoord_samples,
        &mut wire_model.meshes_mut()[0],
    );
    while !render.handle.window_should_close() {
        i_time += render.handle.get_frame_time();
        mesh_rotation -= ANGULAR_VELOCITY * render.handle.get_frame_time();

        let duration = mesh_samples.len() as f32 * TIME_BETWEEN_SAMPLES;
        let time = i_time % duration;
        let frame = time / TIME_BETWEEN_SAMPLES;
        let current_frame = frame.floor() as usize % mesh_samples.len();
        interpolate_mesh_and_texcoord_samples(
            &mut wire_model,
            i_time,
            // (current_frame as f32 * TIME_BETWEEN_SAMPLES).floor(),
            &mesh_samples,
            &texcoord_samples,
        );
        let unfolded_mesh = unsafe { fold(&mut wire_model.meshes_mut()[0], i_time, true).make_weak() };
        // let unfolded_mesh = unsafe { unfold(&mut wire_model.meshes_mut()[0]).make_weak() };
        let unfolded_model = render
            .handle
            .load_model_from_mesh(&render.thread, unfolded_mesh.clone())
            .unwrap();
        let mut draw_handle = render.handle.begin_drawing(&render.thread);
        draw_handle.clear_background(Color::BLACK);
        {
            let mut rl3d = draw_handle.begin_mode3D(observer);
            rl3d.draw_model_wires_ex(&unfolded_model, MODEL_POS, Vector3::Y, 0.0, MODEL_SCALE, Color::WHITE);
        }
        debug_indices(observer, &mut draw_handle, &unfolded_mesh, 0.0);
    }
}
