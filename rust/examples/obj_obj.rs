use gltf::json::deserialize::from_str;
use gltf::json::serialize::to_string_pretty;
use gltf::json::validation::USize64;
use gltf::json::{accessor, buffer, mesh, validation, Accessor, Index, Root};
use gltf::Gltf;
use std::f64::consts::PI;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::process::Command;

const OBJ_OUT: &str = "../assets/meshes/sphere_PREBAKE.obj";
const GLTF_OUT: &str = "../assets/meshes/sphere_PREBAKE.gltf";
const GLB_OUT: &str = "../assets/meshes/sphere_PREBAKE.glb";

enum TexcoordMapping {
    PlanarProjectionXY,
    SphericalEquirectangularAnalytic,
    SphericalEquirectangularUnwrapped,
}

fn main() {
    println!("=== Step 1: Generating OBJ ===");
    let mut file = File::create(OBJ_OUT).unwrap();
    let radius = 0.5;
    let stacks = 8;
    let slices = 8;
    let mode = TexcoordMapping::SphericalEquirectangularUnwrapped;

    writeln!(file, "o sphere").unwrap();
    let mut verts = Vec::new();
    verts.push([0.0, radius, 0.0]);

    for i in 1..stacks {
        let t = PI * (i as f64) / (stacks as f64);
        let y = radius * t.cos();
        let r = radius * t.sin();
        for j in 0..slices {
            let p = 2.0 * PI * (j as f64) / (slices as f64);
            let x = r * p.cos();
            let z = r * p.sin();
            verts.push([x, y, z]);
        }
    }
    verts.push([0.0, -radius, 0.0]);
    for v in &verts {
        writeln!(file, "v {:.6} {:.6} {:.6}", v[0], v[1], v[2]).unwrap();
    }

    emit_sphere_normals(&mut file, &verts);
    emit_sphere_texcoords(&mut file, &verts, stacks, slices, &mode);
    writeln!(file, "s off").unwrap();
    emit_sphere_indexed_triangles(&mut file, stacks, slices, &mode);
    drop(file);
    println!("OBJ generated with {} vertices", verts.len());

    println!("\n=== Step 2: Converting OBJ to GLTF ===");
    run_gltfpack();

    println!("\n=== Step 3: Verifying GLTF after conversion ===");
    verify_gltf_attributes(GLTF_OUT, "after OBJ->GLTF conversion");

    println!("\n=== Step 4: Adding vertex colors to GLTF ===");
    fill_vertex_colors_gltf();

    println!("\n=== Step 5: Verifying GLTF after adding colors ===");
    verify_gltf_attributes(GLTF_OUT, "after adding vertex colors");

    println!("\n=== Step 6: Converting GLTF to GLB ===");
    convert_to_glb();

    println!("\n=== Step 7: Verifying final GLB ===");
    verify_glb_attributes();

    println!("\n=== DONE ===");
}

fn verify_gltf_attributes(path: &str, stage: &str) {
    println!("Verifying GLTF attributes ({}):", stage);
    let gltf = Gltf::from_slice(&fs::read(path).unwrap()).unwrap();
    let (_, buffers, _) = gltf::import(path).unwrap();

    for (i, mesh) in gltf.meshes().enumerate() {
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

            // Check texcoords
            let has_texcoords = primitive.get(&mesh::Semantic::TexCoords(0)).is_some();
            println!("  Mesh {}: Has TEXCOORD_0 attribute: {}", i, has_texcoords);

            if let Some(texcoords_iter) = reader.read_tex_coords(0) {
                let texcoords: Vec<[f32; 2]> = texcoords_iter.into_f32().collect();
                println!(
                    "    Found {} texcoords (first: {:?})",
                    texcoords.len(),
                    texcoords.get(0)
                );
            } else if has_texcoords {
                println!("    TEXCOORD_0 attribute exists but couldn't read texcoords!");
            }

            // Check colors
            let has_colors = primitive.get(&mesh::Semantic::Colors(0)).is_some();
            println!("  Mesh {}: Has COLOR_0 attribute: {}", i, has_colors);

            if let Some(colors_iter) = reader.read_colors(0) {
                let colors: Vec<[u8; 4]> = colors_iter.into_rgba_u8().collect();
                println!("    Found {} colors (first: {:?})", colors.len(), colors.get(0));
            } else if has_colors {
                println!("    COLOR_0 attribute exists but couldn't read colors!");
            }
        }
    }
}

