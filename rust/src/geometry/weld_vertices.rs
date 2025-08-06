use raylib::ffi::{MemAlloc, MemFree};
use raylib::models::WeakMesh;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::ptr::null_mut;

//TODO: this whole fucking file is dumb and i dont understand it anymore, i forgot the purpose other than indexing for drawing only unique triangles in the unfold process
#[derive(Eq, Clone)]
struct QKey(i32, i32, i32);
impl PartialEq for QKey {
    fn eq(&self, o: &Self) -> bool {
        self.0 == o.0 && self.1 == o.1 && self.2 == o.2
    }
}
impl Hash for QKey {
    fn hash<H: Hasher>(&self, h: &mut H) {
        self.0.hash(h);
        self.1.hash(h);
        self.2.hash(h);
    }
}

#[inline]
fn qf(v: f32, s: f32) -> i32 {
    if !v.is_finite() {
        return 0;
    }
    let r = (v * s).round();
    if r > i32::MAX as f32 {
        i32::MAX
    } else if r < i32::MIN as f32 {
        i32::MIN
    } else {
        r as i32
    }
}

pub fn weld_and_index_mesh(mesh: &mut WeakMesh, eps: f32) {
    if !mesh.indices.is_null() {
        return;
    }
    let vertex_count = mesh.vertexCount as usize;
    let src_vertices = unsafe { std::slice::from_raw_parts(mesh.vertices, vertex_count * 3) };
    let src_texcoords = if mesh.texcoords.is_null() {
        None
    } else {
        Some(unsafe { std::slice::from_raw_parts(mesh.texcoords, vertex_count * 2) })
    };
    let src_normals = if mesh.normals.is_null() {
        None
    } else {
        Some(unsafe { std::slice::from_raw_parts(mesh.normals, vertex_count * 3) })
    };
    let scale = 1.0 / eps.max(1e-12);
    let mut map: HashMap<QKey, u16> = HashMap::with_capacity(vertex_count / 2 + 1);
    let mut dst_vertices = Vec::with_capacity(vertex_count * 3 / 2 + 3);
    let mut dst_texcoords = Vec::with_capacity(src_texcoords.map(|_| vertex_count).unwrap_or(0));
    let mut dst_normals = Vec::with_capacity(src_normals.map(|_| vertex_count * 3 / 2 + 3).unwrap_or(0));
    let mut welded_indices: Vec<u16> = Vec::with_capacity(vertex_count);
    for i in 0..vertex_count {
        let p = &src_vertices[i * 3..i * 3 + 3];
        let k = QKey(qf(p[0], scale), qf(p[1], scale), qf(p[2], scale));
        let id = if let Some(&e) = map.get(&k) {
            e
        } else {
            let nid = (dst_vertices.len() / 3) as u16;
            dst_vertices.extend_from_slice(p);
            if let Some(t) = src_texcoords {
                dst_texcoords.extend_from_slice(&t[i * 2..i * 2 + 2]);
            }
            if let Some(n) = src_normals {
                dst_normals.extend_from_slice(&n[i * 3..i * 3 + 3]);
            }
            map.insert(k, nid);
            nid
        };
        welded_indices.push(id);
    }
    unsafe {
        MemFree(mesh.vertices as *mut _);
        if !mesh.texcoords.is_null() {
            MemFree(mesh.texcoords as *mut _);
        }
        if !mesh.normals.is_null() {
            MemFree(mesh.normals as *mut _);
        }
        if !mesh.tangents.is_null() {
            MemFree(mesh.tangents as *mut _);
            mesh.tangents = null_mut();
        }
        if !mesh.colors.is_null() {
            MemFree(mesh.colors as *mut _);
            mesh.colors = null_mut();
        }

        mesh.vertexCount = (dst_vertices.len() / 3) as i32;
        mesh.triangleCount = (welded_indices.len() / 3) as i32;
        mesh.vertices = MemAlloc((dst_vertices.len() * size_of::<f32>()) as u32) as *mut f32;
        std::ptr::copy_nonoverlapping(dst_vertices.as_ptr(), mesh.vertices, dst_vertices.len());

        if !dst_texcoords.is_empty() {
            mesh.texcoords = MemAlloc((dst_texcoords.len() * size_of::<f32>()) as u32) as *mut f32;
            std::ptr::copy_nonoverlapping(dst_texcoords.as_ptr(), mesh.texcoords, dst_texcoords.len());
        } else {
            mesh.texcoords = null_mut();
        }

        if !dst_normals.is_empty() {
            mesh.normals = MemAlloc((dst_normals.len() * size_of::<f32>()) as u32) as *mut f32;
            std::ptr::copy_nonoverlapping(dst_normals.as_ptr(), mesh.normals, dst_normals.len());
        } else {
            mesh.normals = null_mut();
        }

        mesh.indices = MemAlloc((welded_indices.len() * size_of::<u16>()) as u32) as *mut u16;
        std::ptr::copy_nonoverlapping(welded_indices.as_ptr(), mesh.indices, welded_indices.len());
    }
}
