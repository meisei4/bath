use std::env;
use std::f32::consts::PI;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

const DEFAULT_TARGET_TRIS: usize = 500;
const MESHDUMP_DIR: &str = "/Users/adduser/meshdump";
const UV_TILE_SCALE: f32 = 3.0;

type Mat4 = [[f32; 4]; 4];

const MAT4_IDENTITY: Mat4 = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

fn mat4_mul(a: &Mat4, b: &Mat4) -> Mat4 {
    let mut out = [[0.0f32; 4]; 4];
    for i in 0..4 {
        for j in 0..4 {
            out[i][j] = a[i][0] * b[0][j] + a[i][1] * b[1][j] + a[i][2] * b[2][j] + a[i][3] * b[3][j];
        }
    }
    out
}

fn transform_point(m: &Mat4, p: [f32; 3]) -> [f32; 3] {
    let x = m[0][0] * p[0] + m[0][1] * p[1] + m[0][2] * p[2] + m[0][3];
    let y = m[1][0] * p[0] + m[1][1] * p[1] + m[1][2] * p[2] + m[1][3];
    let z = m[2][0] * p[0] + m[2][1] * p[1] + m[2][2] * p[2] + m[2][3];
    [x, y, z]
}

fn node_local_transform(node: &gltf::Node) -> Mat4 {
    let t = node.transform();
    let cols = t.matrix();
    [
        [cols[0][0], cols[1][0], cols[2][0], cols[3][0]],
        [cols[0][1], cols[1][1], cols[2][1], cols[3][1]],
        [cols[0][2], cols[1][2], cols[2][2], cols[3][2]],
        [cols[0][3], cols[1][3], cols[2][3], cols[3][3]],
    ]
}

fn collect_meshes_recursive(
    node: &gltf::Node,
    parent_transform: &Mat4,
    buffers: &[gltf::buffer::Data],
    all_positions: &mut Vec<[f32; 3]>,
    all_indices: &mut Vec<u32>,
) {
    let local = node_local_transform(node);
    let global = mat4_mul(parent_transform, &local);

    if let Some(mesh) = node.mesh() {
        for (prim_idx, primitive) in mesh.primitives().enumerate() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
            let base_vertex = all_positions.len() as u32;

            if let Some(positions) = reader.read_positions() {
                let positions: Vec<[f32; 3]> = positions.collect();
                let count = positions.len();
                for p in &positions {
                    all_positions.push(transform_point(&global, *p));
                }
                println!(
                    "  node {:?} mesh {:?} prim[{}]: {} verts",
                    node.name().unwrap_or("?"),
                    mesh.name().unwrap_or("?"),
                    prim_idx,
                    count
                );
            }

            if let Some(indices) = reader.read_indices() {
                let indices: Vec<u32> = indices.into_u32().map(|i| i + base_vertex).collect();
                println!("    {} tris", indices.len() / 3);
                all_indices.extend(indices);
            }
        }
    }

    for child in node.children() {
        collect_meshes_recursive(&child, &global, buffers, all_positions, all_indices);
    }
}

fn spherical_uv(pos: [f32; 3], center: [f32; 3], scale: f32) -> [f32; 2] {
    let dx = pos[0] - center[0];
    let dy = pos[1] - center[1];
    let dz = pos[2] - center[2];
    let len = (dx * dx + dy * dy + dz * dz).sqrt().max(1e-8);
    let nx = dx / len;
    let ny = dy / len;
    let nz = dz / len;

    let mut u = 0.5 + f32::atan2(nz, nx) / (2.0 * PI);
    let v = 0.5 + f32::asin(ny.clamp(-1.0, 1.0)) / PI;

    if u < 0.0 {
        u += 1.0;
    }

    [u * scale, v * scale]
}

