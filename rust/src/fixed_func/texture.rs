use crate::fixed_func::silhouette::{
    add_phase, smoothstep, spatial_phase, temporal_phase, uv_to_grid_space, ALPHA_FADE_RAMP_MAX, ALPHA_FADE_RAMP_MIN,
    DITHER_BLEND_FACTOR, DITHER_TEXTURE_SCALE, FOVY_ORTHOGRAPHIC, UMBRAL_MASK_CENTER, UMBRAL_MASK_FADE_BAND,
    UMBRAL_MASK_INNER_RADIUS, UMBRAL_MASK_OFFSET_X, UMBRAL_MASK_OFFSET_Y, UMBRAL_MASK_OUTER_RADIUS,
};
use crate::fixed_func::topology::{observed_line_of_sight, rotate_vertices_in_plane_slice, Topology};
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

//TODO: oh my god is this actually making that umbral mask texture by accidentally failing to make a front face only alpha radial feather??
pub fn generate_silhouette_texture(texture_w: i32, texture_h: i32) -> Image {
    let mut image = Image::gen_image_color(texture_w, texture_h, Color::BLANK);
    image.set_format(PIXELFORMAT_UNCOMPRESSED_R8G8B8A8);
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
    image
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
    let topology = Topology::build_topology(mesh)
        .welded_vertices()
        .triangles()
        .vertex_normals()
        .smooth_vertex_normals()
        .build();
    let mut smooth_vertex_normals = topology.vertex_normals_snapshot.unwrap();
    rotate_vertices_in_plane_slice(&mut vertices, mesh_rotation);
    rotate_vertices_in_plane_slice(&mut smooth_vertex_normals, mesh_rotation);
    let observed_line_of_sight = observed_line_of_sight(observer);
    let vertex_count = mesh.vertexCount as usize;
    let texcoords = mesh.ensure_texcoords().unwrap();
    let world_to_pixels = screen_h as f32 / FOVY_ORTHOGRAPHIC;
    for i in 0..vertex_count {
        let vertex = vertices[i];
        let vertex_normal = smooth_vertex_normals[i];
        let x_component = vertex.x * world_to_pixels + (screen_w as f32) * 0.5;
        let s = x_component / DITHER_TEXTURE_SCALE;
        let alignment_magnitude = vertex_normal.dot(observed_line_of_sight).abs();
        // let t = smoothstep(ALPHA_FADE_RAMP_MIN, ALPHA_FADE_RAMP_MAX, alignment_magnitude);
        let t = 1.0 - smoothstep(ALPHA_FADE_RAMP_MIN, ALPHA_FADE_RAMP_MAX, alignment_magnitude);
        texcoords[i].x = s;
        texcoords[i].y = t;
    }
}

pub fn rotate_silhouette_texture(model: &mut Model, observer: &Camera3D, mesh_rotation: f32) {
    let mesh = &mut model.meshes_mut()[0];
    let mut vertices = mesh.vertices_mut().to_vec();
    let topology = Topology::build_topology(mesh)
        .welded_vertices()
        .triangles()
        .vertex_normals()
        .smooth_vertex_normals()
        .build();
    let mut smooth_vertex_normals = topology.vertex_normals_snapshot.unwrap();
    rotate_vertices_in_plane_slice(&mut vertices, mesh_rotation);
    rotate_vertices_in_plane_slice(&mut smooth_vertex_normals, mesh_rotation);
    let observed_line_of_sight = observed_line_of_sight(observer);
    let vertex_count = mesh.vertexCount as usize;
    let texcoords = mesh.ensure_texcoords().unwrap();
    for vertex_index in 0..vertex_count {
        let vertex = vertices[vertex_index];
        let vertex_normal = smooth_vertex_normals[vertex_index];
        let angle_component = vertex.y.atan2(vertex.x).rem_euclid(TAU);
        let s = angle_component / TAU;

        let alignment_magnitude = vertex_normal.dot(observed_line_of_sight).abs();
        let t = 1.0 - smoothstep(ALPHA_FADE_RAMP_MIN, ALPHA_FADE_RAMP_MAX, alignment_magnitude);
        texcoords[vertex_index].x = s;
        texcoords[vertex_index].y = t;
    }
    // unsafe {
    // let texcoord_data = from_raw_parts(texcoords.as_ptr() as *const u8, texcoords.len() * size_of::<Vector2>());
    // mesh.update_buffer(RL_DEFAULT_SHADER_ATTRIB_LOCATION_TEXCOORD as i32, texcoord_data, 0);
    // }
}

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

pub struct ScreenPassDither {
    pub blit_texture: WeakTexture2D,
    pub is_initialized: bool,
    pub staging_rgba_bytes: Vec<u8>,
}

pub fn screen_pass_dither(draw_handle: &mut RaylibDrawHandle, dither_staging: &mut ScreenPassDither) {
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

pub fn umbral_mask_strength_uv(uv: Vector2, i_time: f32) -> f32 {
    let mut grid = uv_to_grid_space(uv);
    let mut phase = spatial_phase(grid);
    phase += temporal_phase(i_time);
    grid += add_phase(phase);
    let d_shape = grid.distance(UMBRAL_MASK_CENTER);
    let mask = smoothstep(
        UMBRAL_MASK_OUTER_RADIUS,
        UMBRAL_MASK_OUTER_RADIUS - UMBRAL_MASK_FADE_BAND,
        d_shape,
    );
    let light_pos = UMBRAL_MASK_CENTER + Vector2::new(UMBRAL_MASK_OFFSET_X, UMBRAL_MASK_OFFSET_Y);
    let lo = grid.distance(light_pos);
    let shade = smoothstep(UMBRAL_MASK_INNER_RADIUS, UMBRAL_MASK_OUTER_RADIUS, lo * 0.5);
    (mask * shade).clamp(0.0, 1.0)
}
