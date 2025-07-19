use asset_payload::SPHERE_PATH;
use bath::fixed_func::ghost::{
    smoothstep, GRID_ORIGIN_UV_OFFSET, GRID_SCALE, UMBRAL_MASK_CENTER, UMBRAL_MASK_FADE_BAND, UMBRAL_MASK_OUTER_RADIUS,
};
use bath::geometry::weld_vertices::weld_and_index_mesh;
use bath::render::raylib::RaylibRenderer;
use bath::render::raylib_util::N64_WIDTH;
use bath::render::renderer::Renderer;
use raylib::camera::Camera3D;
use raylib::color::Color;
use raylib::consts::BlendMode::BLEND_ALPHA;
use raylib::consts::CameraProjection;
use raylib::consts::MaterialMapIndex::MATERIAL_MAP_ALBEDO;
use raylib::consts::PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8;
use raylib::consts::TextureFilter::TEXTURE_FILTER_BILINEAR;
use raylib::consts::TextureWrap::TEXTURE_WRAP_CLAMP;
use raylib::drawing::{RaylibBlendModeExt, RaylibDraw, RaylibDraw3D, RaylibMode3DExt};
use raylib::ffi::MemAlloc;
use raylib::math::{Vector2, Vector3};
use raylib::models::{RaylibMaterial, RaylibMesh, RaylibModel};
use raylib::prelude::Image;
use raylib::texture::RaylibTexture2D;
use std::slice::from_raw_parts_mut;

fn main() {
    let mut render = RaylibRenderer::init(N64_WIDTH, N64_WIDTH);
    let screen_w = render.handle.get_screen_width();
    let screen_h = render.handle.get_screen_height();
    let mut i_time = 0.0f32;
    let circle_img = generate_circle_image(screen_w, screen_h);
    let texture = render
        .handle
        .load_texture_from_image(&render.thread, &circle_img)
        .unwrap();

    let mut model = render.handle.load_model(&render.thread, SPHERE_PATH).unwrap();
    let observer = Camera3D {
        position: Vector3::new(0.0, 0.0, 2.0),
        target: Vector3::ZERO,
        up: Vector3::Y,
        fovy: 10.0,
        projection: CameraProjection::CAMERA_ORTHOGRAPHIC,
    };

    let mesh_slice = model.meshes_mut();
    let mesh = &mut mesh_slice[0];
    weld_and_index_mesh(mesh, 1e-6);
    let gradient_img = make_radial_gradient(256);
    let gradient_tex = render
        .handle
        .load_texture_from_image(&render.thread, &gradient_img)
        .unwrap();

    gradient_tex.set_texture_wrap(&render.thread, TEXTURE_WRAP_CLAMP);
    gradient_tex.set_texture_filter(&render.thread, TEXTURE_FILTER_BILINEAR);

    let vertices = mesh.vertices_mut();
    let radius = vertices.iter().fold(0.0f32, |r, v| r.max(v.length()));

    let mut texcoords = Vec::<f32>::with_capacity(vertices.len() * 2);
    for v in vertices.iter() {
        texcoords.push(v.x / radius * 0.5 + 0.5);
        texcoords.push(-v.y / radius * 0.5 + 0.5);
    }

    unsafe {
        mesh.texcoords = MemAlloc((texcoords.len() * 4) as u32) as *mut f32;
        std::ptr::copy_nonoverlapping(texcoords.as_ptr(), mesh.texcoords, texcoords.len());
    }
    unsafe {
        mesh.upload(false);
    }
    model.materials_mut()[0].maps_mut()[MATERIAL_MAP_ALBEDO as usize].texture = *gradient_tex;

    while !render.handle.window_should_close() {
        i_time += render.handle.get_frame_time();
        let mut draw_handle = render.handle.begin_drawing(&render.thread);
        draw_handle.clear_background(Color::BLACK);
        // draw_handle.draw_texture_rec(
        //     &texture,
        //     flip_framebuffer(screen_w as f32, screen_h as f32),
        //     ORIGIN,
        //     Color::WHITE,
        // );
        let model_pos = Vector3::new(0.0, 2.0, 0.0);
        let model_scale = Vector3::ONE;
        let mut rl3d = draw_handle.begin_mode3D(observer);
        let mut blend_mode = rl3d.begin_blend_mode(BLEND_ALPHA);
        //blend_mode.draw_model_wires_ex(&model, model_pos, Vector3::Y, 0.0, model_scale, Color::WHITE);
        blend_mode.draw_model_ex(&model, model_pos, Vector3::Y, i_time * 90.0, model_scale, Color::WHITE);
    }
}

fn make_radial_gradient(size: i32) -> Image {
    let mut img = Image::gen_image_color(size, size, Color::BLANK);
    let data = unsafe { from_raw_parts_mut(img.data as *mut u8, (size * size * 4) as usize) };
    let c = (size - 1) as f32 * 0.5;
    let inner = 0.9;
    for y in 0..size {
        for x in 0..size {
            let r = ((x as f32 - c).hypot(y as f32 - c)) / c;
            let alpha = ((inner - r) / (1.0 - inner)).clamp(0.0, 1.0);

            let px = (y * size + x) as usize * 4;
            data[px + 0] = 255; // R
            data[px + 1] = 255; // G
            data[px + 2] = 255; // B
            data[px + 3] = (alpha * 255.0) as u8; // A
        }
    }
    img.set_format(PIXELFORMAT_UNCOMPRESSED_R8G8B8A8);
    img
}

#[inline]
fn generate_circle_image(width: i32, height: i32) -> Image {
    let img = Image::gen_image_color(width, height, Color::BLANK);
    let total_bytes = (width * height * 4) as usize;
    let pixels: &mut [u8] = unsafe { from_raw_parts_mut(img.data as *mut u8, total_bytes) };
    for y in 0..height {
        for x in 0..width {
            let s = (x as f32 + 0.5) / width as f32;
            let t = (y as f32 + 0.5) / height as f32;
            let uv = Vector2::new(s, t);
            let centre_offset = GRID_ORIGIN_UV_OFFSET - Vector2::splat(0.5 / GRID_SCALE);
            let grid_coords = (uv - centre_offset) * GRID_SCALE;
            let distance_from_center = grid_coords.distance(UMBRAL_MASK_CENTER);
            let fade_start = UMBRAL_MASK_OUTER_RADIUS - UMBRAL_MASK_FADE_BAND;
            let alpha = 1.0 - smoothstep(fade_start, UMBRAL_MASK_OUTER_RADIUS, distance_from_center);
            let lum = alpha.clamp(0.0, 1.0);
            let lum_u8 = (lum * 255.0) as u8;
            let idx = 4 * (y as usize * width as usize + x as usize);
            pixels[idx] = lum_u8; // R
            pixels[idx + 1] = lum_u8; // G
            pixels[idx + 2] = lum_u8; // B
            pixels[idx + 3] = 255u8; // A
        }
    }
    img
}
