use gltf::json::deserialize::from_str;
use gltf::json::serialize::to_string_pretty;
use gltf::json::validation::USize64;
use gltf::json::*;
use gltf::Gltf;
use std::f32::consts::PI;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::process::Command;

const SPHERE_OBJ_OUT: &str = "../assets/meshes/sphere_PREBAKE.obj";
const SPHERE_GLTF_OUT: &str = "../assets/meshes/sphere_PREBAKE.gltf";
const SPHERE_GLB_OUT: &str = "../assets/meshes/sphere_PREBAKE.glb";

const FUSUMA_OBJ_OUT: &str = "../assets/meshes/fusuma_PREBAKE.obj";
const FUSUMA_GLTF_OUT: &str = "../assets/meshes/fusuma_PREBAKE.gltf";
const FUSUMA_GLB_OUT: &str = "../assets/meshes/fusuma_PREBAKE.glb";

const KIWAKU_OBJ_OUT: &str = "../assets/meshes/kiwaku_PREBAKE.obj";
const KIWAKU_GLTF_OUT: &str = "../assets/meshes/kiwaku_PREBAKE.gltf";
const KIWAKU_GLB_OUT: &str = "../assets/meshes/window_PREBAKE.glb";

const KOSHIDAKA_OBJ_OUT: &str = "../assets/meshes/koshidaka_PREBAKE.obj";
const KOSHIDAKA_GLTF_OUT: &str = "../assets/meshes/koshidaka_PREBAKE.gltf";
const KOSHIDAKA_GLB_OUT: &str = "../assets/meshes/window_PREBAKE.glb";

const ARROW_OBJ_OUT: &str = "../assets/meshes/arrow_PREBAKE.obj";
const ARROW_GLTF_OUT: &str = "../assets/meshes/arrow_PREBAKE.gltf";
const ARROW_GLB_OUT: &str = "../assets/meshes/arrow_PREBAKE.glb";

enum TexcoordMapping {
    PlanarProjectionXY,
    SphericalEquirectangularAnalytic,
    SphericalEquirectangularUnwrapped,
}

struct WindowSpec {
    frame_w_mm: f32,
    frame_h_mm: f32,
    frame_d_mm: f32,
    frame_border_mm: f32,
    mullion_mm: f32,
    glass_recess_mm: f32,
    chamfer_mm: f32,
    mullion_inset_mm: f32,
}

#[derive(Clone, Copy)]
struct MullionConfig {
    vertical_count: u32,
    horizontal_count: u32,
}

const KOSHIDAKA_SPEC: WindowSpec = WindowSpec {
    frame_w_mm: 850.0,
    frame_h_mm: 1170.0,
    frame_d_mm: 75.0,
    frame_border_mm: 50.0,
    mullion_mm: 26.0,
    glass_recess_mm: 8.0,
    chamfer_mm: 7.0,
    mullion_inset_mm: 6.0,
};

const KIWAKU_SPEC: WindowSpec = WindowSpec {
    frame_w_mm: 800.0,
    frame_h_mm: 800.0,
    frame_d_mm: 68.0,
    frame_border_mm: 45.0,
    mullion_mm: 20.0,
    glass_recess_mm: 8.0,
    chamfer_mm: 7.0,
    mullion_inset_mm: 6.0,
};

fn main() {
    println!("=== Step 1: Generating OBJ ===");

    let obj_out = ARROW_OBJ_OUT;
    let gltf_out = ARROW_GLTF_OUT;
    let glb_out = ARROW_GLB_OUT;
    let obj_name = "arrow";
    let spec = &ARROW_SPEC_TEST;

    let mut file = File::create(obj_out).unwrap();

    //write_fusuma_with_handle_obj(&mut file); //TODO: need to get the CLAM vs PIE disk back but whatever..

    write_arrow_obj(&mut file, spec, obj_name);

    drop(file);

    println!("\n=== Step 2: Converting OBJ to GLTF ===");
    run_gltfpack(obj_out, gltf_out);

    println!("\n=== Step 3: Verifying GLTF after conversion ===");
    verify_gltf_attributes(gltf_out, "after OBJ->GLTF conversion");

    println!("\n=== Step 4: Adding vertex colors to GLTF ===");
    fill_vertex_colors_gltf(gltf_out);

    println!("\n=== Step 5: Verifying GLTF after adding colors ===");
    verify_gltf_attributes(gltf_out, "after adding vertex colors");

    println!("\n=== Step 6: Converting GLTF to GLB ===");
    convert_to_glb(gltf_out, glb_out);

    println!("\n=== Step 7: Verifying final GLB ===");
    verify_glb_attributes(glb_out);

    println!("\n=== Step 8: Cleaning up intermediate files ===");

    if let Err(e) = fs::remove_file(gltf_out) {
        eprintln!("Failed to delete GLTF: {}", e);
    } else {
        println!("Deleted: {}", gltf_out);
    }

    let bin_out = format!("{}.bin", gltf_out.trim_end_matches(".gltf"));

    if let Err(e) = fs::remove_file(&bin_out) {
        eprintln!("Failed to delete BIN: {}", e);
    } else {
        println!("Deleted: {}", bin_out);
    }
}