fn verify_glb_attributes() {
    println!("Verifying final GLB attributes:");
    let final_glb = Gltf::from_slice(&fs::read(GLB_OUT).unwrap()).unwrap();

    for (i, mesh) in final_glb.meshes().enumerate() {
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|_| final_glb.blob.as_ref().map(|b| &b[..]));

            // Check texcoords
            let has_texcoords = primitive.get(&mesh::Semantic::TexCoords(0)).is_some();
            println!("  Mesh {}: Has TEXCOORD_0 attribute: {}", i, has_texcoords);

            if let Some(texcoords_iter) = reader.read_tex_coords(0) {
                let texcoords: Vec<[f32; 2]> = texcoords_iter.into_f32().collect();
                println!(
                    "    Found {} texcoords in GLB! (first: {:?})",
                    texcoords.len(),
                    texcoords.get(0)
                );
            } else if has_texcoords {
                println!("    TEXCOORD_0 attribute exists but couldn't read texcoords!");
            }

            // Check colors
            let has_colors = primitive.get(&mesh::Semantic::Colors(0)).is_some();
            println!("  Mesh {}: Has COLOR_0 attribute: {}", i, has_colors);

            if let Some(colors_iter) = reader.read_colors(0) {
                let colors: Vec<[u8; 4]> = colors_iter.into_rgba_u8().collect();
                println!(
                    "    Found {} vertex colors in GLB! (first: {:?})",
                    colors.len(),
                    colors.get(0)
                );
            } else if has_colors {
                println!("    COLOR_0 attribute exists but couldn't read colors!");
            }
        }
    }
}

fn emit_sphere_normals(file: &mut File, verts: &[[f64; 3]]) {
    for v in verts {
        let x = v[0];
        let y = v[1];
        let z = v[2];
        let l = (x * x + y * y + z * z).sqrt();
        let nx = if l != 0.0 { x / l } else { 0.0 };
        let ny = if l != 0.0 { y / l } else { 0.0 };
        let nz = if l != 0.0 { z / l } else { 0.0 };
        writeln!(file, "vn {:.6} {:.6} {:.6}", nx, ny, nz).unwrap();
    }
}

fn emit_sphere_texcoords(file: &mut File, verts: &[[f64; 3]], stacks: usize, slices: usize, mode: &TexcoordMapping) {
    match mode {
        TexcoordMapping::PlanarProjectionXY => {
            for v in verts {
                let (s, t) = planar_projection_xy_st(v[0], v[1], v[2]);
                writeln!(file, "vt {:.6} {:.6}", s, t).unwrap();
            }
        },
        TexcoordMapping::SphericalEquirectangularAnalytic => {
            for v in verts {
                let (s, t) = spherical_equirectangular_analytic_st(v[0], v[1], v[2]);
                writeln!(file, "vt {:.6} {:.6}", s, t).unwrap();
            }
        },
        TexcoordMapping::SphericalEquirectangularUnwrapped => {
            emit_sphere_equirectangular_unwrapped_texcoords(file, stacks, slices);
        },
    }
}

fn emit_sphere_indexed_triangles(file: &mut File, stacks: usize, slices: usize, mode: &TexcoordMapping) {
    match mode {
        TexcoordMapping::PlanarProjectionXY | TexcoordMapping::SphericalEquirectangularAnalytic => {
            emit_indexed_triangles_shared_texcoords(file, stacks, slices);
        },
        TexcoordMapping::SphericalEquirectangularUnwrapped => {
            emit_indexed_triangles_equirectangular_unwrapped(file, stacks, slices);
        },
    }
}

fn emit_indexed_triangles_shared_texcoords(file: &mut File, stacks: usize, slices: usize) {
    let top = 1;
    let rings = stacks - 1;
    let bottom = 2 + (rings * slices);
    for j in 0..slices {
        let k = (j + 1) % slices;
        let a = 2 + j;
        let b = 2 + k;
        writeln!(file, "f {0}/{0} {1}/{1} {2}/{2}", top, b, a).unwrap();
    }
    for s in 0..(rings - 1) {
        for j in 0..slices {
            let k = (j + 1) % slices;
            let u0 = 2 + s * slices + j;
            let u1 = 2 + s * slices + k;
            let l0 = 2 + (s + 1) * slices + j;
            let l1 = 2 + (s + 1) * slices + k;
            writeln!(file, "f {0}/{0} {1}/{1} {2}/{2}", u0, u1, l0).unwrap();
            writeln!(file, "f {0}/{0} {1}/{1} {2}/{2}", u1, l1, l0).unwrap();
        }
    }
    let base = 2 + (rings - 1) * slices;
    for j in 0..slices {
        let k = (j + 1) % slices;
        let a = base + j;
        let b = base + k;
        writeln!(file, "f {0}/{0} {1}/{1} {2}/{2}", bottom, a, b).unwrap();
    }
}