fn compute_centroid(positions: &[[f32; 3]]) -> [f32; 3] {
    let n = positions.len() as f32;
    let mut cx = 0.0f32;
    let mut cy = 0.0f32;
    let mut cz = 0.0f32;
    for p in positions {
        cx += p[0];
        cy += p[1];
        cz += p[2];
    }
    [cx / n, cy / n, cz / n]
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: mesh_decimate <input.glb> [target_tris] [texture.png]");
        eprintln!(
            "  target_tris: number of output triangles (default: {})",
            DEFAULT_TARGET_TRIS
        );
        eprintln!("  texture.png: optional texture to apply (copied to output dir)");
        eprintln!("  output goes to {}/", MESHDUMP_DIR);
        std::process::exit(1);
    }

    let input_path = &args[1];
    let target_tris: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(DEFAULT_TARGET_TRIS);
    let texture_path: Option<&str> = args.get(3).map(|s| s.as_str());

    println!("=== mesh_decimate ===");
    println!("Input:   {}", input_path);
    println!("Target:  {} tris", target_tris);
    if let Some(tp) = texture_path {
        println!("Texture: {}", tp);
    }

    // Load GLB
    println!("\n--- Loading ---");
    let (doc, buffers, _images) = gltf::import(input_path).expect("Failed to load GLB");

    let mut all_positions: Vec<[f32; 3]> = Vec::new();
    let mut all_indices: Vec<u32> = Vec::new();

    for scene in doc.scenes() {
        for node in scene.nodes() {
            collect_meshes_recursive(&node, &MAT4_IDENTITY, &buffers, &mut all_positions, &mut all_indices);
        }
    }

    let original_tris = all_indices.len() / 3;
    let original_verts = all_positions.len();
    println!("\nMerged: {} verts, {} tris", original_verts, original_tris);

    if original_tris == 0 {
        eprintln!("No triangles found!");
        std::process::exit(1);
    }

    let target_index_count = (target_tris * 3).min(all_indices.len());
    let target_index_count = (target_index_count / 3) * 3;
    let target_index_count = target_index_count.max(3);

    let ratio = target_tris as f32 / original_tris as f32;

    let vertex_data: &[u8] = unsafe {
        std::slice::from_raw_parts(
            all_positions.as_ptr() as *const u8,
            all_positions.len() * std::mem::size_of::<[f32; 3]>(),
        )
    };

    let adapter = meshopt::VertexDataAdapter::new(vertex_data, std::mem::size_of::<[f32; 3]>(), 0)
        .expect("Failed to create vertex adapter");

    println!("\n--- Simplifying ---");
    println!(
        "Pass 1: QUALITY (ratio {:.4}, {} -> {} tris)",
        ratio, original_tris, target_tris
    );

    let quality_result = meshopt::simplify(
        &all_indices,
        &adapter,
        target_index_count,
        f32::MAX,
        meshopt::SimplifyOptions::empty(),
        None,
    );

    let quality_tris = quality_result.len() / 3;
    println!("  quality got: {} tris", quality_tris);

    let simplified = if quality_tris > target_tris * 2 {
        println!(
            "Pass 2: SLOPPY (quality stalled at {}, target {})",
            quality_tris, target_tris
        );

        let (compact_pos_q, compact_idx_q) = compact_mesh(&all_positions, &quality_result);

        let vertex_data_q: &[u8] = unsafe {
            std::slice::from_raw_parts(
                compact_pos_q.as_ptr() as *const u8,
                compact_pos_q.len() * std::mem::size_of::<[f32; 3]>(),
            )
        };

        let adapter_q = meshopt::VertexDataAdapter::new(vertex_data_q, std::mem::size_of::<[f32; 3]>(), 0)
            .expect("Failed to create vertex adapter");

        let sloppy_result = meshopt::simplify_sloppy(&compact_idx_q, &adapter_q, target_index_count, f32::MAX, None);

        let sloppy_tris = sloppy_result.len() / 3;
        println!("  sloppy got: {} tris", sloppy_tris);

        let (final_pos, final_idx) = compact_mesh(&compact_pos_q, &sloppy_result);
        write_output(
            input_path,
            &final_pos,
            &final_idx,
            original_tris,
            original_verts,
            texture_path,
        );
        return;
    } else {
        quality_result
    };

    let (final_pos, final_idx) = compact_mesh(&all_positions, &simplified);
    write_output(
        input_path,
        &final_pos,
        &final_idx,
        original_tris,
        original_verts,
        texture_path,
    );
}

