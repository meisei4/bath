use asset_payload::SPHERE_PATH;
use bath::fixed_func::ghost::{
    build_radial_magnitudes, build_radial_magnitudes_fast, deform_mesh_by_radial_magnitudes_fast,
    deform_mesh_by_silhouette_radii, generate_circle_image_no_dither_fast, generate_silhouette_image,
    map_mesh_vertices_to_silhouette_texcoords, precompute_thetas, update_texcoords, RADIAL_SAMPLE_COUNT,
};

use bath::render::raylib::RaylibRenderer;
use bath::render::raylib_util::N64_WIDTH;
use bath::render::renderer::Renderer;
use raylib::camera::Camera3D;
use raylib::color::Color;
use raylib::consts::CameraProjection;
use raylib::consts::MaterialMapIndex::MATERIAL_MAP_ALBEDO;
use raylib::consts::PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8;
use raylib::consts::TextureWrap::TEXTURE_WRAP_CLAMP;
use raylib::drawing::{RaylibDraw, RaylibDraw3D, RaylibMode3DExt};
use raylib::math::Vector3;
use raylib::models::{RaylibMaterial, RaylibMesh, RaylibModel};
use raylib::prelude::Image;
use raylib::texture::RaylibTexture2D;
use std::f32::consts::TAU;
use std::slice::{from_raw_parts, from_raw_parts_mut};

fn main() {
    let mut render = RaylibRenderer::init(N64_WIDTH, N64_WIDTH);
    let screen_w = render.handle.get_screen_width();
    let screen_h = render.handle.get_screen_height();
    let mut i_time = 0.0f32;
    let mut mesh_rotation = 0.0f32;

    let circle_img_2d = generate_silhouette_image(screen_w, screen_h, i_time);
    // let texture_2d = render.handle.load_texture_from_image(&render.thread, &circle_img).unwrap();
    let radial_magnitudes_2d = build_radial_magnitudes(&circle_img_2d);

    let mut model = render.handle.load_model(&render.thread, SPHERE_PATH).unwrap();

    let mut halo_tile = [0u16; 64];
    let mut tile_img = Image::gen_image_color(8, 8, Color::BLANK);
    tile_img.set_format(raylib::consts::PixelFormat::PIXELFORMAT_UNCOMPRESSED_GRAY_ALPHA);
    let mut tile_texture = render
        .handle
        .load_texture_from_image(&render.thread, &tile_img)
        .unwrap();
    tile_texture.set_texture_wrap(&render.thread, TEXTURE_WRAP_CLAMP);
    let thetas = precompute_thetas(&model.meshes()[0]);
    generate_circle_image_no_dither_fast(i_time, &mut halo_tile);
    let radial = build_radial_magnitudes_fast(i_time);
    let byte_count = halo_tile.len() * size_of::<u16>();
    let pixels = unsafe { from_raw_parts(halo_tile.as_ptr() as *const u8, byte_count) };
    tile_texture.update_texture(pixels).unwrap();
    deform_mesh_by_radial_magnitudes_fast(&mut model.meshes_mut()[0], &radial, &thetas);

    deform_mesh_by_silhouette_radii(&mut model.meshes_mut()[0], &radial_magnitudes_2d);

    let mut wire_model = render.handle.load_model(&render.thread, SPHERE_PATH).unwrap();

    deform_mesh_by_silhouette_radii(&mut wire_model.meshes_mut()[0], &radial_magnitudes_2d);

    let model_pos = Vector3::new(0.0, 0.0, 0.0);
    let model_scale = Vector3::ONE;
    let deformed_vertices = &mut model.meshes_mut()[0].vertices().to_vec();

    let circle_img_3d = make_radial_gradient_face(256, &radial_magnitudes_2d);
    let texture_3d = render
        .handle
        .load_texture_from_image(&render.thread, &circle_img_3d)
        .unwrap();
    texture_3d.set_texture_wrap(&render.thread, TEXTURE_WRAP_CLAMP);

    let mut silhouette_uv = map_mesh_vertices_to_silhouette_texcoords(&deformed_vertices, 0.0, &radial_magnitudes_2d);
    unsafe {
        update_texcoords(&mut model.meshes_mut()[0], &silhouette_uv);
    }
    model.materials_mut()[0].maps_mut()[MATERIAL_MAP_ALBEDO as usize].texture = *texture_3d;
    let observer = Camera3D {
        position: Vector3::new(0.0, 0.0, 2.0),
        target: Vector3::ZERO,
        up: Vector3::Y,
        fovy: 2.0,
        projection: CameraProjection::CAMERA_ORTHOGRAPHIC,
    };
    while !render.handle.window_should_close() {
        i_time += render.handle.get_frame_time();
        mesh_rotation += render.handle.get_frame_time() * TAU * 0.25;

        silhouette_uv =
            map_mesh_vertices_to_silhouette_texcoords(&deformed_vertices, mesh_rotation, &radial_magnitudes_2d);
        unsafe {
            update_texcoords(&mut model.meshes_mut()[0], &silhouette_uv);
        }
        let mut draw_handle = render.handle.begin_drawing(&render.thread);
        draw_handle.clear_background(Color::BLACK);
        // draw_handle.draw_texture_rec(
        //     &texture,
        //     flip_framebuffer(screen_w as f32, screen_h as f32),
        //     ORIGIN,
        //     Color::WHITE,
        // );
        let mut rl3d = draw_handle.begin_mode3D(observer);
        // rl3d.draw_model_ex(
        //     &model,
        //     model_pos,
        //     Vector3::Y,
        //     mesh_rotation.to_degrees(),
        //     model_scale,
        //     Color::WHITE,
        // );

        rl3d.draw_model_wires_ex(
            &wire_model,
            model_pos,
            Vector3::Y,
            mesh_rotation.to_degrees(),
            model_scale,
            Color::WHITE,
        );
    }
}

fn make_radial_gradient_face(size: i32, mags: &[f32]) -> Image {
    const FADE_FRAC: f32 = 0.1;
    let mut img = Image::gen_image_color(size, size, Color::BLANK);
    let data = unsafe { from_raw_parts_mut(img.data as *mut u8, (size * size * 4) as usize) };
    let c = (size - 1) as f32 * 0.5;
    let mags_max = mags.iter().cloned().fold(0.0_f32, f32::max).max(1e-6);
    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 - c;
            let dy = y as f32 - c;
            let r = (dx * dx + dy * dy).sqrt();
            let theta = dy.atan2(dx).rem_euclid(TAU);
            let f = theta / TAU * RADIAL_SAMPLE_COUNT as f32;
            let i0 = f.floor() as usize % RADIAL_SAMPLE_COUNT;
            let i1 = (i0 + 1) % RADIAL_SAMPLE_COUNT;
            let w_hi = f.fract();
            let w_lo = 1.0 - w_hi;
            let edge_norm = (mags[i0] * w_lo + mags[i1] * w_hi) / mags_max; // 0‥1
            let radius_outer = edge_norm * c;
            if radius_outer <= 0.0 {
                continue;
            }
            let radius_inner = radius_outer * (1.0 - FADE_FRAC);
            let alpha = if r <= radius_inner {
                1.0
            } else if r >= radius_outer {
                0.0
            } else {
                1.0 - (r - radius_inner) / (radius_outer - radius_inner)
            };

            let px = (y * size + x) as usize * 4;
            let a = (alpha * 255.0) as u8;
            data[px..px + 4].copy_from_slice(&[255, 255, 255, a]);
        }
    }
    img.set_format(PIXELFORMAT_UNCOMPRESSED_R8G8B8A8);
    img
}
