use asset_payload::SPHERE_PATH;
use bath::fixed_func::silhouette_inverse_projection_util::{
    generate_mesh_and_texcoord_samples_from_silhouette, lerp_intermediate_mesh_samples_to_single_mesh,
    TIME_BETWEEN_SAMPLES,
};
use bath::geometry::unfold_mst::unfold_sphere_like;
use bath::geometry::weld_vertices::weld_and_index_mesh;
use bath::render::raylib::RaylibRenderer;
use bath::render::raylib_util::{MODEL_POS, MODEL_SCALE, N64_WIDTH};
use bath::render::renderer::Renderer;
use raylib::camera::Camera3D;
use raylib::color::Color;
use raylib::consts::CameraProjection;
use raylib::drawing::{RaylibDraw, RaylibDraw3D, RaylibMode3DExt};
use raylib::math::Vector3;
use raylib::models::RaylibModel;
use std::slice::from_raw_parts;

const DEBUG_COLORS_IDS: bool = true;

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
    let mut unfold = unfold_sphere_like(&mut wire_model.meshes_mut()[0]);
    let mut triangle_count = unfold.triangleCount as usize;
    let mut indices = unsafe { from_raw_parts(unfold.indices, triangle_count * 3) };
    let mut vertices = unsafe { from_raw_parts(unfold.vertices, unfold.vertexCount as usize * 3) };
    let mut unfolded_model = render
        .handle
        .load_model_from_mesh(&render.thread, unsafe { unfold.make_weak() })
        .unwrap();

    while !render.handle.window_should_close() {
        i_time += render.handle.get_frame_time();
        let duration = mesh_samples.len() as f32 * TIME_BETWEEN_SAMPLES;
        let time = i_time % duration;
        let frame = time / TIME_BETWEEN_SAMPLES;
        let current_frame = frame.floor() as usize % mesh_samples.len();
        //TODO: no idea why but some of the unfolds just jitter like mad, and some triangles glitch back and forth
        lerp_intermediate_mesh_samples_to_single_mesh(
            (current_frame as f32 * TIME_BETWEEN_SAMPLES).floor(),
            &mesh_samples,
            &texcoord_samples,
            &mut wire_model.meshes_mut()[0],
        );
        unfold = unfold_sphere_like(&mut wire_model.meshes_mut()[0]);
        indices = unsafe { from_raw_parts(unfold.indices, triangle_count * 3) };
        vertices = unsafe { from_raw_parts(unfold.vertices, unfold.vertexCount as usize * 3) };
        unfolded_model = render
            .handle
            .load_model_from_mesh(&render.thread, unsafe { unfold.make_weak() })
            .unwrap();
        let mut draw_handle = render.handle.begin_drawing(&render.thread);
        draw_handle.clear_background(Color::BLACK);
        let mut tri_idx_labels: Vec<(i32, i32, usize)> = Vec::with_capacity(triangle_count);
        {
            let mut rl3d = draw_handle.begin_mode3D(observer);
            rl3d.draw_model_wires_ex(&unfolded_model, MODEL_POS, Vector3::Y, 0.0, MODEL_SCALE, Color::WHITE);
            if DEBUG_COLORS_IDS {
                for triangle_index in 0..triangle_count {
                    let ia = indices[triangle_index * 3] as usize;
                    let ib = indices[triangle_index * 3 + 1] as usize;
                    let ic = indices[triangle_index * 3 + 2] as usize;

                    let pa = Vector3::new(vertices[ia * 3], vertices[ia * 3 + 1], vertices[ia * 3 + 2]);
                    let pb = Vector3::new(vertices[ib * 3], vertices[ib * 3 + 1], vertices[ib * 3 + 2]);
                    let pc = Vector3::new(vertices[ic * 3], vertices[ic * 3 + 1], vertices[ic * 3 + 2]);

                    let pa2 = Vector3::new(pa.x, pa.y, pa.z);
                    let pb2 = Vector3::new(pb.x, pb.y, pb.z);
                    let pc2 = Vector3::new(pc.x, pc.y, pc.z);

                    let color = Color::new(
                        (triangle_index.wrapping_mul(66) & 255) as u8,
                        (triangle_index.wrapping_mul(124) & 255) as u8,
                        (triangle_index.wrapping_mul(199) & 255) as u8,
                        255,
                    );
                    rl3d.draw_triangle3D(pa2, pb2, pc2, color);
                    let centroid = (pa + pb + pc) / 3.0;
                    let x = ((centroid.x) * 0.5 + 0.5) * screen_w as f32;
                    let y = ((-centroid.y) * 0.5 + 0.5) * screen_h as f32;
                    tri_idx_labels.push((x as i32, y as i32, triangle_index));
                    //TODO: why the hell does 3D fuck this up
                    // rl3d.draw_text(&triangle_index.to_string(), x as i32, y as i32, 10, Color::YELLOW);
                }
                // for (x, y, triangle_index) in tri_idx_labels {
                // rl3d.draw_text_pro(Font::default(), &triangle_index.to_string(), Vec2::new(x, y), 10, Color::WHITE);
                // draw_handle.draw_text(&triangle_index.to_string(), x, y, 10, Color::WHITE);
                // }
            }
        }
        for (x, y, triangle_index) in tri_idx_labels {
            draw_handle.draw_text(&triangle_index.to_string(), x, y, 10, Color::WHITE);
        }
    }
}
