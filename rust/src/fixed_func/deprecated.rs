use crate::fixed_func::silhouette::{
    deform_vertices_from_radial_field, generate_silhouette_radial_field, rotate_vertices, silhouette_uvs_polar,
    ANGULAR_VELOCITY, ROTATIONAL_SAMPLES_FOR_INV_PROJ, TIME_BETWEEN_SAMPLES,
};
use crate::fixed_func::topology::{Topology, WeldedEdge, WeldedVertex};
use crate::render::raylib::RaylibRenderer;
use asset_payload::SPHERE_PATH;
use raylib::color::Color;
use raylib::consts::PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8;
use raylib::math::glam::Vec2;
use raylib::math::Vector3;
use raylib::models::{Model, RaylibMesh, RaylibModel};
use raylib::prelude::{Image, Texture2D};
use std::slice::from_raw_parts_mut;

pub fn collect_deformed_mesh_and_texture_samples(renderer: &mut RaylibRenderer) -> (Vec<Vec<Vector3>>, Vec<Vec<f32>>) {
    let model = renderer.handle.load_model(&renderer.thread, SPHERE_PATH).unwrap();
    let vertices = model.meshes()[0].vertices();
    let mut mesh_samples = Vec::with_capacity(ROTATIONAL_SAMPLES_FOR_INV_PROJ);
    let mut texcoord_samples = Vec::with_capacity(ROTATIONAL_SAMPLES_FOR_INV_PROJ);
    for i in 0..ROTATIONAL_SAMPLES_FOR_INV_PROJ {
        let frame_time = i as f32 * TIME_BETWEEN_SAMPLES;
        let frame_rotation = -ANGULAR_VELOCITY * frame_time;
        let mut mesh_sample = vertices.to_vec();
        rotate_vertices(&mut mesh_sample, frame_rotation);
        let radial_field = generate_silhouette_radial_field(frame_time);
        deform_vertices_from_radial_field(&mut mesh_sample, &radial_field);
        let mut texcoord_sample = Vec::with_capacity(mesh_sample.len() * 2);
        for vertex in mesh_sample.clone() {
            let (u, v) = silhouette_uvs_polar(vertex.x, vertex.y, &radial_field);
            texcoord_sample.push(u);
            texcoord_sample.push(v);
        }
        texcoord_samples.push(texcoord_sample);
        rotate_vertices(&mut mesh_sample, -frame_rotation);
        mesh_samples.push(mesh_sample);
    }
    (mesh_samples, texcoord_samples)
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

pub fn interpolate_between_deformed_meshes_and_textures(
    model: &mut Model,
    i_time: f32,
    mesh_samples: &Vec<Vec<Vector3>>,
    texcoord_samples: &Vec<Vec<f32>>,
) {
    let target_mesh = &mut model.meshes_mut()[0];
    let duration = mesh_samples.len() as f32 * TIME_BETWEEN_SAMPLES;
    let time = i_time % duration;
    let frame = time / TIME_BETWEEN_SAMPLES;
    let current_frame = frame.floor() as usize % mesh_samples.len();
    let next_frame = (current_frame + 1) % mesh_samples.len();
    let weight = frame.fract();
    let vertices = target_mesh.vertices_mut();
    for ((dst_vertex, src_vertex), next_vertex) in vertices
        .iter_mut()
        .zip(mesh_samples[current_frame].iter())
        .zip(mesh_samples[next_frame].iter())
    {
        dst_vertex.x = src_vertex.x * (1.0 - weight) + next_vertex.x * weight;
        dst_vertex.y = src_vertex.y * (1.0 - weight) + next_vertex.y * weight;
        dst_vertex.z = src_vertex.z * (1.0 - weight) + next_vertex.z * weight;
    }
    let texcoords = unsafe { from_raw_parts_mut(target_mesh.as_mut().texcoords, target_mesh.vertices().len() * 2) };
    for ((dst_texcoord, src_texcoord), next_texcoord) in texcoords
        .iter_mut()
        .zip(texcoord_samples[current_frame].iter())
        .zip(texcoord_samples[next_frame].iter())
    {
        *dst_texcoord = *src_texcoord * (1.0 - weight) + *next_texcoord * weight;
    }
}

pub fn generate_view_silhouette_texture_uvspace(
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