#[derive(Clone, Copy)]
struct ArrowSpec {
    width: f32,
    length: f32,
    tail_scale: f32,
    thickness: f32,
    chamfer_xy: f32,
    chamfer_z: f32,
}

const ARROW_SPEC_TEST: ArrowSpec = ArrowSpec {
    width: 0.25,
    length: 0.33,
    tail_scale: 1.0,
    thickness: 0.1,
    chamfer_xy: 0.033,
    chamfer_z: 0.033,
};

const ARROW_OUTLINE_BASE: [[f32; 2]; 16] = [
    [-0.004027, 0.125000],
    [-0.125000, 0.004027],
    [-0.125000, -0.028353],
    [-0.100674, -0.052515],
    [-0.066732, -0.052515],
    [-0.044379, -0.028600],
    [-0.044297, -0.088675],
    [-0.008054, -0.125000],
    [0.008054, -0.125000],
    [0.044297, -0.088675],
    [0.044379, -0.028600],
    [0.066732, -0.052515],
    [0.100674, -0.052515],
    [0.125000, -0.028353],
    [0.125000, 0.004027],
    [0.004027, 0.125000],
];

const ARROW_TAIL_PIVOT_Y: f32 = -0.052515;

fn build_arrow_outline_2d(spec: &ArrowSpec) -> Vec<[f32; 2]> {
    let tail_scale = spec.tail_scale.max(0.01);

    let mut pts: Vec<[f32; 2]> = ARROW_OUTLINE_BASE.iter().copied().collect();

    if (tail_scale - 1.0).abs() > f32::EPSILON {
        for p in pts.iter_mut() {
            if p[1] <= ARROW_TAIL_PIVOT_Y {
                p[1] = ARROW_TAIL_PIVOT_Y + (p[1] - ARROW_TAIL_PIVOT_Y) * tail_scale;
            }
        }
    }

    let mut min_x = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_y = f32::NEG_INFINITY;

    for p in &pts {
        min_x = min_x.min(p[0]);
        max_x = max_x.max(p[0]);
        min_y = min_y.min(p[1]);
        max_y = max_y.max(p[1]);
    }

    let width_base = max_x - min_x;
    let length_base = max_y - min_y;

    let cx = 0.5 * (min_x + max_x);
    let cy = 0.5 * (min_y + max_y);

    let sx = if width_base > 0.0 { spec.width / width_base } else { 1.0 };
    let sy = if length_base > 0.0 {
        spec.length / length_base
    } else {
        1.0
    };

    for p in pts.iter_mut() {
        p[0] = (p[0] - cx) * sx;
        p[1] = (p[1] - cy) * sy;
    }

    pts
}

