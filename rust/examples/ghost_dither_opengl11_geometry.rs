use asset_payload::SPHERE_PATH;
use bath::fixed_func::ghost::{
    build_radial_magnitudes, deform_mesh_by_phase, deform_mesh_by_silhouette_radii, generate_silhouette_image,
    precompute_thetas,
};
use bath::geometry::unfold_mst::unfold_sphere_like;
use bath::geometry::weld_vertices::weld_and_index_mesh;
use bath::render::raylib::RaylibRenderer;
use bath::render::raylib_util::N64_WIDTH;
use bath::render::renderer::Renderer;
use raylib::camera::Camera3D;
use raylib::color::Color;
use raylib::consts::CameraProjection;
use raylib::consts::TextureWrap::TEXTURE_WRAP_CLAMP;
use raylib::drawing::{RaylibDraw, RaylibDraw3D, RaylibMode3DExt};
use raylib::math::{Vector2, Vector3};
use raylib::models::{RaylibMesh, RaylibModel};
use raylib::prelude::{Image, RaylibTexture2D};
use std::f32::consts::TAU;
use std::slice::{from_raw_parts, from_raw_parts_mut};

fn main() {
    let mut render = RaylibRenderer::init(N64_WIDTH, N64_WIDTH);
    let screen_w = render.handle.get_screen_width();
    let screen_h = render.handle.get_screen_height();
    let i_resolution = Vector2::new(screen_w as f32, screen_h as f32);
    let mut i_time = 0.0f32;
    let mut time_since_update = 0.0f32;

    let circle_img = generate_silhouette_image(screen_w, screen_h, i_time);
    let total_bytes = (screen_w * screen_h * 4) as usize;
    let pixels: &mut [u8] = unsafe { from_raw_parts_mut(circle_img.data as *mut u8, total_bytes) };
    let mut texture = render
        .handle
        .load_texture_from_image(&render.thread, &circle_img)
        .unwrap();
    let mut wire_model = render.handle.load_model(&render.thread, SPHERE_PATH).unwrap();
    let observer = Camera3D {
        position: Vector3::new(0.0, 0.0, 2.0),
        target: Vector3::ZERO,
        up: Vector3::Y,
        fovy: 2.0,
        projection: CameraProjection::CAMERA_ORTHOGRAPHIC,
    };
    let model_pos = Vector3::ZERO;
    let model_scale = Vector3::ONE;

    let mesh_slice = wire_model.meshes_mut();
    let mesh = &mut mesh_slice[0];
    weld_and_index_mesh(mesh, 1e-6);
    println!("vertexCount = {}", mesh.vertexCount);
    println!("triangleCount = {}", mesh.triangleCount);
    println!("after welding: vx={}, tri={}", mesh.vertexCount, mesh.triangleCount);
    let unfold = unfold_sphere_like(mesh);
    println!(
        "after unfolding: vx={}, tri={}",
        unfold.vertexCount, unfold.triangleCount
    );
    let tri_count = unfold.triangleCount as usize;
    let indices = unsafe { from_raw_parts(unfold.indices, tri_count * 3) };
    let verts = unsafe { from_raw_parts(unfold.vertices, unfold.vertexCount as usize * 3) };
    let unfolded_model = render
        .handle
        .load_model_from_mesh(&render.thread, unsafe { unfold.make_weak() })
        .unwrap();
    let debug_show_ids = false;

    let mut tile_img = Image::gen_image_color(8, 8, Color::BLANK);
    tile_img.set_format(raylib::consts::PixelFormat::PIXELFORMAT_UNCOMPRESSED_GRAY_ALPHA);
    let mut tile_texture = render
        .handle
        .load_texture_from_image(&render.thread, &tile_img)
        .unwrap();
    tile_texture.set_texture_wrap(&render.thread, TEXTURE_WRAP_CLAMP);
    let original_verts = wire_model.meshes()[0].vertices().to_vec();
    let radial_magnitudes = build_radial_magnitudes(&circle_img); //just testing the initial state compared with the silhhoute at i_time = 0
    deform_mesh_by_silhouette_radii(&mut wire_model.meshes_mut()[0], &radial_magnitudes);
    while !render.handle.window_should_close() {
        i_time += render.handle.get_frame_time();
        // let rotation_angle = i_time * 90.0;
        let rotation_angle = 0.0;
        let mut draw_handle = render.handle.begin_drawing(&render.thread);
        draw_handle.clear_background(Color::BLACK);
        let mut labels: Vec<(i32, i32, usize)> = Vec::with_capacity(tri_count);
        {
            time_since_update += i_time;
            if time_since_update >= 0.2 {
                let circle_img = generate_silhouette_image(screen_w, screen_h, i_time);
                let radial = build_radial_magnitudes(&circle_img);
                deform_mesh_by_silhouette_radii(&mut wire_model.meshes_mut()[0], &radial);

                wire_model.meshes_mut()[0]
                    .vertices_mut()
                    .copy_from_slice(&original_verts);
                let yaw_rad = -i_time * 90.0f32.to_radians();
                // deform_mesh_with_yaw(&mut wire_model.meshes_mut()[0], &radial, yaw_rad);

                const OMEGA: f32 = TAU * 0.25;
                let r0 = build_radial_magnitudes(&circle_img);
                let base_vertices = wire_model.meshes()[0].vertices().to_vec();
                let thetas = precompute_thetas(&wire_model.meshes()[0]);
                let phase = (OMEGA * i_time) % TAU;
                wire_model.meshes_mut()[0]
                    .vertices_mut()
                    .copy_from_slice(&base_vertices);
                deform_mesh_by_phase(&mut wire_model.meshes_mut()[0], &r0.try_into().unwrap(), &thetas, phase);
                time_since_update = 0.0;
            }

            let mut rl3d = draw_handle.begin_mode3D(observer);
            rl3d.draw_model_wires_ex(
                &wire_model,
                model_pos,
                Vector3::Y,
                rotation_angle,
                model_scale,
                Color::WHITE,
            );
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
}
