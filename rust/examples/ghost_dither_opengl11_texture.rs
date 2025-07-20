use asset_payload::SPHERE_PATH;
use bath::fixed_func::ghost::{
    add_phase, smoothstep, spatial_phase, temporal_phase, GRID_ORIGIN_UV_OFFSET, GRID_SCALE, UMBRAL_MASK_CENTER,
    UMBRAL_MASK_FADE_BAND, UMBRAL_MASK_OUTER_RADIUS,
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
use raylib::models::{RaylibMaterial, RaylibMesh, RaylibModel, WeakMesh};
use raylib::prelude::Image;
use raylib::texture::RaylibTexture2D;
use std::f32::consts::TAU;
use std::slice::from_raw_parts_mut;

fn project_uvs(verts: &[Vector3], angle: f32, radius: f32) -> Vec<f32> {
    let c = angle.cos();
    let s = angle.sin();
    let mut uv = Vec::<f32>::with_capacity(verts.len() * 2);
    for v in verts {
        let rx = c * v.x + s * v.z;
        let ry = v.y;
        uv.push(rx / radius * 0.5 + 0.5);
        uv.push(ry / radius * 0.5 + 0.5);
    }
    uv
}

fn main() {
    let mut render = RaylibRenderer::init(N64_WIDTH, N64_WIDTH);
    let screen_w = render.handle.get_screen_width();
    let screen_h = render.handle.get_screen_height();
    let mut i_time = 0.0f32;
    let circle_img = generate_circle_image(screen_w, screen_h, i_time);
    let texture = render
        .handle
        .load_texture_from_image(&render.thread, &circle_img)
        .unwrap();

    let mut model = render.handle.load_model(&render.thread, SPHERE_PATH).unwrap();
    let radial_magnitudes = build_radial_magnitudes(&circle_img); //just testing the initial state compared with the silhhoute at i_time = 0
    deform_mesh_by_radial_magnitudes(&mut model.meshes_mut()[0], &radial_magnitudes);

    let mesh_slice = model.meshes_mut();
    let mesh = &mut mesh_slice[0];
    weld_and_index_mesh(mesh, 1e-6);
    let deformed_verts: Vec<Vector3> = mesh.vertices().to_vec();
    let radius = deformed_verts.iter().fold(0.0_f32, |r, v| r.max(v.length()));
    let radial_feather_img = make_radial_gradient_face(256, &*radial_magnitudes);
    let mesh_texture = render
        .handle
        .load_texture_from_image(&render.thread, &radial_feather_img)
        .unwrap();
    mesh_texture.set_texture_wrap(&render.thread, TEXTURE_WRAP_CLAMP);
    mesh_texture.set_texture_filter(&render.thread, TEXTURE_FILTER_BILINEAR);

    let init_texcoords = project_uvs(&deformed_verts, 0.0, radius);
    unsafe {
        mesh.texcoords = MemAlloc((init_texcoords.len() * 4) as u32) as *mut f32;
        std::ptr::copy_nonoverlapping(init_texcoords.as_ptr(), mesh.texcoords, init_texcoords.len());
        mesh.upload(true);
    }
    model.materials_mut()[0].maps_mut()[MATERIAL_MAP_ALBEDO as usize].texture = *mesh_texture;
    let observer = Camera3D {
        position: Vector3::new(0.0, 0.0, 2.0),
        target: Vector3::ZERO,
        up: Vector3::Y,
        fovy: 2.0,
        projection: CameraProjection::CAMERA_ORTHOGRAPHIC,
    };
    let mut angle = 0.0f32;
    while !render.handle.window_should_close() {
        i_time += render.handle.get_frame_time();
        let dt = render.handle.get_frame_time();
        angle += dt * TAU * 0.25;
        {
            let mesh = &mut model.meshes_mut()[0];
            let dst = project_uvs(&deformed_verts, angle, radius);
            unsafe {
                std::ptr::copy_nonoverlapping(dst.as_ptr(), mesh.texcoords, dst.len());
                mesh.upload(true);
            }
        }
        let mut draw_handle = render.handle.begin_drawing(&render.thread);
        draw_handle.clear_background(Color::DARKRED);
        // draw_handle.draw_texture_rec(
        //     &texture,
        //     flip_framebuffer(screen_w as f32, screen_h as f32),
        //     ORIGIN,
        //     Color::WHITE,
        // );
        let model_pos = Vector3::new(0.0, 0.0, 0.0);
        let model_scale = Vector3::ONE;
        let mut rl3d = draw_handle.begin_mode3D(observer);
        let mut blend_mode = rl3d.begin_blend_mode(BLEND_ALPHA);
        //blend_mode.draw_model_wires_ex(&model, model_pos, Vector3::Y, 0.0, model_scale, Color::WHITE);
        blend_mode.draw_model_ex(
            &model,
            model_pos,
            Vector3::Y,
            angle.to_degrees(),
            model_scale,
            Color::WHITE,
        );
        blend_mode.draw_model_wires_ex(
            &model,
            model_pos,
            Vector3::Y,
            angle.to_degrees(),
            model_scale,
            Color::BLACK,
        );
    }
}

fn make_radial_gradient_face(size: i32, mags: &[f32]) -> Image {
    const FADE_FRAC: f32 = 0.025;
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
            let r_edge = edge_norm * c;
            if r_edge <= 0.0 {
                continue;
            }
            let r_start = r_edge * (1.0 - FADE_FRAC);
            let alpha = if r <= r_start {
                1.0
            } else if r >= r_edge {
                0.0
            } else {
                1.0 - (r - r_start) / (r_edge - r_start)
            };

            let px = (y * size + x) as usize * 4;
            let a = (alpha * 255.0) as u8;
            data[px..px + 4].copy_from_slice(&[255, 255, 255, a]);
        }
    }

    img.set_format(PIXELFORMAT_UNCOMPRESSED_R8G8B8A8);
    img
}