fn write_arrow_obj(file: &mut File, spec: &ArrowSpec, object_name: &str) {
    let outline_2d = build_arrow_outline_2d(spec);
    let tris_2d = triangulate_polygon_ccw(&outline_2d);
    let n = outline_2d.len();

    let mut verts: Vec<[f32; 3]> = Vec::new();
    let mut faces: Vec<[usize; 3]> = Vec::new();

    let thickness = spec.thickness.max(0.0001);
    let half_t = 0.5 * thickness;

    let chamfer_z = spec.chamfer_z.min(half_t * 0.99).max(0.0001);

    let z0 = -half_t;
    let z1 = -half_t + chamfer_z;
    let z2 = half_t - chamfer_z;
    let z3 = half_t;

    let mut max_extent = 0.0f32;
    for p in &outline_2d {
        max_extent = max_extent.max(p[0].abs());
        max_extent = max_extent.max(p[1].abs());
    }

    let chamfer_xy = spec.chamfer_xy.min(max_extent * 0.99).max(0.0);
    let inner_scale = if max_extent > 0.0 {
        (max_extent - chamfer_xy) / max_extent
    } else {
        1.0
    };

    let r0_start = verts.len();
    for p in &outline_2d {
        verts.push([p[0] * inner_scale, p[1] * inner_scale, z0]);
    }
    let r0: Vec<usize> = (r0_start..r0_start + n).collect();

    let r1_start = verts.len();
    for p in &outline_2d {
        verts.push([p[0], p[1], z1]);
    }
    let r1: Vec<usize> = (r1_start..r1_start + n).collect();

    let r2_start = verts.len();
    for p in &outline_2d {
        verts.push([p[0], p[1], z2]);
    }
    let r2: Vec<usize> = (r2_start..r2_start + n).collect();

    let r3_start = verts.len();
    for p in &outline_2d {
        verts.push([p[0] * inner_scale, p[1] * inner_scale, z3]);
    }
    let r3: Vec<usize> = (r3_start..r3_start + n).collect();

    connect_rings(&mut faces, &r0, &r1);
    connect_rings(&mut faces, &r1, &r2);
    connect_rings(&mut faces, &r2, &r3);

    for tri in &tris_2d {
        let i0 = r0[tri[0]];
        let i1 = r0[tri[1]];
        let i2 = r0[tri[2]];
        faces.push([i0 + 1, i2 + 1, i1 + 1]);
    }

    for tri in &tris_2d {
        let i0 = r3[tri[0]];
        let i1 = r3[tri[1]];
        let i2 = r3[tri[2]];
        faces.push([i0 + 1, i1 + 1, i2 + 1]);
    }

    writeln!(file, "o {}", object_name).unwrap();
    writeln!(file, "s off").unwrap();

    for v in &verts {
        writeln!(file, "v {:.6} {:.6} {:.6}", v[0], v[1], v[2]).unwrap();
    }

    for f in &faces {
        writeln!(file, "f {} {} {}", f[0], f[1], f[2]).unwrap();
    }
}

fn triangulate_polygon_ccw(points: &[[f32; 2]]) -> Vec<[usize; 3]> {
    let n = points.len();
    assert!(n >= 3);

    let mut indices: Vec<usize> = (0..n).collect();
    let mut tris: Vec<[usize; 3]> = Vec::new();

    fn is_convex(points: &[[f32; 2]], i0: usize, i1: usize, i2: usize) -> bool {
        let (x0, y0) = (points[i0][0], points[i0][1]);
        let (x1, y1) = (points[i1][0], points[i1][1]);
        let (x2, y2) = (points[i2][0], points[i2][1]);
        let ux = x1 - x0;
        let uy = y1 - y0;
        let vx = x2 - x1;
        let vy = y2 - y1;
        (ux * vy - uy * vx) > 0.0
    }

    fn point_in_tri(points: &[[f32; 2]], px: f32, py: f32, i0: usize, i1: usize, i2: usize) -> bool {
        fn sign(ax: f32, ay: f32, bx: f32, by: f32, cx: f32, cy: f32) -> f32 {
            (ax - cx) * (by - cy) - (bx - cx) * (ay - cy)
        }

        let (x0, y0) = (points[i0][0], points[i0][1]);
        let (x1, y1) = (points[i1][0], points[i1][1]);
        let (x2, y2) = (points[i2][0], points[i2][1]);

        let b1 = sign(px, py, x0, y0, x1, y1) < 0.0;
        let b2 = sign(px, py, x1, y1, x2, y2) < 0.0;
        let b3 = sign(px, py, x2, y2, x0, y0) < 0.0;

        (b1 == b2) && (b2 == b3)
    }

    while indices.len() > 3 {
        let m = indices.len();
        let mut ear_found = false;

        for k in 0..m {
            let i_prev = indices[(k + m - 1) % m];
            let i_curr = indices[k];
            let i_next = indices[(k + 1) % m];

            if !is_convex(points, i_prev, i_curr, i_next) {
                continue;
            }

            let mut has_inside = false;
            for &j in &indices {
                if j == i_prev || j == i_curr || j == i_next {
                    continue;
                }
                let (px, py) = (points[j][0], points[j][1]);
                if point_in_tri(points, px, py, i_prev, i_curr, i_next) {
                    has_inside = true;
                    break;
                }
            }

            if has_inside {
                continue;
            }

            tris.push([i_prev, i_curr, i_next]);
            indices.remove(k);
            ear_found = true;
            break;
        }

        if !ear_found {
            panic!("triangulate_polygon_ccw: no ear found (degenerate polygon?)");
        }
    }

    tris.push([indices[0], indices[1], indices[2]]);

    tris
}

