use crate::fixed_func::silhouette::{
    add_phase, rotate_vertices, smoothstep, spatial_phase, temporal_phase, uv_to_grid_space, ALPHA_FADE_RAMP_MAX,
    ALPHA_FADE_RAMP_MIN, DITHER_BLEND_FACTOR, DITHER_TEXTURE_SCALE, FOVY, UMBRAL_MASK_CENTER, UMBRAL_MASK_FADE_BAND,
    UMBRAL_MASK_INNER_RADIUS, UMBRAL_MASK_OFFSET_X, UMBRAL_MASK_OFFSET_Y, UMBRAL_MASK_OUTER_RADIUS,
};
use crate::fixed_func::topology::{
    collect_vertex_normals, observed_line_of_sight, smooth_vertex_normals, topology_init,
};
use raylib::camera::Camera3D;
use raylib::color::Color;
use raylib::consts::PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8;
use raylib::drawing::{RaylibDraw, RaylibDrawHandle};
use raylib::ffi::{rlReadScreenPixels, MemFree};
use raylib::math::{Rectangle, Vector2};
use raylib::models::{Model, RaylibMesh, RaylibModel};
use raylib::prelude::{Image, WeakTexture2D};
use raylib::texture::RaylibTexture2D;
use std::f32::consts::TAU;
use std::ffi::c_void;
use std::ptr::copy_nonoverlapping;
use std::slice::from_raw_parts_mut;

const BAYER_SIZE: usize = 8;
const BAYER_8X8_RANKS: [[u8; BAYER_SIZE]; BAYER_SIZE] = [
    [0, 32, 8, 40, 2, 34, 10, 42],
    [48, 16, 56, 24, 50, 18, 58, 26],
    [12, 44, 4, 36, 14, 46, 6, 38],
    [60, 28, 52, 20, 62, 30, 54, 22],
    [3, 35, 11, 43, 1, 33, 9, 41],
    [51, 19, 59, 27, 49, 17, 57, 25],
    [15, 47, 7, 39, 13, 45, 5, 37],
    [63, 31, 55, 23, 61, 29, 53, 21],
];

pub fn generate_silhouette_texture(texture_w: i32, texture_h: i32) -> Image {
    let mut image = Image::gen_image_color(texture_w, texture_h, Color::BLANK);
    let total_byte_count = (texture_w * texture_h * 4) as usize;
    let pixel_data_bytes = unsafe { from_raw_parts_mut(image.data as *mut u8, total_byte_count) };
    for texel_y in 0..texture_h {
        let row_normalized = texel_y as f32 / (texture_h as f32 - 1.0);
        let alpha_1_to_0 = (1.0 - smoothstep(0.0, 1.0, row_normalized)).clamp(0.0, 1.0);
        let alpha_0_to_255 = (alpha_1_to_0 * 255.0).round() as u8;
        for texel_x in 0..texture_w {
            let pixel_index = 4 * (texel_y as usize * texture_w as usize + texel_x as usize);
            pixel_data_bytes[pixel_index + 0] = 255;
            pixel_data_bytes[pixel_index + 1] = 255;
            pixel_data_bytes[pixel_index + 2] = 255;
            pixel_data_bytes[pixel_index + 3] = alpha_0_to_255;
        }
    }
    image.set_format(PIXELFORMAT_UNCOMPRESSED_R8G8B8A8);
    image
}