fn compact_mesh(positions: &[[f32; 3]], indices: &[u32]) -> (Vec<[f32; 3]>, Vec<u32>) {
    let mut used = vec![false; positions.len()];
    for &idx in indices {
        used[idx as usize] = true;
    }

    let mut remap = vec![0u32; positions.len()];
    let mut compact_positions: Vec<[f32; 3]> = Vec::new();
    for (old_idx, &is_used) in used.iter().enumerate() {
        if is_used {
            remap[old_idx] = compact_positions.len() as u32;
            compact_positions.push(positions[old_idx]);
        }
    }

    let compact_indices: Vec<u32> = indices.iter().map(|&i| remap[i as usize]).collect();
    (compact_positions, compact_indices)
}

fn write_output(
    input_path: &str,
    positions: &[[f32; 3]],
    indices: &[u32],
    original_tris: usize,
    original_verts: usize,
    texture_path: Option<&str>,
) {
    let result_tris = indices.len() / 3;

    println!("\nFinal: {} verts, {} tris", positions.len(), result_tris);

    let input_stem = Path::new(input_path).file_stem().unwrap().to_string_lossy();
    let output_filename = format!("{}_dec{}.obj", input_stem, result_tris);
    let output_path = Path::new(MESHDUMP_DIR).join(&output_filename);

    // Generate spherical UVs
    let center = compute_centroid(positions);
    let uvs: Vec<[f32; 2]> = positions
        .iter()
        .map(|p| spherical_uv(*p, center, UV_TILE_SCALE))
        .collect();

    // Handle texture: copy to meshdump, write MTL
    let mtl_name = format!("{}_dec{}.mtl", input_stem, result_tris);
    let tex_filename = if let Some(tp) = texture_path {
        let tex_src = Path::new(tp);
        let tex_name = tex_src.file_name().unwrap().to_string_lossy().to_string();
        let tex_dst = Path::new(MESHDUMP_DIR).join(&tex_name);
        if !tex_dst.exists() {
            fs::copy(tex_src, &tex_dst).expect("Failed to copy texture");
            println!("Copied texture: {}", tex_dst.display());
        }

        // Write MTL
        let mtl_path = Path::new(MESHDUMP_DIR).join(&mtl_name);
        let mut mtl = File::create(&mtl_path).expect("Failed to create MTL");
        writeln!(mtl, "newmtl jiwen").unwrap();
        writeln!(mtl, "Ka 1.0 1.0 1.0").unwrap();
        writeln!(mtl, "Kd 1.0 1.0 1.0").unwrap();
        writeln!(mtl, "map_Kd {}", tex_name).unwrap();
        println!("Wrote MTL: {}", mtl_path.display());

        Some(tex_name)
    } else {
        None
    };

    println!("\n--- Writing OBJ ---");

    let mut file = File::create(&output_path).expect("Failed to create output file");

    if tex_filename.is_some() {
        writeln!(file, "mtllib {}", mtl_name).unwrap();
    }

    writeln!(file, "o {}", input_stem).unwrap();

    for v in positions {
        writeln!(file, "v {:.6} {:.6} {:.6}", v[0], v[1], v[2]).unwrap();
    }

    for uv in &uvs {
        writeln!(file, "vt {:.6} {:.6}", uv[0], uv[1]).unwrap();
    }

    if tex_filename.is_some() {
        writeln!(file, "usemtl jiwen").unwrap();
    }
    writeln!(file, "s off").unwrap();

    for tri in indices.chunks(3) {
        writeln!(
            file,
            "f {}/{} {}/{} {}/{}",
            tri[0] + 1,
            tri[0] + 1,
            tri[1] + 1,
            tri[1] + 1,
            tri[2] + 1,
            tri[2] + 1
        )
        .unwrap();
    }

    drop(file);

    let file_size = fs::metadata(&output_path).map(|m| m.len()).unwrap_or(0);

    println!("Wrote: {}", output_path.display());
    println!("File:  {} bytes", file_size);
    println!(
        "\nReduction: {} -> {} tris ({:.1}% removed)",
        original_tris,
        result_tris,
        (1.0 - result_tris as f64 / original_tris as f64) * 100.0
    );
    println!("Vertices:  {} -> {}", original_verts, positions.len());
}
