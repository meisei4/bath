use asset_payload::SPHERE_PATH;
use bath::fixed_func::silhouette::ANGULAR_VELOCITY;
use bath::fixed_func::silhouette::{
    collect_deformed_mesh_samples, draw_inverted_hull_guassian_silhouette_stack, interpolate_between_deformed_meshes,
    rotate_inverted_hull, FOVY,
};
use bath::fixed_func::topology::{ensure_drawable, observed_line_of_sight, reverse_vertex_winding};
use bath::render::raylib::RaylibRenderer;
use bath::render::raylib_util::N64_WIDTH;
use bath::render::renderer::Renderer;
use raylib::camera::Camera3D;
use raylib::color::Color;
use raylib::consts::CameraProjection;
use raylib::drawing::{RaylibDraw, RaylibMode3DExt};
use raylib::math::Vector3;
use raylib::models::RaylibModel;

fn main() {
    let mut i_time = 0.0f32;
    let mut mesh_rotation = 0.0f32;
    let mut render = RaylibRenderer::init(N64_WIDTH, N64_WIDTH);
    let main_observer = Camera3D {
        position: Vector3::new(0.0, 0.0, 2.0),
        target: Vector3::ZERO,
        up: Vector3::Y,
        fovy: FOVY,
        projection: CameraProjection::CAMERA_ORTHOGRAPHIC,
    };

    let mut wire_model = render.handle.load_model(&render.thread, SPHERE_PATH).unwrap();
    let mut main_model = render.handle.load_model(&render.thread, SPHERE_PATH).unwrap();
    let mut inverted_hull_model = render.handle.load_model(&render.thread, SPHERE_PATH).unwrap();

    ensure_drawable(&mut wire_model.meshes_mut()[0]);
    ensure_drawable(&mut main_model.meshes_mut()[0]);
    ensure_drawable(&mut inverted_hull_model.meshes_mut()[0]);
    reverse_vertex_winding(&mut inverted_hull_model.meshes_mut()[0]);

    let mesh_samples = collect_deformed_mesh_samples(&mut render);
    interpolate_between_deformed_meshes(&mut wire_model, i_time, &mesh_samples);
    interpolate_between_deformed_meshes(&mut main_model, i_time, &mesh_samples);
    while !render.handle.window_should_close() {
        i_time += render.handle.get_frame_time();
        mesh_rotation -= ANGULAR_VELOCITY * render.handle.get_frame_time();
        interpolate_between_deformed_meshes(&mut wire_model, i_time, &mesh_samples);
        interpolate_between_deformed_meshes(&mut main_model, i_time, &mesh_samples);
        let observed_line_of_sight = observed_line_of_sight(&main_observer);
        rotate_inverted_hull(
            &main_model,
            &mut inverted_hull_model,
            observed_line_of_sight,
            mesh_rotation,
        );
        let mut draw_handle = render.handle.begin_drawing(&render.thread);
        draw_handle.clear_background(Color::BLACK);
        let mut rl3d = draw_handle.begin_mode3D(main_observer);
        draw_inverted_hull_guassian_silhouette_stack(&mut rl3d, &inverted_hull_model, mesh_rotation);
        // rl3d.draw_model_ex(
        //     &main_model,
        //     MODEL_POS,
        //     Vector3::Y,
        //     mesh_rotation.to_degrees(),
        //     MODEL_SCALE * SCALE_TWEAK,
        //     Color::WHITE,
        // );
        // rl3d.draw_model_wires_ex(
        //     &wire_model,
        //     MODEL_POS,
        //     Vector3::Y,
        //     mesh_rotation.to_degrees(),
        //     MODEL_SCALE * SCALE_TWEAK,
        //     Color::BLACK,
        // );
    }
}