fn connect_rings(faces: &mut Vec<[usize; 3]>, ring_a: &[usize], ring_b: &[usize]) {
    assert_eq!(ring_a.len(), ring_b.len());
    let m = ring_a.len();
    for i in 0..m {
        let j = (i + 1) % m;
        let a0 = ring_a[i];
        let a1 = ring_a[j];
        let b0 = ring_b[i];
        let b1 = ring_b[j];

        faces.push([a0 + 1, a1 + 1, b1 + 1]);
        faces.push([a0 + 1, b1 + 1, b0 + 1]);
    }
}

fn write_window_obj(file: &mut File, spec: &WindowSpec, mullions: MullionConfig, object_name: &str) {
    let mm_to_unit = 1.0_f32 / 800.0_f32;

    let frame_w = spec.frame_w_mm * mm_to_unit;
    let frame_h = spec.frame_h_mm * mm_to_unit;
    let frame_d = spec.frame_d_mm * mm_to_unit;

    let border = spec.frame_border_mm * mm_to_unit;
    let mullion = spec.mullion_mm * mm_to_unit;
    let glass_recess = spec.glass_recess_mm * mm_to_unit;
    let chamfer = spec.chamfer_mm * mm_to_unit;
    let mullion_inset = spec.mullion_inset_mm * mm_to_unit;

    let hx = frame_w * 0.5_f32;
    let hy = frame_h * 0.5_f32;
    let hz = frame_d * 0.5_f32;

    let ix = hx - border;
    let iy = hy - border;

    let mx = mullion * 0.5_f32;
    let my = mullion * 0.5_f32;
    let mullion_hz = hz - glass_recess;

    let mut verts: Vec<[f32; 3]> = Vec::new();
    let mut faces: Vec<[usize; 3]> = Vec::new();

    add_box(&mut verts, &mut faces, [-hx, -hy, -hz], [-hx + border, hy, hz]);
    add_box(&mut verts, &mut faces, [hx - border, -hy, -hz], [hx, hy, hz]);
    add_box(&mut verts, &mut faces, [-hx, -hy, -hz], [hx, -hy + border, hz]);
    add_box(&mut verts, &mut faces, [-hx, hy - border, -hz], [hx, hy, hz]);

    if mullions.vertical_count > 0 {
        let n = mullions.vertical_count as f32;
        let span = 2.0 * ix;
        let step = span / (n + 1.0);

        for i in 0..mullions.vertical_count {
            let k = i as f32;

            let cx = -ix + step * (k + 1.0) + mullion_inset;

            let miny = -iy;
            let maxy = iy;

            add_mullion_vertical(&mut verts, &mut faces, cx, miny, maxy, mx, mullion_hz, chamfer);
        }
    }

    if mullions.horizontal_count > 0 {
        let n = mullions.horizontal_count as f32;
        let span = 2.0 * iy;
        let step = span / (n + 1.0);

        for i in 0..mullions.horizontal_count {
            let k = i as f32;

            let cy = -iy + step * (k + 1.0) + mullion_inset;

            let minx = -ix;
            let maxx = ix;

            add_mullion_horizontal(&mut verts, &mut faces, minx, maxx, cy, my, mullion_hz, chamfer);
        }
    }

    writeln!(file, "o {}", object_name).unwrap();
    writeln!(file, "s off").unwrap();

    for v in verts.iter() {
        writeln!(file, "v {:.6} {:.6} {:.6}", v[0], v[1], v[2]).unwrap();
    }

    for f in faces.iter() {
        writeln!(file, "f {} {} {}", f[0], f[1], f[2]).unwrap();
    }
}