pub fn dither(silhouette_image: &mut Image) {
    let total_bytes = (silhouette_image.width * silhouette_image.height * 4) as usize;
    let pixel_data = unsafe { from_raw_parts_mut(silhouette_image.data as *mut u8, total_bytes) };
    let bayer_blend = DITHER_BLEND_FACTOR.clamp(0.0, 1.0);
    for y in 0..silhouette_image.height as usize {
        for x in 0..silhouette_image.width as usize {
            let pixel_index = 4 * (y * silhouette_image.width as usize + x);
            let alpha_1_to_0 = (pixel_data[pixel_index + 3] as f32) / 255.0;
            let bayer_x = x % BAYER_SIZE;
            let bayer_y = y % BAYER_SIZE;
            let threshold_0_to_1 = (BAYER_8X8_RANKS[bayer_y][bayer_x] as f32) / 64.0;
            let bayer_check = if alpha_1_to_0 > threshold_0_to_1 { 1.0 } else { 0.0 };
            let bayer_alpha_1_to_0 = alpha_1_to_0 * (1.0 - bayer_blend) + bayer_check * bayer_blend;
            pixel_data[pixel_index + 3] = (bayer_alpha_1_to_0 * 255.0).round() as u8;
        }
    }
}

pub fn rotate_silhouette_texture_dither(
    model: &mut Model,
    observer: &Camera3D,
    mesh_rotation: f32,
    screen_w: i32,
    screen_h: i32,
) {
    let mesh = &mut model.meshes_mut()[0];
    let mut vertices = mesh.vertices_mut().to_vec();
    let mut smooth_vertex_normals = {
        let mut topology = topology_init(mesh);
        collect_vertex_normals(&mut topology, mesh);
        smooth_vertex_normals(&topology)
    };
    rotate_vertices(&mut vertices, mesh_rotation);
    rotate_vertices(&mut smooth_vertex_normals, mesh_rotation);
    let observed_line_of_sight = observed_line_of_sight(observer);
    let vertex_count = mesh.vertexCount as usize;
    if mesh.as_mut().texcoords.is_null() {
        let texcoords = vec![0.0f32; vertex_count * 2];
        mesh.as_mut().texcoords = Box::leak(texcoords.into_boxed_slice()).as_mut_ptr();
    }
    let texcoords = unsafe { from_raw_parts_mut(mesh.as_mut().texcoords, vertex_count * 2) };
    let world_to_pixels = screen_h as f32 / FOVY;
    for i in 0..vertex_count {
        let vertex = vertices[i];
        let vertex_normal = smooth_vertex_normals[i];
        let x_component = vertex.x * world_to_pixels + (screen_w as f32) * 0.5;
        let s = x_component / DITHER_TEXTURE_SCALE;
        let alignment_magnitude = vertex_normal.dot(observed_line_of_sight).abs();
        // let t = smoothstep(ALPHA_FADE_RAMP_MIN, ALPHA_FADE_RAMP_MAX, alignment_magnitude);
        let t = 1.0 - smoothstep(ALPHA_FADE_RAMP_MIN, ALPHA_FADE_RAMP_MAX, alignment_magnitude);
        texcoords[i * 2 + 0] = s;
        texcoords[i * 2 + 1] = t;
    }
}

pub fn rotate_silhouette_texture(model: &mut Model, observer: &Camera3D, mesh_rotation: f32) {
    let mesh = &mut model.meshes_mut()[0];
    let mut vertices = mesh.vertices_mut().to_vec();
    let mut smooth_vertex_normals = {
        let mut topology = topology_init(mesh);
        collect_vertex_normals(&mut topology, mesh);
        smooth_vertex_normals(&topology)
    };
    rotate_vertices(&mut vertices, mesh_rotation);
    rotate_vertices(&mut smooth_vertex_normals, mesh_rotation);
    let observed_line_of_sight = observed_line_of_sight(observer);
    let vertex_count = mesh.vertexCount as usize;
    if mesh.as_mut().texcoords.is_null() {
        let texcoords = vec![0.0f32; vertex_count * 2];
        mesh.as_mut().texcoords = Box::leak(texcoords.into_boxed_slice()).as_mut_ptr();
    }
    let texcoords = unsafe { from_raw_parts_mut(mesh.as_mut().texcoords, vertex_count * 2) };
    for vertex_index in 0..vertex_count {
        let vertex = vertices[vertex_index];
        let vertex_normal = smooth_vertex_normals[vertex_index];
        let angle_component = vertex.y.atan2(vertex.x).rem_euclid(TAU);
        let s = angle_component / TAU;

        let alignment_magnitude = vertex_normal.dot(observed_line_of_sight).abs();
        let t = 1.0 - smoothstep(ALPHA_FADE_RAMP_MIN, ALPHA_FADE_RAMP_MAX, alignment_magnitude);

        texcoords[vertex_index * 2 + 0] = s;
        texcoords[vertex_index * 2 + 1] = t;
    }
}

