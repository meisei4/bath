use crate::fixed_func::silhouette::{
    deform_vertices_with_radial_field, generate_silhouette_radial_field, interpolate_between_radial_field_elements,
    rotate_vertices, ANGULAR_VELOCITY, RADIAL_FIELD_SIZE, ROTATIONAL_SAMPLES_FOR_INV_PROJ, TIME_BETWEEN_SAMPLES,
};
use crate::fixed_func::topology::{Topology, WeldedEdge, WeldedVertex};
use crate::render::raylib::RaylibRenderer;
use asset_payload::SPHERE_PATH;
use raylib::color::Color;
use raylib::consts::PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8;
use raylib::math::glam::Vec2;
use raylib::models::{Model, RaylibMesh, RaylibModel};
use raylib::prelude::{Image, Texture2D};
use std::f32::consts::TAU;
use std::slice::from_raw_parts_mut;

pub fn collect_software_render_texture_samples(renderer: &mut RaylibRenderer) -> Vec<Vec<f32>> {
    let model = renderer.handle.load_model(&renderer.thread, SPHERE_PATH).unwrap();
    let vertices = model.meshes()[0].vertices();
    let mut texcoord_samples = Vec::with_capacity(ROTATIONAL_SAMPLES_FOR_INV_PROJ);
    for i in 0..ROTATIONAL_SAMPLES_FOR_INV_PROJ {
        let sample_time = i as f32 * TIME_BETWEEN_SAMPLES;
        let sample_rotation = -ANGULAR_VELOCITY * sample_time;
        let mut mesh_sample = vertices.to_vec();
        rotate_vertices(&mut mesh_sample, sample_rotation);
        let radial_field = generate_silhouette_radial_field(sample_time);
        deform_vertices_with_radial_field(&mut mesh_sample, &radial_field);
        let mut texcoord_sample = Vec::with_capacity(mesh_sample.len() * 2);
        for vertex in mesh_sample.clone() {
            let (s, t) = texcoords_at_radial_field_element(vertex.x, vertex.y, &radial_field);
            texcoord_sample.push(s);
            texcoord_sample.push(t);
        }
        texcoord_samples.push(texcoord_sample);
        rotate_vertices(&mut mesh_sample, -sample_rotation);
    }
    texcoord_samples
}

pub fn generate_silhouette_texture(render: &mut RaylibRenderer, width: i32, height: i32, fade_frac: f32) -> Texture2D {
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
            data[i..i + 4].copy_from_slice(&[255, 255, 255, a]);
        }
    }
    img.set_format(PIXELFORMAT_UNCOMPRESSED_R8G8B8A8);
    render.handle.load_texture_from_image(&render.thread, &img).unwrap()
}

pub fn interpolate_between_texture_samples(model: &mut Model, i_time: f32, texcoord_samples: &Vec<Vec<f32>>) {
    let target_mesh = &mut model.meshes_mut()[0];
    let duration = texcoord_samples.len() as f32 * TIME_BETWEEN_SAMPLES;
    let time = i_time % duration;
    let frame = time / TIME_BETWEEN_SAMPLES;
    let current_frame = frame.floor() as usize % texcoord_samples.len();
    let next_frame = (current_frame + 1) % texcoord_samples.len();
    let weight = frame.fract();
    let texcoords = unsafe { from_raw_parts_mut(target_mesh.as_mut().texcoords, target_mesh.vertices().len() * 2) };
    for ((dst_texcoord, src_texcoord), next_texcoord) in texcoords
        .iter_mut()
        .zip(texcoord_samples[current_frame].iter())
        .zip(texcoord_samples[next_frame].iter())
    {
        *dst_texcoord = *src_texcoord * (1.0 - weight) + *next_texcoord * weight;
    }
}

pub fn texcoords_at_radial_field_element(element_x: f32, element_y: f32, radial_field: &[f32]) -> (f32, f32) {
    let radial_field_element = element_y.atan2(element_x).rem_euclid(TAU);
    let radial_field_index = radial_field_element / TAU * RADIAL_FIELD_SIZE as f32;
    let lower_index = radial_field_index.floor() as usize % RADIAL_FIELD_SIZE;
    let s = lower_index as f32 / (RADIAL_FIELD_SIZE as f32);

    let radius = (element_x * element_x + element_y * element_y).sqrt();
    let interpolated_radius = interpolate_between_radial_field_elements(element_x, element_y, radial_field);
    let t = (radius / (interpolated_radius + 1e-6)).clamp(0.0, 1.0);

    (s, t)
}