fn add_mullion_vertical(
    verts: &mut Vec<[f32; 3]>,
    faces: &mut Vec<[usize; 3]>,
    cx: f32,
    miny: f32,
    maxy: f32,
    hx: f32,
    hz: f32,
    chamfer: f32,
) {
    let base = verts.len();

    let c = chamfer.min(hx * 0.99).min(hz * 0.99);

    let ring_xz = [
        [-hx + c, -hz],
        [hx - c, -hz],
        [hx, -hz + c],
        [hx, hz - c],
        [hx - c, hz],
        [-hx + c, hz],
        [-hx, hz - c],
        [-hx, -hz + c],
    ];

    for [x, z] in ring_xz {
        verts.push([cx + x, miny, z]);
    }

    for [x, z] in ring_xz {
        verts.push([cx + x, maxy, z]);
    }

    for i in 0..8 {
        let j = (i + 1) % 8;
        let b0 = base + i;
        let b1 = base + j;
        let t0 = base + 8 + i;
        let t1 = base + 8 + j;

        faces.push([b0 + 1, t1 + 1, b1 + 1]);
        faces.push([b0 + 1, t0 + 1, t1 + 1]);
    }

    let bottom_ring: Vec<usize> = (0..8).map(|k| base + k).collect();
    let top_ring: Vec<usize> = (0..8).map(|k| base + 8 + k).collect();

    cap_polygon(verts, faces, &bottom_ring, false);
    cap_polygon(verts, faces, &top_ring, true);
}
fn add_mullion_horizontal(
    verts: &mut Vec<[f32; 3]>,
    faces: &mut Vec<[usize; 3]>,
    minx: f32,
    maxx: f32,
    cy: f32,
    hy: f32,
    hz: f32,
    chamfer: f32,
) {
    let base = verts.len();
    let c = chamfer.min(hy * 0.99).min(hz * 0.99);

    let ring = [
        [cy - hy, -hz + c],
        [cy - hy + c, -hz],
        [cy + hy - c, -hz],
        [cy + hy, -hz + c],
        [cy + hy, hz - c],
        [cy + hy - c, hz],
        [cy - hy + c, hz],
        [cy - hy, hz - c],
    ];

    for [y, z] in ring {
        verts.push([minx, y, z]);
    }

    for [y, z] in ring {
        verts.push([maxx, y, z]);
    }

    for i in 0..8 {
        let j = (i + 1) % 8;

        let l0 = base + i;
        let l1 = base + j;
        let r0 = base + 8 + i;
        let r1 = base + 8 + j;

        faces.push([l0 + 1, l1 + 1, r1 + 1]);
        faces.push([l0 + 1, r1 + 1, r0 + 1]);
    }

    let left_ring: Vec<usize> = (0..8).map(|k| base + k).collect();
    let right_ring: Vec<usize> = (0..8).map(|k| base + 8 + k).collect();

    cap_polygon(verts, faces, &left_ring, true);

    cap_polygon(verts, faces, &right_ring, false);
}

fn cap_polygon(verts: &mut Vec<[f32; 3]>, faces: &mut Vec<[usize; 3]>, ring: &[usize], flip: bool) {
    debug_assert!(ring.len() >= 3);

    let mut cx = 0.0f32;
    let mut cy = 0.0f32;
    let mut cz = 0.0f32;
    let n = ring.len() as f32;

    for &idx in ring {
        let v = verts[idx];
        cx += v[0];
        cy += v[1];
        cz += v[2];
    }

    cx /= n;
    cy /= n;
    cz /= n;

    let center_index = verts.len();
    verts.push([cx, cy, cz]);

    for i in 0..ring.len() {
        let a = center_index;
        let b = ring[i];
        let c = ring[(i + 1) % ring.len()];

        if !flip {
            faces.push([a + 1, b + 1, c + 1]);
        } else {
            faces.push([a + 1, c + 1, b + 1]);
        }
    }
}

fn add_box(verts: &mut Vec<[f32; 3]>, faces: &mut Vec<[usize; 3]>, min: [f32; 3], max: [f32; 3]) {
    let base = verts.len();
    let minx = min[0];
    let miny = min[1];
    let minz = min[2];
    let maxx = max[0];
    let maxy = max[1];
    let maxz = max[2];

    let box_verts = [
        [minx, miny, minz],
        [maxx, miny, minz],
        [maxx, maxy, minz],
        [minx, maxy, minz],
        [minx, miny, maxz],
        [maxx, miny, maxz],
        [maxx, maxy, maxz],
        [minx, maxy, maxz],
    ];

    verts.extend_from_slice(&box_verts);

    let quads = [
        [0usize, 3, 2, 1],
        [4, 5, 6, 7],
        [0, 4, 7, 3],
        [1, 2, 6, 5],
        [0, 1, 5, 4],
        [3, 7, 6, 2],
    ];

    for q in quads.iter() {
        let a = base + q[0];
        let b = base + q[1];
        let c = base + q[2];
        let d = base + q[3];
        faces.push([a + 1, b + 1, c + 1]);
        faces.push([a + 1, c + 1, d + 1]);
    }
}