pub struct DitherStaging {
    pub blit_texture: WeakTexture2D,
    pub is_initialized: bool,
    pub staging_rgba_bytes: Vec<u8>,
}

pub fn screen_pass_dither(draw_handle: &mut RaylibDrawHandle, dither_staging: &mut DitherStaging) {
    let screen_w = draw_handle.get_screen_width();
    let screen_h = draw_handle.get_screen_height();
    let byte_count = (screen_w * screen_h * 4) as usize;
    if dither_staging.staging_rgba_bytes.len() != byte_count {
        dither_staging.staging_rgba_bytes.resize(byte_count, 0);
    }
    unsafe {
        let screen_pixels = rlReadScreenPixels(screen_w, screen_h);
        copy_nonoverlapping(
            screen_pixels,
            dither_staging.staging_rgba_bytes.as_mut_ptr(),
            byte_count,
        );
        MemFree(screen_pixels as *mut c_void);
    }

    dither_byte_level(&mut dither_staging.staging_rgba_bytes, screen_w, screen_h);
    dither_staging
        .blit_texture
        .update_texture(&dither_staging.staging_rgba_bytes)
        .unwrap();
    let src = Rectangle {
        x: 0.0,
        y: 0.0,
        width: screen_w as f32,
        height: screen_h as f32,
    };
    let dst = Rectangle {
        x: 0.0,
        y: 0.0,
        width: screen_w as f32,
        height: screen_h as f32,
    };
    draw_handle.draw_texture_pro(&dither_staging.blit_texture, src, dst, Vector2::ZERO, 0.0, Color::WHITE);
}

pub fn dither_byte_level(pixels_rgba8: &mut [u8], width_pixels: i32, height_pixels: i32) {
    let blend_lambda = DITHER_BLEND_FACTOR.clamp(0.0, 1.0);
    for screen_y in 0..height_pixels {
        for screen_x in 0..width_pixels {
            let pixel_index = 4 * (screen_y as usize * width_pixels as usize + screen_x as usize);
            let r = pixels_rgba8[pixel_index + 0];
            let g = pixels_rgba8[pixel_index + 1];
            let b = pixels_rgba8[pixel_index + 2];
            let lum = luminance(r, g, b);
            let cell_x_index = (screen_x as usize) & 7;
            let cell_y_index = (screen_y as usize) & 7;
            let threshold_0_to_1 = (BAYER_8X8_RANKS[cell_y_index][cell_x_index] as f32) / 64.0;
            let on = if lum > threshold_0_to_1 { 1.0 } else { 0.0 };
            let dither_normal = lum * (1.0 - blend_lambda) + on * blend_lambda;
            let dither = (dither_normal * 255.0).round() as u8;
            pixels_rgba8[pixel_index + 0] = dither;
            pixels_rgba8[pixel_index + 1] = dither;
            pixels_rgba8[pixel_index + 2] = dither;
            pixels_rgba8[pixel_index + 3] = 255;
        }
    }
}

#[inline]
fn luminance(r: u8, g: u8, b: u8) -> f32 {
    (0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32) / 255.0
}

// --- tiny 4x4 PS1-ish Bayer ranks (binary mask) ---
const N: i32 = 4;
const BAYER4: [[u8; 4]; 4] = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]];

/// How many discrete coverage levels to approximate (more = smoother fade, still cheap)
pub const STIPPLE_LEVELS: i32 = 17; // 0..16 ≈ 5-bit like PS1 feel

