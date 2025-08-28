use raylib::math::glam::{Vec2, Vec3};
use raylib::math::Vector3;
use raylib::models::{RaylibMesh, WeakMesh};
use std::collections::HashMap;
use std::slice::from_raw_parts;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct WeldedVertex {
    pub id: u32,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Face {
    pub id: usize,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct WeldedEdge {
    pub vertex_a: WeldedVertex,
    pub vertex_b: WeldedVertex,
}

impl WeldedEdge {
    pub fn new(node_a: WeldedVertex, node_b: WeldedVertex) -> Self {
        if node_a.id <= node_b.id {
            WeldedEdge {
                vertex_a: node_a,
                vertex_b: node_b,
            }
        } else {
            WeldedEdge {
                vertex_a: node_b,
                vertex_b: node_a,
            }
        }
    }
}

pub struct WeldedMesh {
    pub original_vertices: Vec<[Vec3; 3]>,
    pub welded_faces: Vec<[WeldedVertex; 3]>,
    pub texcoords: Vec<[Vec2; 3]>,
}

pub fn weld_mesh(mesh: &WeakMesh) -> WeldedMesh {
    let triangle_count = mesh.triangleCount as usize;
    let vertex_count = mesh.vertexCount as usize;
    let src_vertices = mesh.vertices();
    let dummy_uvs: Vec<f32>;
    let src_texcoords: &[f32] = if mesh.texcoords.is_null() {
        dummy_uvs = vec![0.0; 2 * vertex_count];
        &dummy_uvs
    } else {
        unsafe { from_raw_parts(mesh.texcoords, 2 * vertex_count) }
    };
    let src_indices = if mesh.indices.is_null() {
        debug_assert!(vertex_count % 3 == 0, "non-indexed mesh must be triangle soup");
        (0..vertex_count as u16).collect()
    } else {
        unsafe { from_raw_parts(mesh.indices, 3 * triangle_count) }.to_vec()
    };

    let use_indices_for_weld = !mesh.indices.is_null();
    //TODO: use the UTIL FUNCTIONS!!!!!!!
    let mut quantized_vertex_to_welded_vertex_map: HashMap<(i32, i32, i32), WeldedVertex> = HashMap::new();
    let mut index_to_welded_vertex_map: Option<HashMap<usize, WeldedVertex>> = if use_indices_for_weld {
        Some(HashMap::new())
    } else {
        None
    };

    let mut welded_vertices_count = 0;
    let mut welded_faces = Vec::with_capacity(src_indices.len() / 3);
    let mut face_texcoords = Vec::with_capacity(src_indices.len() / 3);
    let mut face_vertices = Vec::with_capacity(src_indices.len() / 3);
    for face_index in 0..(src_indices.len() / 3) {
        let mut face_welded_vertices = [WeldedVertex { id: 0 }; 3];
        let mut face_texcoord = [Vec2::ZERO; 3];
        let mut face_vertex = [Vec3::ZERO; 3];
        for i in 0..3 {
            let vertex_index = src_indices[face_index * 3 + i] as usize;
            let vertex = src_vertices[vertex_index];
            let welded_vertex_id = if let Some(ref mut map) = index_to_welded_vertex_map {
                *map.entry(vertex_index).or_insert_with(|| {
                    let weld_id = WeldedVertex {
                        id: welded_vertices_count,
                    };
                    welded_vertices_count += 1;
                    weld_id
                })
            } else {
                *quantized_vertex_to_welded_vertex_map
                    .entry((quantize(vertex.x), quantize(vertex.y), quantize(vertex.z)))
                    .or_insert_with(|| {
                        let weld_id = WeldedVertex {
                            id: welded_vertices_count,
                        };
                        welded_vertices_count += 1;
                        weld_id
                    })
            };

            face_welded_vertices[i] = welded_vertex_id;
            face_vertex[i] = vertex;
            let s = *src_texcoords.get(vertex_index * 2 + 0).unwrap();
            let t = *src_texcoords.get(vertex_index * 2 + 1).unwrap();
            face_texcoord[i] = Vec2::new(s, t);
        }
        welded_faces.push(face_welded_vertices);
        face_texcoords.push(face_texcoord);
        face_vertices.push(face_vertex);
    }

    WeldedMesh {
        original_vertices: face_vertices,
        welded_faces,
        texcoords: face_texcoords,
    }
}

pub fn collect_welded_vertices(original_vertices: &[Vector3]) -> Vec<u32> {
    let mut welded_vertices_to_faces_map: HashMap<(i32, i32, i32), u32> = HashMap::new();
    let mut next_wid: u32 = 0;
    let mut welded_id_per_vertex = vec![0u32; original_vertices.len()];
    for (i, v) in original_vertices.iter().enumerate() {
        let key = (quantize(v.x), quantize(v.y), quantize(v.z));
        let wid = *welded_vertices_to_faces_map.entry(key).or_insert_with(|| {
            let id = next_wid;
            next_wid += 1;
            id
        });
        welded_id_per_vertex[i] = wid;
    }
    welded_id_per_vertex
}

pub fn build_edge_owner_map(faces: &[[u32; 3]], welded_vertices: &[u32]) -> HashMap<WeldedEdge, usize> {
    let mut edge_owner: HashMap<WeldedEdge, usize> = HashMap::new();
    for (face, [vertex_a_index, vertex_b_index, vertex_c_index]) in faces.iter().copied().enumerate() {
        let welded_vertex_a = WeldedVertex {
            id: welded_vertices[vertex_a_index as usize],
        };
        let welded_vertex_b = WeldedVertex {
            id: welded_vertices[vertex_b_index as usize],
        };
        let welded_vertex_c = WeldedVertex {
            id: welded_vertices[vertex_c_index as usize],
        };
        for (w0, w1) in [
            (welded_vertex_a, welded_vertex_b),
            (welded_vertex_b, welded_vertex_c),
            (welded_vertex_c, welded_vertex_a),
        ] {
            edge_owner.entry(WeldedEdge::new(w0, w1)).or_insert(face);
        }
    }
    edge_owner
}

#[inline]
pub fn quantize(x: f32) -> i32 {
    const WELD_VERTEX_EPSILON: f32 = 1e-5; // e-1 and up works, 0 goes crazy
    (x / WELD_VERTEX_EPSILON).round() as i32
}

#[inline]
pub fn edge_opposing_vertex(edge: (u8, u8)) -> u8 {
    3 - (edge.0 + edge.1)
}

#[inline]
pub fn welded_eq(a: WeldedVertex, b: WeldedVertex) -> bool {
    a.id == b.id
}

#[inline]
pub fn face_normal(a: Vec3, b: Vec3, c: Vec3) -> Vec3 {
    (b - a).cross(c - a).normalize_or_zero()
}
