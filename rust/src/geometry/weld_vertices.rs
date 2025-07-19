use raylib::ffi::{MemAlloc, MemFree};
use raylib::models::{RaylibMesh, WeakMesh};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::ptr::null_mut;

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
    let vc = mesh.vertexCount as usize;
    let pos = unsafe { std::slice::from_raw_parts(mesh.vertices, vc * 3) };
    let tx = if mesh.texcoords.is_null() {
        None
    } else {
        Some(unsafe { std::slice::from_raw_parts(mesh.texcoords, vc * 2) })
    };
    let nm = if mesh.normals.is_null() {
        None
    } else {
        Some(unsafe { std::slice::from_raw_parts(mesh.normals, vc * 3) })
    };
    let scale = 1.0 / eps.max(1e-12);
    let mut map: HashMap<QKey, u16> = HashMap::with_capacity(vc / 2 + 1);
    let mut new_pos = Vec::with_capacity(vc * 3 / 2 + 3);
    let mut new_tx = Vec::with_capacity(tx.map(|_| vc).unwrap_or(0));
    let mut new_nm = Vec::with_capacity(nm.map(|_| vc * 3 / 2 + 3).unwrap_or(0));
    let mut out_idx: Vec<u16> = Vec::with_capacity(vc);
    for i in 0..vc {
        let p = &pos[i * 3..i * 3 + 3];
        let k = QKey(qf(p[0], scale), qf(p[1], scale), qf(p[2], scale));
        let id = if let Some(&e) = map.get(&k) {
            e
        } else {
            let nid = (new_pos.len() / 3) as u16;
            new_pos.extend_from_slice(p);
            if let Some(t) = tx {
                new_tx.extend_from_slice(&t[i * 2..i * 2 + 2]);
            }
            if let Some(n) = nm {
                new_nm.extend_from_slice(&n[i * 3..i * 3 + 3]);
            }
            map.insert(k, nid);
            nid
        };
        out_idx.push(id);
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

        mesh.vertexCount = (new_pos.len() / 3) as i32;
        mesh.triangleCount = (out_idx.len() / 3) as i32;
        mesh.vertices = MemAlloc((new_pos.len() * size_of::<f32>()) as u32) as *mut f32;
        std::ptr::copy_nonoverlapping(new_pos.as_ptr(), mesh.vertices, new_pos.len());

        if !new_tx.is_empty() {
            mesh.texcoords = MemAlloc((new_tx.len() * size_of::<f32>()) as u32) as *mut f32;
            std::ptr::copy_nonoverlapping(new_tx.as_ptr(), mesh.texcoords, new_tx.len());
        } else {
            mesh.texcoords = null_mut();
        }

        if !new_nm.is_empty() {
            mesh.normals = MemAlloc((new_nm.len() * size_of::<f32>()) as u32) as *mut f32;
            std::ptr::copy_nonoverlapping(new_nm.as_ptr(), mesh.normals, new_nm.len());
        } else {
            mesh.normals = null_mut();
        }

        mesh.indices = MemAlloc((out_idx.len() * size_of::<u16>()) as u32) as *mut u16;
        std::ptr::copy_nonoverlapping(out_idx.as_ptr(), mesh.indices, out_idx.len());
        mesh.upload(false);
    }
}