pub fn build_stipple_atlas_rgba() -> Image {
    // Atlas is Nx(N*L) where each NxN slice is the mask for one level
    let w = N;
    let h = N * STIPPLE_LEVELS;
    let mut img = Image::gen_image_color(w, h, Color::BLANK);
    let total_bytes: usize = (w * h * 4) as usize;
    let px: &mut [u8] = unsafe { from_raw_parts_mut(img.data as *mut u8, total_bytes) };

    for level in 0..STIPPLE_LEVELS {
        // Threshold is the number of "on" ranks we allow for this level
        // level==0 -> nothing on (fully transparent), level==16 -> mostly on
        for y in 0..N {
            for x in 0..N {
                let idx = 4 * (((level * N + y) * w + x) as usize);
                let on = (BAYER4[y as usize][x as usize] as i32) < level;
                // White color, alpha 0/255 as ON/OFF mask
                px[idx + 0] = 255;
                px[idx + 1] = 255;
                px[idx + 2] = 255;
                px[idx + 3] = if on { 255 } else { 0 };
            }
        }
    }
    img.set_format(PIXELFORMAT_UNCOMPRESSED_R8G8B8A8);
    img
}

pub fn rotate_silhouette_texture_stipple_screen_locked(
    model: &mut Model,
    observer: &Camera3D,
    mesh_rotation: f32,
    screen_w: i32,
    screen_h: i32,
) {
    let mesh = &mut model.meshes_mut()[0];
    let mut vertices = mesh.vertices_mut().to_vec();
    let mut smooth_vertex_normals = {
        let mut topo = topology_init(mesh);
        collect_vertex_normals(&mut topo, mesh);
        smooth_vertex_normals(&topo)
    };
    rotate_vertices(&mut vertices, mesh_rotation);
    rotate_vertices(&mut smooth_vertex_normals, mesh_rotation);
    let los = observed_line_of_sight(observer);
    let vertex_count = mesh.vertexCount as usize;
    if mesh.as_mut().texcoords.is_null() {
        let tex = vec![0.0f32; vertex_count * 2];
        mesh.as_mut().texcoords = Box::leak(tex.into_boxed_slice()).as_mut_ptr();
    }
    let texcoords = unsafe { from_raw_parts_mut(mesh.as_mut().texcoords, vertex_count * 2) };
    let world_to_px = (screen_h as f32) / FOVY;
    let half_w = (screen_w as f32) * 0.5;
    let half_h = (screen_h as f32) * 0.5;

    for i in 0..vertex_count {
        let vtx = vertices[i];
        let x_px = (vtx.x * world_to_px + half_w).floor() as i32;
        let y_px = (vtx.y * world_to_px + half_h).floor() as i32;
        let n = smooth_vertex_normals[i];
        let align = n.dot(los).abs();
        let t = 1.0 - smoothstep(ALPHA_FADE_RAMP_MIN, ALPHA_FADE_RAMP_MAX, align);
        let coverage = (1.0 - t).clamp(0.0, 1.0);
        let level_f = (coverage * (STIPPLE_LEVELS as f32 - 1.0)).round();
        let level = level_f as i32;
        let u = ((x_px & (N - 1)) as f32 + 0.5) / (N as f32);
        let v = ((level * N + (y_px & (N - 1))) as f32 + 0.5) / ((STIPPLE_LEVELS * N) as f32);
        texcoords[i * 2 + 0] = u;
        texcoords[i * 2 + 1] = v;
    }
}