#[inline]
fn generate_circle_image(width: i32, height: i32, i_time: f32) -> Image {
    let img = Image::gen_image_color(width, height, Color::BLANK);
    let total_bytes = (width * height * 4) as usize;
    let pixels: &mut [u8] = unsafe { from_raw_parts_mut(img.data as *mut u8, total_bytes) };
    for y in 0..height {
        for x in 0..width {
            let s = (x as f32 + 0.5) / width as f32;
            let t = (y as f32 + 0.5) / height as f32;
            let uv = Vector2::new(s, t);
            let centre_offset = GRID_ORIGIN_UV_OFFSET - Vector2::splat(0.5 / GRID_SCALE);
            let mut grid_coords = (uv - centre_offset) * GRID_SCALE;
            let mut grid_phase = Vector2::ZERO;
            grid_phase += spatial_phase(grid_coords);
            grid_phase += temporal_phase(i_time);
            grid_coords += add_phase(grid_phase);
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

pub const RADIAL_SAMPLE_COUNT: usize = 24;
pub fn build_radial_magnitudes(source_image: &Image) -> Vec<f32> {
    let image_width_in_pixels = source_image.width();
    let image_height_in_pixels = source_image.height();
    let centre_coordinate_x = (image_width_in_pixels as f32 - 1.0) * 0.5;
    let centre_coordinate_y = (image_height_in_pixels as f32 - 1.0) * 0.5;
    let total_bytes = (image_width_in_pixels * image_height_in_pixels * 4) as usize;
    let pixel_bytes: &[u8] = unsafe { std::slice::from_raw_parts(source_image.data as *const u8, total_bytes) };
    let reference_pixel_radius = UMBRAL_MASK_OUTER_RADIUS * image_width_in_pixels.min(image_height_in_pixels) as f32;
    let mut normalised_radial_magnitudes = vec![0.0_f32; RADIAL_SAMPLE_COUNT];
    for sample_angle_index in 0..RADIAL_SAMPLE_COUNT {
        let angle_theta_radians = (sample_angle_index as f32) * TAU / RADIAL_SAMPLE_COUNT as f32;
        let direction_unit_x = angle_theta_radians.cos();
        let direction_unit_y = angle_theta_radians.sin();
        let mut current_step_in_pixels = 0_i32;
        loop {
            let sample_x = centre_coordinate_x + direction_unit_x * current_step_in_pixels as f32;
            let sample_y = centre_coordinate_y + direction_unit_y * current_step_in_pixels as f32;
            let texel_x = sample_x as i32 as usize;
            let texel_y = sample_y as i32 as usize;
            let pixel_index = 4 * (texel_y * image_width_in_pixels as usize + texel_x);
            let luminance_value = pixel_bytes[pixel_index];
            if luminance_value == 0 {
                break;
            }
            current_step_in_pixels += 1;
        }
        let boundary_distance_in_pixels = current_step_in_pixels as f32;
        let normalised_distance = boundary_distance_in_pixels / reference_pixel_radius;
        normalised_radial_magnitudes[sample_angle_index] = normalised_distance.max(0.0);
    }
    normalised_radial_magnitudes
}

pub fn deform_mesh_by_radial_magnitudes(mesh: &mut WeakMesh, radial_magnitudes: &[f32]) {
    let vertices: &mut [Vector3] = mesh.vertices_mut();
    for vertex in vertices.iter_mut() {
        let theta = vertex.y.atan2(vertex.x).rem_euclid(TAU);
        let idx_f = theta / TAU * RADIAL_SAMPLE_COUNT as f32;
        let i0 = idx_f.floor() as usize % RADIAL_SAMPLE_COUNT;
        let i1 = (i0 + 1) % RADIAL_SAMPLE_COUNT;
        let w_hi = idx_f.fract();
        let w_lo = 1.0 - w_hi;
        let r_equator = radial_magnitudes[i0] * w_lo + radial_magnitudes[i1] * w_hi;
        vertex.x *= r_equator;
        vertex.y *= r_equator;
        vertex.z *= r_equator;
    }
}