pub fn generate_silhouette_texture_view_aware(
    render: &mut RaylibRenderer,
    topology: &Topology,
    texture_width: i32,
    texture_height: i32,
    fade_frac: f32,
) -> Texture2D {
    let segments = derive_silhouette_texcoords_from_topology(topology);
    let w = texture_width.max(1) as usize;
    let h = texture_height.max(1) as usize;
    let mut img = Image::gen_image_color(texture_width, texture_height, Color::WHITE);
    let data = unsafe { from_raw_parts_mut(img.data as *mut u8, w * h * 4) };

    let uv_to_px = |uv: Vec2| -> Vec2 {
        Vec2::new(
            uv.x.clamp(0.0, 1.0) * (w as f32 - 1.0),
            uv.y.clamp(0.0, 1.0) * (h as f32 - 1.0),
        )
    };
    let fade_px = (fade_frac.clamp(0.0, 1.0) * (w.min(h) as f32)).max(1.0);
    for (u0, u1) in segments {
        let a_px = uv_to_px(u0);
        let b_px = uv_to_px(u1);
        let bb_min = a_px.min(b_px) - Vec2::splat(fade_px);
        let bb_max = a_px.max(b_px) + Vec2::splat(fade_px);
        let x0 = bb_min.x.floor().max(0.0) as i32;
        let y0 = bb_min.y.floor().max(0.0) as i32;
        let x1 = bb_max.x.ceil().min((w - 1) as f32) as i32;
        let y1 = bb_max.y.ceil().min((h - 1) as f32) as i32;
        for py in y0..=y1 {
            for px in x0..=x1 {
                let p = Vec2::new(px as f32, py as f32);
                let d = distance_point_to_segment_uv(p, a_px, b_px);
                if d > fade_px {
                    continue;
                }
                let alpha01 = (d / fade_px).clamp(0.0, 1.0);
                let a = (alpha01 * 255.0).round() as u8;
                let i = 4 * (py as usize * w + px as usize);
                let current_a = data[i + 3];
                if a < current_a {
                    data[i + 3] = a;
                }
            }
        }
    }
    img.set_format(PIXELFORMAT_UNCOMPRESSED_R8G8B8A8);
    render.handle.load_texture_from_image(&render.thread, &img).unwrap()
}

pub fn derive_silhouette_texcoords_from_topology(topology: &Topology) -> Vec<(Vec2, Vec2)> {
    let neighbors_per_face = topology
        .neighbors_per_face
        .as_ref()
        .expect("neighbors_per_face missing");
    let face_texcoords = topology.face_texcoords.as_ref().expect("face_texcoords missing");
    let welded_vertices = topology
        .welded_vertices_per_face
        .as_ref()
        .expect("welded_vertices_per_face missing");
    let front_faces = topology.front_faces.as_ref().expect("front_faces missing");
    let mut segments = Vec::new();
    for face in 0..neighbors_per_face.len() {
        for edge in 0..3 {
            match neighbors_per_face[face][edge] {
                Some(neighbor) => {
                    if front_faces.contains(&face) ^ front_faces.contains(&neighbor) {
                        let (front_face, front_face_edge) = if front_faces.contains(&face) {
                            (face, edge)
                        } else {
                            let welded_edge = welded_edge_for_welded_face_edge(&welded_vertices[face], edge);
                            let neighbor_edge =
                                local_edge_index_for_welded_edge(&welded_vertices[neighbor], welded_edge);
                            (neighbor, neighbor_edge)
                        };
                        let (a, b) = texcoords_to_local_edge(&face_texcoords[front_face], front_face_edge);
                        segments.push((a, b));
                    }
                },
                None => {
                    if front_faces.contains(&face) {
                        let (a, b) = texcoords_to_local_edge(&face_texcoords[face], edge);
                        segments.push((a, b));
                    }
                },
            }
        }
    }
    segments
}

#[inline]
fn distance_point_to_segment_uv(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    let ap = p - a;
    let ab_len2 = ab.length_squared().max(1e-12);
    let t = (ap.dot(ab) / ab_len2).clamp(0.0, 1.0);
    let closest = a + ab * t;
    (p - closest).length()
}

#[inline]
fn texcoords_to_local_edge(texcoords: &[Vec2; 3], edge: usize) -> (Vec2, Vec2) {
    match edge {
        0 => (texcoords[0], texcoords[1]), // AB
        1 => (texcoords[1], texcoords[2]), // BC
        _ => (texcoords[2], texcoords[0]), // CA
    }
}

#[inline]
fn welded_edge_for_welded_face_edge(welded_face: &[WeldedVertex; 3], edge: usize) -> WeldedEdge {
    let (a, b) = match edge {
        0 => (welded_face[0], welded_face[1]),
        1 => (welded_face[1], welded_face[2]),
        _ => (welded_face[2], welded_face[0]),
    };
    WeldedEdge::new(a, b)
}

#[inline]
fn local_edge_index_for_welded_edge(welded: &[WeldedVertex; 3], key: WeldedEdge) -> usize {
    let candidates = [
        WeldedEdge::new(welded[0], welded[1]),
        WeldedEdge::new(welded[1], welded[2]),
        WeldedEdge::new(welded[2], welded[0]),
    ];
    for (i, e) in candidates.iter().enumerate() {
        if *e == key {
            return i;
        }
    }
    0
}