fn write_fusuma_with_handle_obj(file: &mut File) {
    let room_height = 3.0_f32;
    let fusuma_height_ratio = 0.85_f32;
    let fusuma_aspect_ratio = 0.5_f32;
    let fusuma_thickness = 0.04_f32;

    let hikite_height_ratio = 0.45_f32;
    let frame_stile_width_ratio = 0.10_f32;
    let hikite_radius_ratio = 0.025_f32;
    let hikite_recess_ratio_of_thickness = 0.15_f32;
    let hikite_front_offset_ratio_of_thickness = 0.02_f32;

    let handle_segments = 12usize;

    let door_h = room_height * fusuma_height_ratio;
    let door_w = door_h * fusuma_aspect_ratio;
    let door_t = fusuma_thickness;

    let hw = door_w * 0.5;
    let ht = door_t * 0.5;
    let bottom = 0.0_f32;
    let top = door_h;

    let stile_width = door_w * frame_stile_width_ratio;
    let hikite_radius = door_h * hikite_radius_ratio;
    let hikite_center_y = bottom + door_h * hikite_height_ratio;
    let hikite_center_x = hw - stile_width * 0.5;
    let hikite_recess_depth = door_t * hikite_recess_ratio_of_thickness;
    let hikite_center_z = ht - hikite_recess_depth;
    let hikite_front_offset = door_t * hikite_front_offset_ratio_of_thickness;
    let hikite_front_z = ht + hikite_front_offset;

    let origin_x = hikite_center_x;
    let origin_y = hikite_center_y;
    let origin_z = hikite_center_z;

    let panel_verts = [
        [-hw, bottom, -ht],
        [hw, bottom, -ht],
        [hw, top, -ht],
        [-hw, top, -ht],
        [-hw, bottom, ht],
        [hw, bottom, ht],
        [hw, top, ht],
        [-hw, top, ht],
    ];

    let mut verts: Vec<[f32; 3]> = Vec::new();
    verts.extend(panel_verts.iter().cloned());

    for i in 0..handle_segments {
        let theta = (i as f32) * 2.0 * PI / (handle_segments as f32);
        let x = hikite_center_x + hikite_radius * theta.cos();
        let y = hikite_center_y + hikite_radius * theta.sin();
        let z = hikite_front_z;
        verts.push([x, y, z]);
    }

    for i in 0..handle_segments {
        let theta = (i as f32) * 2.0 * PI / (handle_segments as f32);
        let x = hikite_center_x + hikite_radius * theta.cos();
        let y = hikite_center_y + hikite_radius * theta.sin();
        let z = hikite_center_z;
        verts.push([x, y, z]);
    }

    writeln!(file, "o fusuma").unwrap();
    for v in &verts {
        writeln!(
            file,
            "v {:.6} {:.6} {:.6}",
            v[0] - origin_x,
            v[1] - origin_y,
            v[2] - origin_z
        )
        .unwrap();
    }

    writeln!(file, "s off").unwrap();

    let panel_quads: [[usize; 4]; 6] = [
        [1, 0, 3, 2],
        [4, 5, 6, 7],
        [0, 4, 7, 3],
        [5, 1, 2, 6],
        [0, 1, 5, 4],
        [3, 7, 6, 2],
    ];

    for q in panel_quads.iter() {
        let a = q[0] + 1;
        let b = q[1] + 1;
        let c = q[2] + 1;
        let d = q[3] + 1;
        writeln!(file, "f {} {} {}", a, b, c).unwrap();
        writeln!(file, "f {} {} {}", a, c, d).unwrap();
    }
    let front_start = panel_verts.len() as i32 + 1;
    let back_start = front_start + handle_segments as i32;

    for i in 0..handle_segments {
        let a1 = front_start + i as i32;
        let a2 = front_start + if i + 1 == handle_segments { 0 } else { i as i32 + 1 };

        let b1 = back_start + i as i32;
        let b2 = back_start + if i + 1 == handle_segments { 0 } else { i as i32 + 1 };

        writeln!(file, "f {} {} {}", a1, a2, b2).unwrap();
        writeln!(file, "f {} {} {}", a1, b2, b1).unwrap();
    }

    for i in 1..(handle_segments - 1) {
        let c0 = back_start;
        let c1 = back_start + i as i32;
        let c2 = back_start + i as i32 + 1;
        writeln!(file, "f {} {} {}", c0, c1, c2).unwrap();
    }
}

