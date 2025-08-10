use raylib::math::Vector3;
use raylib::models::WeakMesh;
use std::collections::{HashMap, HashSet};
use std::slice::from_raw_parts;

#[inline]
fn quantize_for_unfolding(value: f32, scale: f32) -> i32 {
    if !value.is_finite() {
        return 0;
    }
    let rounded = (value * scale).round();
    rounded.clamp(i32::MIN as f32, i32::MAX as f32) as i32
}

pub fn weld_and_index_mesh_for_unfolding(mesh: &mut WeakMesh, position_epsilon: f32) {
    if !mesh.indices.is_null() {
        return;
    }
    let vertex_count = mesh.vertexCount as usize;
    if vertex_count == 0 {
        return;
    }
    let src_vertices = unsafe { from_raw_parts(mesh.vertices, vertex_count * 3) };
    let src_texcoords = unsafe { from_raw_parts(mesh.texcoords, vertex_count * 2) };

    // Quantization scales (vertex uses epsilon; texcoords uses a large scale to keep seams distinct)
    let vertex_scale = 1.0 / position_epsilon.max(1e-9);
    let texcoord_scale = 1e6;

    let mut welded_vertices: Vec<f32> = Vec::with_capacity(vertex_count * 3);
    let mut welded_texcoords: Vec<f32> = Vec::with_capacity(vertex_count * 2);
    let mut indices: Vec<u16> = Vec::with_capacity(vertex_count);

    // (qx, qy, qz, qu, qv) -> new vertex index
    let mut key_to_index: HashMap<(i32, i32, i32, i32, i32), u16> = HashMap::with_capacity(vertex_count);

    for i in 0..vertex_count {
        let px = src_vertices[i * 3 + 0];
        let py = src_vertices[i * 3 + 1];
        let pz = src_vertices[i * 3 + 2];

        let (u, v) = (src_texcoords[i * 2 + 0], src_texcoords[i * 2 + 1]);
        // Quantized key so nearly-identical positions weld, but different UVs do NOT
        let key = (
            quantize_for_unfolding(px, vertex_scale),
            quantize_for_unfolding(py, vertex_scale),
            quantize_for_unfolding(pz, vertex_scale),
            quantize_for_unfolding(u, texcoord_scale),
            quantize_for_unfolding(v, texcoord_scale),
        );

        let new_index = *key_to_index.entry(key).or_insert_with(|| {
            let out_index = (welded_vertices.len() / 3) as u16;
            welded_vertices.extend_from_slice(&[px, py, pz]);
            welded_texcoords.extend_from_slice(&[u, v]);
            out_index
        });

        indices.push(new_index);
    }
    mesh.vertexCount = (welded_vertices.len() / 3) as i32;
    mesh.triangleCount = (indices.len() / 3) as i32;
    mesh.vertices = Box::leak(welded_vertices.into_boxed_slice()).as_mut_ptr();
    mesh.texcoords = Box::leak(welded_texcoords.into_boxed_slice()).as_mut_ptr();
    mesh.indices = Box::leak(indices.into_boxed_slice()).as_mut_ptr();
}

pub static mut SMOOTH_OLD_TO_WELDED: Option<Vec<usize>> = None;
pub static mut SMOOTH_WELDED_NEIGHBORS: Option<Vec<Vec<usize>>> = None;
static POSITION_WELD_EPSILON: f32 = 1e-6;

#[inline]
fn quantize_for_smoothing(v: f32, inv_eps: f32) -> i32 {
    if !v.is_finite() {
        0
    } else {
        (v * inv_eps).round() as i32
    }
}

pub fn weld_for_smoothing_topo(vertices_current_frame: &[Vector3]) {
    unsafe {
        if SMOOTH_OLD_TO_WELDED.is_some() && SMOOTH_WELDED_NEIGHBORS.is_some() {
            return;
        }
        let inv_eps = 1.0 / POSITION_WELD_EPSILON.max(1e-12);
        let mut map: HashMap<(i32, i32, i32), usize> = HashMap::with_capacity(vertices_current_frame.len());
        let mut old_to_welded = vec![0usize; vertices_current_frame.len()];
        let mut next_id = 0usize;
        for (i, p) in vertices_current_frame.iter().enumerate() {
            let key = (
                quantize_for_smoothing(p.x, inv_eps),
                quantize_for_smoothing(p.y, inv_eps),
                quantize_for_smoothing(p.z, inv_eps),
            );
            let id = *map.entry(key).or_insert_with(|| {
                let id = next_id;
                next_id += 1;
                id
            });
            old_to_welded[i] = id;
        }
        let welded_count = next_id;
        let mut nb: Vec<HashSet<usize>> = vec![HashSet::new(); welded_count];
        for tri_start in (0..vertices_current_frame.len()).step_by(3) {
            let i0 = tri_start + 0;
            let i1 = tri_start + 1;
            let i2 = tri_start + 2;
            if i2 >= vertices_current_frame.len() {
                break;
            }
            let w0 = old_to_welded[i0];
            let w1 = old_to_welded[i1];
            let w2 = old_to_welded[i2];
            if w0 != w1 {
                nb[w0].insert(w1);
                nb[w1].insert(w0);
            }
            if w1 != w2 {
                nb[w1].insert(w2);
                nb[w2].insert(w1);
            }
            if w2 != w0 {
                nb[w2].insert(w0);
                nb[w0].insert(w2);
            }
        }
        let welded_neighbors: Vec<Vec<usize>> = nb.into_iter().map(|s| s.into_iter().collect()).collect();
        SMOOTH_OLD_TO_WELDED = Some(old_to_welded);
        SMOOTH_WELDED_NEIGHBORS = Some(welded_neighbors);
    }
}

pub fn aggregate_to_welded(old_to_welded: &[usize], per_original: &[f32]) -> Vec<f32> {
    let welded_len = 1 + old_to_welded.iter().copied().max().unwrap_or(0);
    let mut sum = vec![0.0f32; welded_len];
    let mut count = vec![0u32; welded_len];
    for (i, &w) in old_to_welded.iter().enumerate() {
        sum[w] += per_original[i];
        count[w] += 1;
    }
    for weld in 0..welded_len {
        if count[weld] > 0 {
            sum[weld] /= count[weld] as f32;
        }
    }
    sum
}