pub fn umbral_mask_strength_uv(uv: Vector2, i_time: f32) -> f32 {
    // map to your tiled grid space (uses GRID_ORIGIN_UV_OFFSET / GRID_SCALE)
    let mut grid = uv_to_grid_space(uv);

    // animate with your phase helpers
    let mut phase = spatial_phase(grid);
    phase += temporal_phase(i_time);
    grid += add_phase(phase);

    // soft disk mask
    let d_shape = grid.distance(UMBRAL_MASK_CENTER);
    let mask = smoothstep(
        UMBRAL_MASK_OUTER_RADIUS,
        UMBRAL_MASK_OUTER_RADIUS - UMBRAL_MASK_FADE_BAND,
        d_shape,
    );

    // offset “light” bias
    let light_pos = UMBRAL_MASK_CENTER + Vector2::new(UMBRAL_MASK_OFFSET_X, UMBRAL_MASK_OFFSET_Y);
    let lo = grid.distance(light_pos);
    let shade = smoothstep(UMBRAL_MASK_INNER_RADIUS, UMBRAL_MASK_OUTER_RADIUS, lo * 0.5);

    (mask * shade).clamp(0.0, 1.0)
}

pub fn apply_umbral_mask_alpha_from_uv(model: &mut Model, i_time: f32) {
    let mesh = &mut model.meshes_mut()[0];
    let vcount = mesh.vertexCount as usize;
    if vcount == 0 || mesh.as_mut().texcoords.is_null() {
        return;
    }
    if mesh.as_mut().colors.is_null() {
        let colors = vec![255u8; vcount * 4];
        mesh.as_mut().colors = Box::leak(colors.into_boxed_slice()).as_mut_ptr();
    }
    let uvs = unsafe { from_raw_parts_mut(mesh.as_mut().texcoords, vcount * 2) };
    let colors = unsafe { from_raw_parts_mut(mesh.as_mut().colors, vcount * 4) };
    for i in 0..vcount {
        let uv = Vector2::new(uvs[i * 2 + 0], uvs[i * 2 + 1]);
        let strength = umbral_mask_strength_uv(uv, i_time);
        let j = i * 4;
        colors[j + 0] = 255;
        colors[j + 1] = 255;
        colors[j + 2] = 255;
        colors[j + 3] = (strength * 255.0).round() as u8;
    }
}

pub fn update_umbral_animation(model: &mut Model, i_time: f32) {
    apply_umbral_mask_alpha_from_uv(model, i_time);
}

pub fn generate_spherical_uvs(mesh: &mut raylib::models::WeakMesh) {
    let vcount = mesh.vertexCount as usize;
    if vcount == 0 {
        return;
    }
    if mesh.texcoords.is_null() {
        let texcoords = vec![0.0f32; vcount * 2];
        mesh.texcoords = Box::leak(texcoords.into_boxed_slice()).as_mut_ptr();
    }

    let verts = mesh.vertices();
    let uvs = unsafe { from_raw_parts_mut(mesh.texcoords, vcount * 2) };

    for i in 0..vcount {
        let v = verts[i];
        // u in [0,1)
        let u = v.z.atan2(v.x) / TAU;
        let u = if u < 0.0 { u + 1.0 } else { u };
        // v in [0,1]
        let vv = (v.y * 0.5 + 0.5).clamp(0.0, 1.0);
        uvs[i * 2 + 0] = u;
        uvs[i * 2 + 1] = vv;
    }
    unsafe {
        raylib::ffi::UploadMesh(mesh.as_mut(), true);
    }
}

pub fn write_umbral_mask_image(img: &mut Image, i_time: f32) {
    let w = img.width;
    let h = img.height;
    let total = (w * h * 4) as usize;
    let px: &mut [u8] = unsafe { from_raw_parts_mut(img.data as *mut u8, total) };

    for y in 0..h {
        let v = (y as f32 + 0.5) / (h as f32);
        for x in 0..w {
            let u = (x as f32 + 0.5) / (w as f32);
            let strength = umbral_mask_strength_uv(Vector2::new(u, v), i_time); // 0..1

            let idx = 4 * (y as usize * w as usize + x as usize);
            px[idx + 0] = 255;
            px[idx + 1] = 255;
            px[idx + 2] = 255;
            px[idx + 3] = (strength * 255.0).round() as u8;
        }
    }
    img.set_format(PIXELFORMAT_UNCOMPRESSED_R8G8B8A8);
}
