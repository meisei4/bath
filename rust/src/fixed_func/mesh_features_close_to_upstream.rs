use raylib::models::{RaylibMesh, WeakMesh};

pub fn reverse_vertex_winding(mesh: &mut WeakMesh) {
    if let Some(indices) = mesh.indices_mut() {
        for triangle in indices.chunks_exact_mut(3) {
            triangle.swap(1, 2);
        }
    } else {
        let vertex_count = mesh.as_ref().vertexCount as usize;
        assert_eq!(
            vertex_count % 3,
            0,
            "reverse_vertex_winding: vertices not multiple of 3"
        );

        for triangle in mesh.vertices_mut().chunks_exact_mut(3) {
            triangle.swap(1, 2);
        }
        if let Some(normals) = mesh.normals_mut() {
            assert_eq!(
                normals.len() % 3,
                0,
                "reverse_vertex_winding: normals not multiple of 3"
            );
            for triangle in normals.chunks_exact_mut(3) {
                triangle.swap(1, 2);
            }
        }
        if let Some(tangents) = mesh.tangents_mut() {
            assert_eq!(
                tangents.len() % 3,
                0,
                "reverse_vertex_winding: tangents not multiple of 3"
            );
            for triangle in tangents.chunks_exact_mut(3) {
                triangle.swap(1, 2);
            }
        }
        if let Some(texcoords) = mesh.texcoords_mut() {
            assert_eq!(
                texcoords.len() % 3,
                0,
                "reverse_vertex_winding: texcoords not multiple of 3"
            );
            for triangle in texcoords.chunks_exact_mut(3) {
                triangle.swap(1, 2);
            }
        }
        if let Some(texcoords2) = mesh.texcoords2_mut() {
            assert_eq!(
                texcoords2.len() % 3,
                0,
                "reverse_vertex_winding: texcoords2 not multiple of 3"
            );
            for triangle in texcoords2.chunks_exact_mut(3) {
                triangle.swap(1, 2);
            }
        }
        if let Some(colors) = mesh.colors_mut() {
            assert_eq!(colors.len() % 3, 0, "reverse_vertex_winding: colors not multiple of 3");
            for triangle in colors.chunks_exact_mut(3) {
                triangle.swap(1, 2);
            }
        }
    }

    unsafe {
        mesh.upload(true);
    }
}

//TODO: this is even crazier where does this go in the builder please help
pub fn reverse_vertex_winding_old(mesh: &mut WeakMesh) {
    if let Some(indices) = mesh.indices_mut() {
        for triangle in indices.chunks_exact_mut(3) {
            triangle.swap(1, 2);
        }
    } else {
        //TODO: vertexCount??? really??
        let mut indices = Vec::<u16>::with_capacity(mesh.vertexCount as usize);
        for t in 0..mesh.triangleCount {
            let base = (t * 3) as u16;
            indices.extend_from_slice(&[base, base + 2, base + 1]);
        }
        let mesh = mesh.as_mut();
        // TODO: this is dumb please discuss fix based on soundness or ensure guarantees that exist or are needed from upstream

        mesh.indices = Box::leak(indices.into_boxed_slice()).as_mut_ptr();
    }
}
