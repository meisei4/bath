use ffi::MemAlloc;
use raylib::ffi;
use raylib::models::{Model, RaylibMesh, RaylibModel, WeakMesh};
use std::intrinsics::copy_nonoverlapping;
use std::mem::size_of;

const MAX_VERTICES: usize = 1000;

pub unsafe fn deep_copy_model(src_model: &Model) -> Model {
    let src = src_model.as_ref();
    let mut dst_raw: ffi::Model = std::mem::zeroed();
    dst_raw.transform = src.transform;
    dst_raw.meshCount = src.meshCount;
    dst_raw.materialCount = src.materialCount;
    dst_raw.meshes = MemAlloc((src.meshCount as usize * size_of::<ffi::Mesh>()) as u32) as *mut ffi::Mesh;
    dst_raw.materials =
        MemAlloc((src.materialCount as usize * size_of::<ffi::Material>()) as u32) as *mut ffi::Material;
    dst_raw.meshMaterial = MemAlloc((src.meshCount as usize * size_of::<i32>()) as u32) as *mut i32;

    std::ptr::copy_nonoverlapping(src.meshMaterial, dst_raw.meshMaterial, src.meshCount as usize);
    dst_raw.bones = std::ptr::null_mut();
    dst_raw.bindPose = std::ptr::null_mut();
    dst_raw.boneCount = 0;
    for i in 0..src.meshCount as usize {
        let src_mesh = *src.meshes.add(i);
        let mut dst_mesh: ffi::Mesh = std::mem::zeroed();
        dst_mesh.vertexCount = src_mesh.vertexCount;
        dst_mesh.triangleCount = src_mesh.triangleCount;
        unsafe fn copy_f32_array(ptr: &mut *mut f32, src: *const f32, count: usize) {
            if !src.is_null() && count > 0 {
                let bytes = count * size_of::<f32>();
                *ptr = MemAlloc(bytes as u32) as *mut f32;
                std::ptr::copy_nonoverlapping(src, *ptr, count);
            } else {
                *ptr = std::ptr::null_mut();
            }
        }
        unsafe fn copy_u16_array(ptr: *mut *mut u16, src: *const u16, count: usize) {
            if !src.is_null() && count > 0 {
                let bytes = count * size_of::<u16>();
                *ptr = MemAlloc(bytes as u32) as *mut u16;
                std::ptr::copy_nonoverlapping(src, *ptr, count);
            } else {
                *ptr = std::ptr::null_mut();
            }
        }
        unsafe fn copy_u8_array(ptr: *mut *mut u8, src: *const u8, count: usize) {
            if !src.is_null() && count > 0 {
                let bytes = count * size_of::<u8>();
                *ptr = MemAlloc(bytes as u32) as *mut u8;
                std::ptr::copy_nonoverlapping(src, *ptr, count);
            } else {
                *ptr = std::ptr::null_mut();
            }
        }

        copy_f32_array(
            &mut dst_mesh.vertices,
            src_mesh.vertices,
            src_mesh.vertexCount as usize * 3,
        );
        copy_f32_array(
            &mut dst_mesh.texcoords,
            src_mesh.texcoords,
            src_mesh.vertexCount as usize * 2,
        );
        copy_f32_array(
            &mut dst_mesh.normals,
            src_mesh.normals,
            src_mesh.vertexCount as usize * 3,
        );
        copy_f32_array(
            &mut dst_mesh.tangents,
            src_mesh.tangents,
            src_mesh.vertexCount as usize * 4,
        );
        copy_u8_array(&mut dst_mesh.colors, src_mesh.colors as *const u8, 0);
        copy_u16_array(
            &mut dst_mesh.indices,
            src_mesh.indices,
            (src_mesh.triangleCount as usize) * 3,
        );

        dst_mesh.animVertices = std::ptr::null_mut();
        dst_mesh.animNormals = std::ptr::null_mut();
        dst_mesh.boneIds = std::ptr::null_mut();
        dst_mesh.boneWeights = std::ptr::null_mut();
        dst_mesh.vaoId = 0;
        dst_mesh.vboId = MemAlloc((MAX_VERTICES * size_of::<u32>()) as u32) as *mut u32;
        for k in 0..MAX_VERTICES {
            *dst_mesh.vboId.offset(k as isize) = 0;
        }
        *dst_raw.meshes.add(i) = dst_mesh;
    }
    for i in 0..src.materialCount as usize {
        let mut mat = *src.materials.add(i);
        let maps_slice = std::slice::from_raw_parts_mut(mat.maps, raylib::consts::MAX_MATERIAL_MAPS as usize);
        for map in maps_slice.iter_mut() {
            if map.texture.id != 0 {
                map.texture.id = 0;
            }
        }
        *dst_raw.materials.add(i) = mat;
    }

    let mut dst_model = Model::from_raw(dst_raw);
    for mesh in dst_model.meshes_mut().iter_mut() {
        mesh.upload(false);
    }
    dst_model
}

pub unsafe fn update_texcoords(mesh: &mut WeakMesh, texcoords: &[f32]) {
    assert!(
        !mesh.as_ref().texcoords.is_null(),
        "IAN !update_texcoords: mesh has no existing texcoord buffer (NULL)"
    );
    let vertex_count = mesh.vertexCount as usize;
    let expected_len = vertex_count * 2;
    assert_eq!(
        texcoords.len(),
        expected_len,
        "IAN !update_texcoords: length mismatch (got {}, expected {})",
        texcoords.len(),
        expected_len
    );
    assert_eq!(
        (mesh.as_ref().texcoords as usize) % align_of::<f32>(),
        0,
        "IA N! update_texcoords: texcoord pointer alignment invalid"
    );
    copy_nonoverlapping(texcoords.as_ptr(), mesh.texcoords, texcoords.len());
    mesh.upload(false);
}
