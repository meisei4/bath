use raylib::models::{RaylibMesh, WeakMesh};
use std::collections::HashMap;
use std::slice::from_raw_parts;

pub fn weld_and_index_mesh_for_unfolding(mesh: &mut WeakMesh) {
    if !mesh.indices.is_null() {
        return;
    }
    let vertex_count = mesh.vertexCount as usize;
    if vertex_count == 0 {
        return;
    }
    let src_vertices = unsafe { from_raw_parts(mesh.vertices, vertex_count * 3) };
    let src_texcoords = unsafe { from_raw_parts(mesh.texcoords, vertex_count * 2) };

    let mut welded_vertices: Vec<f32> = Vec::with_capacity(vertex_count * 3);
    let mut welded_texcoords: Vec<f32> = Vec::with_capacity(vertex_count * 2);
    let mut indices: Vec<u16> = Vec::with_capacity(vertex_count);

    let mut key_to_index: HashMap<(i32, i32, i32, i32, i32), u16> = HashMap::with_capacity(vertex_count);

    for i in 0..vertex_count {
        let px = src_vertices[i * 3 + 0];
        let py = src_vertices[i * 3 + 1];
        let pz = src_vertices[i * 3 + 2];

        let (u, v) = (src_texcoords[i * 2 + 0], src_texcoords[i * 2 + 1]);
        // Quantized key so nearly-identical positions weld, but different UVs do NOT
        let key = (quantize(px), quantize(py), quantize(pz), quantize(u), quantize(v));

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

//TODO: no fucking idea how to get these fucking indices shit working fr a normal fucking mesh i just want all the fucking triangles fucking indexed jesus christ
pub fn weld_for_smoothing_topo(mesh: &mut WeakMesh) {
    let mut indices: Vec<u16> = Vec::with_capacity(mesh.vertices().len() * 3);
    // let mut map: HashMap<(i32, i32, i32), u16> = HashMap::with_capacity(mesh.vertices().len());
    // let mut map: HashMap<(f32, f32, f32), u16> = HashMap::with_capacity(mesh.vertices().len());
    let mut map: HashMap<String, u16> = HashMap::with_capacity(mesh.vertices().len());

    let mut next_id = 0u16;

    for (i, vertex) in mesh.vertices().iter().enumerate() {
        // let key = (quantize(vertex.x), quantize(vertex.y), quantize(vertex.z));
        // let key = (
        //     (vertex.x * 1e6) as i32,
        //     (vertex.y * 1e6) as i32,
        //     (vertex.z * 1e6) as i32,
        // );
        let key = format!("{}, {}, {}", vertex.x, vertex.y, vertex.z);
        let id = next_id;
        next_id += 1;
        indices.push(id);
        // let id = map.entry(key).or_insert_with(|| {
        //     let id = next_id;
        //     next_id += 1;
        //     id
        // });
        // indices.push(*id);
    }
    // let welded_count = next_id;
    // let mut neighbors: Vec<HashSet<usize>> = vec![HashSet::new(); welded_count];
    // for tri_start in (0..vertex_count).step_by(3) {
    //     let i0 = tri_start + 0;
    //     let i1 = tri_start + 1;
    //     let i2 = tri_start + 2;
    //     if i2 >= vertex_count {
    //         break;
    //     }
    //     let w0 = indices[i0] as usize;
    //     let w1 = indices[i1] as usize;
    //     let w2 = indices[i2] as usize;
    //     if w0 != w1 {
    //         neighbors[w0].insert(w1);
    //         neighbors[w1].insert(w0);
    //     }
    //     if w1 != w2 {
    //         neighbors[w1].insert(w2);
    //         neighbors[w2].insert(w1);
    //     }
    //     if w2 != w0 {
    //         neighbors[w2].insert(w0);
    //         neighbors[w0].insert(w2);
    //     }
    // }
    // let welded_neighbors: Vec<Vec<usize>> = neighbors.into_iter().map(|s| s.into_iter().collect()).collect();
    mesh.indices = Box::leak(indices.into_boxed_slice()).as_mut_ptr();
}

#[inline]
fn quantize(value: f32) -> i32 {
    let rounded = (value * 1e6);//.round();
    rounded.clamp(i32::MIN as f32, i32::MAX as f32) as i32
}
