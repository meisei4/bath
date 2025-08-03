use asset_payload::SPHERE_PATH;
use bath::render::raylib::RaylibRenderer;
use bath::render::raylib_util::N64_WIDTH;
use bath::render::renderer::Renderer;
use raylib::camera::Camera3D;
use raylib::color::Color;
use raylib::consts::CameraProjection;

use bath::fixed_func::silhouette::{
    build_radial_magnitudes, deform_mesh_by_silhouette_radii, generate_silhouette_image, ANGULAR_VELOCITY, MODEL_POS,
    MODEL_SCALE,
};
use raylib::drawing::{RaylibDraw, RaylibDraw3D, RaylibMode3DExt};
use raylib::math::Vector3;
use raylib::models::{RaylibMesh, RaylibModel};

const DT: f32 = 0.1;
const NUM_SAMPLES: usize = 80;
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
    let mut model = render.handle.load_model(&render.thread, SPHERE_PATH).unwrap();
    let silhouette_img = generate_silhouette_image(screen_w, screen_h, i_time);
    let silhouette_radii = build_radial_magnitudes(&silhouette_img);
    deform_mesh_by_silhouette_radii(&mut model.meshes_mut()[0], &silhouette_radii);
    let mut sample_vertices = Vec::with_capacity(NUM_SAMPLES);
    for i in 0..NUM_SAMPLES {
        let sample_time = i as f32 * DT;
        let sample_yaw = -sample_time * ANGULAR_VELOCITY;
        let silhouette_img = generate_silhouette_image(screen_w, screen_h, sample_time);
        let silhouette_radii = build_radial_magnitudes(&silhouette_img);
        let mut model_i = render.handle.load_model(&render.thread, SPHERE_PATH).unwrap();
        {
            let vertices = model_i.meshes_mut()[0].vertices_mut();
            for vertex in vertices.iter_mut() {
                let x0 = vertex.x;
                let z0 = vertex.z;
                vertex.x = sample_yaw.cos() * x0 + sample_yaw.sin() * z0;
                vertex.z = -sample_yaw.sin() * x0 + sample_yaw.cos() * z0;
            }
        }
        deform_mesh_by_silhouette_radii(&mut model_i.meshes_mut()[0], &silhouette_radii);
        let sample_vertices_i: Vec<Vector3> = model_i.meshes()[0].vertices().iter().cloned().collect();
        sample_vertices.push(sample_vertices_i);
    }

    while !render.handle.window_should_close() {
        i_time += render.handle.get_frame_time();
        // mesh_rotation += ANGULAR_VELOCITY * render.handle.get_frame_time();
        mesh_rotation = 0.0;
        let sample_idx = ((i_time / DT).floor() as usize) % NUM_SAMPLES;
        let sample_vertices_src = &sample_vertices[sample_idx];
        let sample_vertices_dst = model.meshes_mut()[0].vertices_mut();
        for (dst, src) in sample_vertices_dst.iter_mut().zip(sample_vertices_src.iter()) {
            *dst = *src;
        }
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
