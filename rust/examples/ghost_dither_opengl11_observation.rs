use asset_payload::SPHERE_PATH;
use bath::fixed_func::happo_giri_observer::{happo_giri_draw, happo_giri_setup};
use bath::fixed_func::silhouette::{collect_deformed_mesh_samples, interpolate_between_deformed_meshes, FOVY};
use bath::fixed_func::silhouette::{rotate_inverted_hull, ANGULAR_VELOCITY};
use bath::fixed_func::topology::{observed_line_of_sight, reverse_vertex_winding};
use bath::render::raylib::RaylibRenderer;
use bath::render::raylib_util::N64_WIDTH;
use bath::render::renderer::Renderer;
use raylib::camera::Camera3D;
use raylib::color::Color;
use raylib::consts::CameraProjection;
use raylib::drawing::RaylibDraw;
use raylib::math::Vector3;
use raylib::models::RaylibModel;

fn main() {
    let mut i_time = 0.0f32;
    let mut mesh_rotation = 0.0f32;
    let mut render = RaylibRenderer::init(N64_WIDTH * 2, N64_WIDTH);
    let main_observer = Camera3D {
        position: Vector3::new(0.0, 0.0, 2.0),
        target: Vector3::ZERO,
        up: Vector3::Y,
        fovy: FOVY,
        projection: CameraProjection::CAMERA_ORTHOGRAPHIC,
    };
    let (observers, labels) = happo_giri_setup();
    let mut wire_model = render.handle.load_model(&render.thread, SPHERE_PATH).unwrap();
    let mut main_model = render.handle.load_model(&render.thread, SPHERE_PATH).unwrap();
    let mut inverted_hull_model = render.handle.load_model(&render.thread, SPHERE_PATH).unwrap();

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
        happo_giri_draw(
            &mut draw_handle,
            &observers,
            &labels,
            4,
            2,
            &inverted_hull_model,
            &wire_model,
            mesh_rotation,
        );
    }
}
