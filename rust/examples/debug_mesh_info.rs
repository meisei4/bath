use asset_payload::SPHERE_PATH;
use bath::fixed_func::silhouette::{MODEL_POS, MODEL_SCALE};
use bath::fixed_func::topology::ensure_drawable;
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
    let mut render = RaylibRenderer::init(N64_WIDTH, N64_WIDTH);
    let main_observer = Camera3D {
        position: Vector3::new(0.0, 0.0, 2.0),
        target: Vector3::ZERO,
        up: Vector3::Y,
        fovy: 2.0,
        projection: CameraProjection::CAMERA_ORTHOGRAPHIC,
    };

    let mut main_model = render.handle.load_model(&render.thread, SPHERE_PATH).unwrap();
    let mut wire_model = render.handle.load_model(&render.thread, SPHERE_PATH).unwrap();
    ensure_drawable(&mut wire_model.meshes_mut()[0]);
    ensure_drawable(&mut main_model.meshes_mut()[0]);

    for (i, m) in main_model.meshes_mut().iter_mut().enumerate() {
        println!(
            "mesh[{i}] vc={} tc={} ptrs: v={:?} n={:?} t={:?} idx={:?} animV={:?} animN={:?}",
            m.vertexCount,
            m.triangleCount,
            m.vertices,
            m.normals,
            m.texcoords,
            m.indices,
            m.animVertices,
            m.animNormals
        );
    }
    while !render.handle.window_should_close() {
        let mut draw_handle = render.handle.begin_drawing(&render.thread);
        draw_handle.clear_background(Color::BLACK);
        draw_handle.draw_mode3D(main_observer, |mut rl3d| {
            rl3d.draw_model_ex(&main_model, MODEL_POS, Vector3::Y, 0.0, MODEL_SCALE, Color::WHITE);
            rl3d.draw_model_wires_ex(&wire_model, MODEL_POS, Vector3::Y, 0.0, MODEL_SCALE, Color::BLACK);
        });
    }
}
