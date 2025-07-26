use asset_payload::SPHERE_PATH;
use bath::fixed_func::ghost::{
    build_radial_magnitudes, deform_mesh_by_radial_magnitudes, generate_circle_image_no_dither,
};
use bath::geometry::unfold_mst::unfold_sphere_like;
use bath::geometry::weld_vertices::weld_and_index_mesh;
use bath::render::raylib::RaylibRenderer;
use bath::render::raylib_util::N64_WIDTH;
use bath::render::renderer::Renderer;
use raylib::camera::Camera3D;
use raylib::color::Color;
use raylib::consts::CameraProjection;
use raylib::drawing::{RaylibDraw, RaylibDraw3D, RaylibMode3DExt};
use raylib::math::{Vector2, Vector3};
use raylib::models::RaylibModel;
use std::slice::from_raw_parts_mut;

fn main() {
    let mut render = RaylibRenderer::init(N64_WIDTH, N64_WIDTH);
    let screen_w = render.handle.get_screen_width();
    let screen_h = render.handle.get_screen_height();
    let i_resolution = Vector2::new(screen_w as f32, screen_h as f32);
    let mut i_time = 0.0f32;

    let circle_img = generate_circle_image_no_dither(screen_w, screen_h, i_time);
    let total_bytes = (screen_w * screen_h * 4) as usize;
    let pixels: &mut [u8] = unsafe { from_raw_parts_mut(circle_img.data as *mut u8, total_bytes) };
    let mut texture = render
        .handle
        .load_texture_from_image(&render.thread, &circle_img)
        .unwrap();
    let mut model = render.handle.load_model(&render.thread, SPHERE_PATH).unwrap();
    let observer = Camera3D {
        position: Vector3::new(0.0, 0.0, 2.0),
        target: Vector3::ZERO,
        up: Vector3::Y,
        fovy: 2.0,
        projection: CameraProjection::CAMERA_ORTHOGRAPHIC,
    };
    let model_pos = Vector3::ZERO;
    let model_scale = Vector3::ONE;
    let radial_magnitudes = build_radial_magnitudes(&circle_img); //just testing the initial state compared with the silhhoute at i_time = 0
    deform_mesh_by_radial_magnitudes(&mut model.meshes_mut()[0], &radial_magnitudes);
    let mesh_slice = model.meshes_mut();
    let mesh = &mut mesh_slice[0];
    weld_and_index_mesh(mesh, 1e-6);
    println!("vertexCount = {}", mesh.vertexCount);
    println!("triangleCount = {}", mesh.triangleCount);
    println!(
        "after welding  : vtx = {}, tri = {}",
        mesh.vertexCount, mesh.triangleCount
    );

    let unfolded_mesh = unfold_sphere_like(mesh);
    println!(
        "after unfolding: vtx = {}, tri = {}",
        unfolded_mesh.vertexCount, unfolded_mesh.triangleCount
    );
    let idx = unsafe { std::slice::from_raw_parts(mesh.indices, mesh.triangleCount as usize * 3) };
    let mut deg = 0;
    for f in 0..mesh.triangleCount as usize {
        let a = idx[f * 3];
        let b = idx[f * 3 + 1];
        let c = idx[f * 3 + 2];
        if a == b || b == c || c == a {
            deg += 1;
        }
    }
    println!("degenerate triangles = {deg}");
    let tri_count = unfolded_mesh.triangleCount as usize;
    let indices = unsafe { std::slice::from_raw_parts(unfolded_mesh.indices, tri_count * 3) };
    let verts = unsafe { std::slice::from_raw_parts(unfolded_mesh.vertices, unfolded_mesh.vertexCount as usize * 3) };
    let unfolded_model = render
        .handle
        .load_model_from_mesh(&render.thread, unsafe { unfolded_mesh.make_weak() })
        .unwrap();
    let debug_show_ids = false;
    while !render.handle.window_should_close() {
        i_time += render.handle.get_frame_time();
        let mut draw_handle = render.handle.begin_drawing(&render.thread);
        draw_handle.clear_background(Color::BLACK);
        let mut labels: Vec<(i32, i32, usize)> = Vec::with_capacity(tri_count);
        {
            let mut rl3d = draw_handle.begin_mode3D(observer);
            // rl3d.draw_model_wires_ex(
            //     &model,
            //     model_pos,
            //     Vector3::Y,
            //     i_time * 90.0,
            //     model_scale,
            //     Color::WHITE,
            // );
            rl3d.draw_model_wires_ex(&unfolded_model, model_pos, Vector3::Y, 0.0, model_scale, Color::WHITE);

            // rl3d.draw_model_wires_ex(&unfolded_model, model_pos, Vector3::Y, 0.0, model_scale, Color::WHITE);
            if debug_show_ids {
                let dz = 0.00005;
                for f in 0..tri_count {
                    let ia = indices[f * 3] as usize;
                    let ib = indices[f * 3 + 1] as usize;
                    let ic = indices[f * 3 + 2] as usize;

                    let pa = Vector3::new(verts[ia * 3], verts[ia * 3 + 1], verts[ia * 3 + 2]);
                    let pb = Vector3::new(verts[ib * 3], verts[ib * 3 + 1], verts[ib * 3 + 2]);
                    let pc = Vector3::new(verts[ic * 3], verts[ic * 3 + 1], verts[ic * 3 + 2]);

                    let pa2 = Vector3::new(pa.x, pa.y, pa.z + dz);
                    let pb2 = Vector3::new(pb.x, pb.y, pb.z + dz);
                    let pc2 = Vector3::new(pc.x, pc.y, pc.z + dz);

                    let color = Color::new(
                        (f.wrapping_mul(73) & 255) as u8,
                        (f.wrapping_mul(151) & 255) as u8,
                        (f.wrapping_mul(199) & 255) as u8,
                        255,
                    );
                    rl3d.draw_triangle3D(pa2, pb2, pc2, color);
                    if debug_show_ids {
                        let centroid = (pa + pb + pc) / 3.0;
                        let sx = ((centroid.x) * 0.5 + 0.5) * screen_w as f32;
                        let sy = ((-centroid.y) * 0.5 + 0.5) * screen_h as f32;
                        labels.push((sx as i32, sy as i32, f));
                    }
                }
            }
        }

        if debug_show_ids {
            for (sx, sy, f) in labels {
                draw_handle.draw_text(&f.to_string(), sx, sy, 10, Color::YELLOW);
            }
        }
    }
    // for mesh in model.meshes_mut() {
    //     for vertex in mesh.vertices_mut() {
    //         vertex.y += (vertex.x * 2.0 + i_time * 2.0).sin() * 0.015;
    //     }
    // }
}