fn write_sphere_obj(file: &mut File) {
    let radius = 0.5_f32;
    let stacks = 8;
    let slices = 8;
    let mode = TexcoordMapping::SphericalEquirectangularUnwrapped;

    writeln!(file, "o sphere").unwrap();
    let mut verts = Vec::new();
    verts.push([0.0, radius, 0.0]);

    for i in 1..stacks {
        let t = PI * (i as f32) / (stacks as f32);
        let y = radius * t.cos();
        let r = radius * t.sin();
        for j in 0..slices {
            let p = 2.0 * PI * (j as f32) / (slices as f32);
            let x = r * p.cos();
            let z = r * p.sin();
            verts.push([x, y, z]);
        }
    }
    verts.push([0.0, -radius, 0.0]);
    for v in &verts {
        writeln!(file, "v {:.6} {:.6} {:.6}", v[0], v[1], v[2]).unwrap();
    }

    emit_sphere_normals(file, &verts);
    emit_sphere_texcoords(file, &verts, stacks, slices, &mode);
    writeln!(file, "s off").unwrap();
    emit_sphere_indexed_triangles(file, stacks, slices, &mode);
    println!("OBJ generated with {} vertices", verts.len());
}

fn verify_gltf_attributes(path: &str, stage: &str) {
    println!("Verifying GLTF attributes ({}):", stage);
    let gltf = Gltf::from_slice(&fs::read(path).unwrap()).unwrap();
    let (_, buffers, _) = gltf::import(path).unwrap();

    for (i, mesh) in gltf.meshes().enumerate() {
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

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
fn verify_glb_attributes(glb_path: &str) {
    println!("Verifying final GLB attributes:");
    let final_glb = Gltf::from_slice(&fs::read(glb_path).unwrap()).unwrap();

    for (i, mesh) in final_glb.meshes().enumerate() {
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|_| final_glb.blob.as_ref().map(|b| &b[..]));

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

fn emit_sphere_normals(file: &mut File, verts: &[[f32; 3]]) {
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

fn emit_sphere_texcoords(file: &mut File, verts: &[[f32; 3]], stacks: usize, slices: usize, mode: &TexcoordMapping) {
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
        let s = (j as f32 + 0.5) / (slices as f32);
        writeln!(file, "vt {:.6} {:.6}", s, 1.0).unwrap();
    }
    for i in 1..stacks {
        let t = 1.0 - (i as f32) / (stacks as f32);
        for j in 0..=slices {
            let s = (j as f32) / (slices as f32);
            writeln!(file, "vt {:.6} {:.6}", s, t).unwrap();
        }
    }
    for j in 0..slices {
        let s = (j as f32 + 0.5) / (slices as f32);
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

fn spherical_equirectangular_analytic_st(x: f32, y: f32, z: f32) -> (f32, f32) {
    let r = (x * x + y * y + z * z).sqrt();
    if r == 0.0 {
        return (0.0, 0.0);
    }
    let mut s = f32::atan2(z, x) / (2.0 * PI);
    if s < 0.0 {
        s += 1.0;
    }
    let ny = (y / r).clamp(-1.0, 1.0);
    let t = 1.0 - ny.acos() / PI;
    (s, t)
}

fn planar_projection_xy_st(x: f32, y: f32, _z: f32) -> (f32, f32) {
    let s = (x + 0.5) / 1.0;
    let t = (y + 0.5) / 1.0;
    (s, t)
}
fn run_gltfpack(obj_in: &str, gltf_out: &str) {
    let output = Command::new("gltfpack")
        .arg("-i")
        .arg(obj_in)
        .arg("-o")
        .arg(gltf_out)
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

fn convert_to_glb(gltf_in: &str, glb_out: &str) {
    let output = Command::new("gltfpack")
        .arg("-i")
        .arg(gltf_in)
        .arg("-o")
        .arg(glb_out)
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

fn fill_vertex_colors_gltf(gltf_path_str: &str) {
    println!("Reading GLTF...");
    let (gltf, buffers, _) = gltf::import(gltf_path_str).unwrap();

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
    let gltf_path = std::path::Path::new(gltf_path_str);
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
