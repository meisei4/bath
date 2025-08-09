use asset_payload::SPHERE_PATH;
use bath::fixed_func::silhouette_inverse_projection_util::{
    generate_mesh_and_texcoord_samples_from_silhouette, interpolate_mesh_and_texcoord_samples,
    lerp_intermediate_mesh_samples_to_single_mesh, TIME_BETWEEN_SAMPLES,
};
use bath::geometry::papercraft::unfold_sphere_like;
use bath::geometry::welding::weld_and_index_mesh;
use bath::render::raylib::RaylibRenderer;
use bath::render::raylib_util::{MODEL_POS, MODEL_SCALE, N64_WIDTH};
use bath::render::renderer::Renderer;
use raylib::camera::Camera3D;
use raylib::color::Color;
use raylib::consts::CameraProjection;
use raylib::drawing::{RaylibDraw, RaylibDraw3D, RaylibDrawHandle, RaylibMode3DExt};
use raylib::math::Vector3;
use raylib::models::{RaylibModel, WeakMesh};
use std::slice::from_raw_parts;

fn main() {
    let mut i_time = 0.0f32;
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
    let mut wire_model = render.handle.load_model(&render.thread, SPHERE_PATH).unwrap();
    let (mesh_samples, texcoord_samples) =
        generate_mesh_and_texcoord_samples_from_silhouette(screen_w, screen_h, &mut render);
    lerp_intermediate_mesh_samples_to_single_mesh(
        i_time,
        &mesh_samples,
        &texcoord_samples,
        &mut wire_model.meshes_mut()[0],
    );
    weld_and_index_mesh(&mut wire_model.meshes_mut()[0], 1e-6);
    while !render.handle.window_should_close() {
        i_time += render.handle.get_frame_time();
        let duration = mesh_samples.len() as f32 * TIME_BETWEEN_SAMPLES;
        let time = i_time % duration;
        let frame = time / TIME_BETWEEN_SAMPLES;
        let current_frame = frame.floor() as usize % mesh_samples.len();
        //TODO: no idea why but some of the unfolds just jitter like mad, and some triangles glitch back and forth
        interpolate_mesh_and_texcoord_samples(
            &mut wire_model,
            (current_frame as f32 * TIME_BETWEEN_SAMPLES).floor(),
            &mesh_samples,
            &texcoord_samples,
        );
        let unfolded_mesh = unsafe { unfold_sphere_like(&mut wire_model.meshes_mut()[0]).make_weak() };
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
        debug_papercraft(observer, &mut draw_handle, &unfolded_mesh);
    }
}

pub fn debug_papercraft(observer: Camera3D, draw_handle: &mut RaylibDrawHandle, unfolded_mesh: &WeakMesh) {
    let triangle_count = unfolded_mesh.triangleCount as usize;
    let indices = unsafe { from_raw_parts(unfolded_mesh.indices, triangle_count * 3) };
    let vertices = unsafe { from_raw_parts(unfolded_mesh.vertices, unfolded_mesh.vertexCount as usize * 3) };
    let screen_w = draw_handle.get_screen_width();
    let screen_h = draw_handle.get_screen_height();
    for i in 0..triangle_count {
        let ia = indices[i * 3] as usize;
        let ib = indices[i * 3 + 1] as usize;
        let ic = indices[i * 3 + 2] as usize;

        let pa = Vector3::new(vertices[ia * 3], vertices[ia * 3 + 1], vertices[ia * 3 + 2]);
        let pb = Vector3::new(vertices[ib * 3], vertices[ib * 3 + 1], vertices[ib * 3 + 2]);
        let pc = Vector3::new(vertices[ic * 3], vertices[ic * 3 + 1], vertices[ic * 3 + 2]);

        let pa2 = Vector3::new(pa.x, pa.y, pa.z);
        let pb2 = Vector3::new(pb.x, pb.y, pb.z);
        let pc2 = Vector3::new(pc.x, pc.y, pc.z);

        let color = Color::new(
            (i.wrapping_mul(66) & 255) as u8,
            (i.wrapping_mul(124) & 255) as u8,
            (i.wrapping_mul(199) & 255) as u8,
            255,
        );
        let centroid = (pa + pb + pc) / 3.0;
        let x = ((centroid.x) * 0.5 + 0.5) * screen_w as f32;
        let y = ((-centroid.y) * 0.5 + 0.5) * screen_h as f32;
        {
            let mut rl3d = draw_handle.begin_mode3D(observer);
            rl3d.draw_triangle3D(pa2, pb2, pc2, color);
        }
        draw_handle.draw_text(&i.to_string(), x as i32, y as i32, 10, Color::WHITE);
    }
}
