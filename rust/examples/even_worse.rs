use asset_payload::SPHERE_PATH;
use bath::fixed_func::silhouette::{collect_deformed_mesh_samples, interpolate_between_deformed_meshes, FOVY};
use bath::fixed_func::silhouette::{ANGULAR_VELOCITY, MODEL_POS, MODEL_SCALE, SCALE_TWEAK};
use bath::fixed_func::texture::{dither, generate_silhouette_texture, generate_spherical_uvs, DitherStaging};
use bath::fixed_func::topology::{ensure_drawable, observed_line_of_sight};
use bath::render::raylib::RaylibRenderer;
use bath::render::raylib_util::N64_WIDTH;
use bath::render::renderer::Renderer;
use raylib::camera::Camera3D;
use raylib::color::Color;
use raylib::consts::CameraProjection;
use raylib::drawing::{RaylibDraw, RaylibDraw3D, RaylibMode3DExt};
use raylib::math::Vector3;


fn main() {
    let mut render = RaylibRenderer::init(N64_WIDTH, N64_WIDTH);
    let main_observer = Camera3D {
        position: Vector3::new(0.0, 0.0, 2.0),
        target: Vector3::ZERO,
        up: Vector3::Y,
        fovy: 2.0,
        projection: CameraProjection::CAMERA_ORTHOGRAPHIC,
    };
    let main_model = render.handle.load_model(&render.thread, SPHERE_PATH).unwrap();
    while !render.handle.window_should_close() {
        let mut draw_handle = render.handle.begin_drawing(&render.thread);
        draw_handle.clear_background(Color::BLACK);
        {
            let mut rl3d = draw_handle.begin_mode3D(main_observer);
            rl3d.draw_model_ex(
                &main_model,
                MODEL_POS,
                Vector3::Y,
                0.0,
                MODEL_SCALE,
                Color::WHITE,
            );
        }

    }
}
