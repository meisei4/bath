use crate::fixed_func::silhouette_constants::{
    ANGULAR_VELOCITY, ROTATIONAL_SAMPLES_FOR_INV_PROJ, SILHOUETTE_RADII_RESOLUTION, TIME_BETWEEN_SAMPLES,
};

use crate::fixed_func::silhouette_util::{
    deform_vertices_from_silhouette_radii, rotate_vertices, silhouette_radius_at_angle, silhouette_uvs_polar,
};
use crate::render::raylib::RaylibRenderer;
use asset_payload::SPHERE_PATH;
use raylib::consts::PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8;
use raylib::math::Vector3;
use raylib::models::{RaylibMesh, RaylibModel};
use raylib::texture::Texture2D;
use std::f32::consts::TAU;
use std::slice::from_raw_parts_mut;

pub fn generate_mesh_and_texcoord_samples_from_silhouette(
    renderer: &mut RaylibRenderer,
) -> (Vec<Vec<Vector3>>, Vec<Vec<f32>>) {
    let model = renderer.handle.load_model(&renderer.thread, SPHERE_PATH).unwrap();
    let vertices = model.meshes()[0].vertices();
    let mut mesh_samples = Vec::with_capacity(ROTATIONAL_SAMPLES_FOR_INV_PROJ);
    let mut texcoord_samples = Vec::with_capacity(ROTATIONAL_SAMPLES_FOR_INV_PROJ);
    for i in 0..ROTATIONAL_SAMPLES_FOR_INV_PROJ {
        let frame_time = i as f32 * TIME_BETWEEN_SAMPLES;
        let frame_rotation = -ANGULAR_VELOCITY * frame_time;
        let mut mesh_sample = vertices.to_vec();
        rotate_vertices(&mut mesh_sample, frame_rotation);
        let radii_normals = build_silhouette_radii_fast(frame_time);
        deform_vertices_from_silhouette_radii(&mut mesh_sample, &radii_normals);
        let mut texcoord_sample = Vec::with_capacity(mesh_sample.len() * 2);
        for vertex in mesh_sample.clone() {
            let (u, v) = silhouette_uvs_polar(vertex.x, vertex.y, &radii_normals);
            texcoord_sample.push(u);
            texcoord_sample.push(v);
        }
        texcoord_samples.push(texcoord_sample);
        rotate_vertices(&mut mesh_sample, -frame_rotation);
        mesh_samples.push(mesh_sample);
    }
    (mesh_samples, texcoord_samples)
}

pub fn build_silhouette_radii_fast(time: f32) -> Vec<f32> {
    let mut radii = Vec::with_capacity(SILHOUETTE_RADII_RESOLUTION);
    for i in 0..SILHOUETTE_RADII_RESOLUTION {
        let theta = (i as f32) * TAU / (SILHOUETTE_RADII_RESOLUTION as f32);
        radii.push(silhouette_radius_at_angle(theta, time));
    }
    let max_radius = radii.iter().cloned().fold(1e-6, f32::max);
    for radius in &mut radii {
        *radius /= max_radius;
    }
    radii
}

pub fn generate_silhouette_texture_fast(
    render: &mut RaylibRenderer,
    width: i32,
    height: i32,
    fade_frac: f32,
) -> Texture2D {
    use raylib::color::Color;
    use raylib::texture::Image;
    let mut img = Image::gen_image_color(width, height, Color::BLANK);
    let data = unsafe { from_raw_parts_mut(img.data as *mut u8, (width * height * 4) as usize) };

    let v_fade_start = (1.0 - fade_frac.clamp(0.0, 0.95)) * (height as f32 - 1.0);
    for y in 0..height {
        let v = y as f32;
        let alpha = if v < v_fade_start {
            1.0
        } else {
            1.0 - (v - v_fade_start) / ((height as f32 - 1.0) - v_fade_start + 1e-6)
        }
        .clamp(0.0, 1.0);

        let a = (alpha * 255.0) as u8;
        for x in 0..width {
            let i = 4 * (y as usize * width as usize + x as usize);
            data[i..i + 4].copy_from_slice(&[255, 255, 255, a]); // white * alpha
        }
    }
    img.set_format(PIXELFORMAT_UNCOMPRESSED_R8G8B8A8);
    render.handle.load_texture_from_image(&render.thread, &img).unwrap()
}