fn emit_sphere_equirectangular_unwrapped_texcoords(file: &mut File, stacks: usize, slices: usize) {
    for j in 0..slices {
        let s = (j as f64 + 0.5) / (slices as f64);
        writeln!(file, "vt {:.6} {:.6}", s, 1.0).unwrap();
    }
    for i in 1..stacks {
        let t = 1.0 - (i as f64) / (stacks as f64);
        for j in 0..=slices {
            let s = (j as f64) / (slices as f64);
            writeln!(file, "vt {:.6} {:.6}", s, t).unwrap();
        }
    }
    for j in 0..slices {
        let s = (j as f64 + 0.5) / (slices as f64);
        writeln!(file, "vt {:.6} {:.6}", s, 0.0).unwrap();
    }
}

fn emit_indexed_triangles_equirectangular_unwrapped(file: &mut File, stacks: usize, slices: usize) {
    let rings = stacks - 1;
    for j in 0..slices {
        let v_a = 2 + j;
        let v_b = 2 + ((j + 1) % slices);
        let st_top = 1 + j;
        let st_a = slices + 1 + j;
        let st_b = slices + 1 + j + 1;
        writeln!(file, "f {}/{} {}/{} {}/{}", 1, st_top, v_b, st_b, v_a, st_a).unwrap();
    }
    for s in 0..(rings - 1) {
        let ring_st_base = slices + 1 + s * (slices + 1);
        let next_ring_st_base = slices + 1 + (s + 1) * (slices + 1);
        for j in 0..slices {
            let v_u0 = 2 + s * slices + j;
            let v_u1 = 2 + s * slices + ((j + 1) % slices);
            let v_l0 = 2 + (s + 1) * slices + j;
            let v_l1 = 2 + (s + 1) * slices + ((j + 1) % slices);
            let st_u0 = ring_st_base + j;
            let st_u1 = ring_st_base + j + 1;
            let st_l0 = next_ring_st_base + j;
            let st_l1 = next_ring_st_base + j + 1;
            writeln!(file, "f {}/{} {}/{} {}/{}", v_u0, st_u0, v_u1, st_u1, v_l0, st_l0).unwrap();
            writeln!(file, "f {}/{} {}/{} {}/{}", v_u1, st_u1, v_l1, st_l1, v_l0, st_l0).unwrap();
        }
    }
    let base_ring_st = 1 + slices + (rings - 1) * (slices + 1);
    let bottom_pole_st_base = 1 + slices + rings * (slices + 1);
    for j in 0..slices {
        let v_a = 2 + (rings - 1) * slices + j;
        let v_b = 2 + (rings - 1) * slices + ((j + 1) % slices);
        let v_bottom = 2 + rings * slices;
        let st_a = base_ring_st + j;
        let st_b = base_ring_st + j + 1;
        let st_bottom = bottom_pole_st_base + j;
        writeln!(file, "f {}/{} {}/{} {}/{}", v_bottom, st_bottom, v_a, st_a, v_b, st_b).unwrap();
    }
}

fn spherical_equirectangular_analytic_st(x: f64, y: f64, z: f64) -> (f64, f64) {
    let r = (x * x + y * y + z * z).sqrt();
    if r == 0.0 {
        return (0.0, 0.0);
    }
    let mut s = f64::atan2(z, x) / (2.0 * PI);
    if s < 0.0 {
        s += 1.0;
    }
    let ny = (y / r).clamp(-1.0, 1.0);
    let t = 1.0 - ny.acos() / PI;
    (s, t)
}

fn planar_projection_xy_st(x: f64, y: f64, _z: f64) -> (f64, f64) {
    let s = (x + 0.5) / 1.0;
    let t = (y + 0.5) / 1.0;
    (s, t)
}

fn run_gltfpack() {
    let output = Command::new("gltfpack")
        .arg("-i")
        .arg(OBJ_OUT)
        .arg("-o")
        .arg(GLTF_OUT)
        .arg("-kv")
        .arg("-noq")
        .output()
        .expect("Failed to run gltfpack");

    if output.status.success() {
        println!("gltfpack OBJ->GLTF: SUCCESS");
    } else {
        eprintln!("gltfpack OBJ->GLTF: FAILED");
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
    }
}

fn convert_to_glb() {
    let output = Command::new("gltfpack")
        .arg("-i")
        .arg(GLTF_OUT)
        .arg("-o")
        .arg(GLB_OUT)
        .arg("-kv")
        .arg("-noq")
        .output()
        .expect("Failed to convert to GLB");

    if output.status.success() {
        println!("gltfpack GLTF->GLB: SUCCESS");
    } else {
        eprintln!("gltfpack GLTF->GLB: FAILED");
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
    }
}

fn fill_vertex_colors_gltf() {
    println!("Reading GLTF...");
    let (gltf, buffers, _) = gltf::import(GLTF_OUT).unwrap();

    println!("Generating vertex colors...");
    let mut all_colors = Vec::new();
    for mesh in gltf.meshes() {
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
            if let Some(positions) = reader.read_positions() {
                let positions: Vec<[f32; 3]> = positions.collect();
                let mut min = [f32::INFINITY; 3];
                let mut max = [f32::NEG_INFINITY; 3];
                for pos in &positions {
                    for i in 0..3 {
                        min[i] = min[i].min(pos[i]);
                        max[i] = max[i].max(pos[i]);
                    }
                }
                let mut colors = Vec::with_capacity(positions.len());
                for pos in &positions {
                    let nx = (pos[0] - 0.5 * (min[0] + max[0])) / (0.5 * (max[0] - min[0]));
                    let ny = (pos[1] - 0.5 * (min[1] + max[1])) / (0.5 * (max[1] - min[1]));
                    let nz = (pos[2] - 0.5 * (min[2] + max[2])) / (0.5 * (max[2] - min[2]));
                    let len = (nx * nx + ny * ny + nz * nz).sqrt();
                    colors.push([
                        (127.5 * (nx / len + 1.0)).round() as u8,
                        (127.5 * (ny / len + 1.0)).round() as u8,
                        (127.5 * (nz / len + 1.0)).round() as u8,
                        255,
                    ]);
                }
                println!(
                    "Generated {} vertex colors (first color: {:?})",
                    colors.len(),
                    colors[0]
                );
                all_colors.push(colors);
            }
        }
    }

    println!("Modifying GLTF to add color attributes...");
    let gltf_path = std::path::Path::new(GLTF_OUT);
    let mut root: Root = from_str(&fs::read_to_string(gltf_path).unwrap()).unwrap();
    let bin_path = gltf_path.with_extension("bin");
    let mut bin_data = fs::read(&bin_path).unwrap_or_default();
    let original_bin_size = bin_data.len();

    for (mesh_idx, colors) in all_colors.iter().enumerate() {
        let color_bytes: Vec<u8> = colors.iter().flat_map(|c| c.iter().copied()).collect();
        let color_offset = bin_data.len();
        bin_data.extend_from_slice(&color_bytes);
        println!(
            "  Mesh {}: Added {} color bytes at offset {}",
            mesh_idx,
            color_bytes.len(),
            color_offset
        );

        let buffer_view_idx = root.buffer_views.len();
        root.buffer_views.push(buffer::View {
            buffer: Index::new(0),
            byte_length: USize64::from(color_bytes.len()),
            byte_offset: Some(USize64::from(color_offset)),
            byte_stride: Some(buffer::Stride(4)),
            extensions: Default::default(),
            extras: Default::default(),
            name: None,
            target: Some(validation::Checked::Valid(buffer::Target::ArrayBuffer)),
        });

        let accessor_idx = root.accessors.len();
        root.accessors.push(Accessor {
            buffer_view: Some(Index::new(buffer_view_idx as u32)),
            byte_offset: Some(USize64::from(0u64)),
            count: USize64::from(colors.len()),
            component_type: validation::Checked::Valid(accessor::GenericComponentType(accessor::ComponentType::U8)),
            extensions: Default::default(),
            extras: Default::default(),
            type_: validation::Checked::Valid(accessor::Type::Vec4),
            min: None,
            max: None,
            name: None,
            normalized: true,
            sparse: None,
        });

        if let Some(mesh) = root.meshes.get_mut(mesh_idx) {
            if let Some(primitive) = mesh.primitives.get_mut(0) {
                primitive.attributes.insert(
                    validation::Checked::Valid(mesh::Semantic::Colors(0)),
                    Index::new(accessor_idx as u32),
                );
                println!(
                    "  Mesh {}: Added COLOR_0 attribute (accessor {})",
                    mesh_idx, accessor_idx
                );
            }
        }
    }

    if let Some(buffer) = root.buffers.get_mut(0) {
        buffer.byte_length = USize64::from(bin_data.len());
    }

    println!("Writing modified GLTF and binary data...");
    println!("  Binary size: {} -> {} bytes", original_bin_size, bin_data.len());
    fs::write(&bin_path, &bin_data).unwrap();
    fs::write(gltf_path, to_string_pretty(&root).unwrap()).unwrap();
}
