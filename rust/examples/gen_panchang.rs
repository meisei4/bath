use gltf::json::deserialize::from_str;
use gltf::json::serialize::to_string_pretty;
use gltf::json::validation::USize64;
use gltf::json::*;
use std::f32::consts::PI;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::process::Command;

#[derive(Clone, Copy)]
struct V3 {
    x: f32,
    y: f32,
    z: f32,
}

fn v3(x: f32, y: f32, z: f32) -> V3 {
    V3 { x, y, z }
}

fn v3_sub(a: V3, b: V3) -> V3 {
    v3(a.x - b.x, a.y - b.y, a.z - b.z)
}

fn v3_add(a: V3, b: V3) -> V3 {
    v3(a.x + b.x, a.y + b.y, a.z + b.z)
}

fn v3_scale(a: V3, s: f32) -> V3 {
    v3(a.x * s, a.y * s, a.z * s)
}

fn v3_dot(a: V3, b: V3) -> f32 {
    a.x * b.x + a.y * b.y + a.z * b.z
}

fn v3_cross(a: V3, b: V3) -> V3 {
    v3(a.y * b.z - a.z * b.y, a.z * b.x - a.x * b.z, a.x * b.y - a.y * b.x)
}

fn v3_norm(a: V3) -> V3 {
    let l = (a.x * a.x + a.y * a.y + a.z * a.z).sqrt();
    if l < 1e-8 {
        return v3(0.0, 0.0, 1.0);
    }
    v3_scale(a, 1.0 / l)
}

fn v3_len(a: V3) -> f32 {
    (a.x * a.x + a.y * a.y + a.z * a.z).sqrt()
}

fn rotate_around_axis(v: V3, axis: V3, angle: f32) -> V3 {
    let c = angle.cos();
    let s = angle.sin();
    let d = v3_dot(axis, v);
    v3_add(
        v3_add(v3_scale(v, c), v3_scale(v3_cross(axis, v), s)),
        v3_scale(axis, d * (1.0 - c)),
    )
}

struct PanChangConfig {
    name: &'static str,
    grid_w: usize,
    grid_h: usize,
    grid_spacing: f32,
    lobe_offset: f32,
    strand_radius: f32,
    crossing_height: f32,
    seg_samples: usize,
    arc_samples: usize,
    corner_samples: usize,
    tube_sides: usize,
    rotate_deg: f32,
    corner_ears: bool,
    side_ear_factor: f32,
    inner_lobe_factor: f32,
}

fn seg(path: &mut Vec<V3>, x0: f32, y0: f32, z0: f32, x1: f32, y1: f32, z1: f32, n: usize) {
    for i in 0..n {
        let t = i as f32 / n as f32;
        path.push(v3(
            x0 + (x1 - x0) * t,
            y0 + (y1 - y0) * t,
            z0 + (z1 - z0) * (1.0 - (PI * t).cos()) * 0.5,
        ));
    }
}

fn arc(path: &mut Vec<V3>, cx: f32, cy: f32, r: f32, a0: f32, a1: f32, n: usize) {
    for i in 0..n {
        let t = i as f32 / n as f32;
        let a = a0 + (a1 - a0) * t;
        path.push(v3(cx + r * a.cos(), cy + r * a.sin(), 0.0));
    }
}

fn bezier(path: &mut Vec<V3>, p0: V3, p1: V3, p2: V3, p3: V3, n: usize) {
    for i in 0..n {
        let t = i as f32 / n as f32;
        let u = 1.0 - t;
        path.push(v3(
            u * u * u * p0.x + 3.0 * u * u * t * p1.x + 3.0 * u * t * t * p2.x + t * t * t * p3.x,
            u * u * u * p0.y + 3.0 * u * u * t * p1.y + 3.0 * u * t * t * p2.y + t * t * t * p3.y,
            u * u * u * p0.z + 3.0 * u * u * t * p1.z + 3.0 * u * t * t * p2.z + t * t * t * p3.z,
        ));
    }
}

fn gen_pass(p: &mut Vec<V3>, n: usize, gs: f32, oh: f32, sn: usize, is_h: bool, idx: usize, fwd: bool, dp: f32) {
    let col_x = |i: usize| -> f32 { (i as f32 - (n - 1) as f32 / 2.0) * gs };
    let row_y = |j: usize| -> f32 { (j as f32 - (n - 1) as f32 / 2.0) * gs };
    let z_h = |i: usize, j: usize| -> f32 {
        if (i + j) % 2 == 0 {
            oh
        } else {
            0.0
        }
    };
    let z_v = |i: usize, j: usize| -> f32 {
        if (i + j) % 2 == 0 {
            0.0
        } else {
            oh
        }
    };

    if is_h {
        let row = idx;
        if fwd {
            seg(p, col_x(0) - dp, row_y(row), 0.0, col_x(0), row_y(row), z_h(0, row), sn);
            for k in 1..n {
                seg(
                    p,
                    col_x(k - 1),
                    row_y(row),
                    z_h(k - 1, row),
                    col_x(k),
                    row_y(row),
                    z_h(k, row),
                    sn,
                );
            }
            seg(
                p,
                col_x(n - 1),
                row_y(row),
                z_h(n - 1, row),
                col_x(n - 1) + dp,
                row_y(row),
                0.0,
                sn,
            );
        } else {
            seg(
                p,
                col_x(n - 1) + dp,
                row_y(row),
                0.0,
                col_x(n - 1),
                row_y(row),
                z_h(n - 1, row),
                sn,
            );
            for k in 1..n {
                seg(
                    p,
                    col_x(n - k),
                    row_y(row),
                    z_h(n - k, row),
                    col_x(n - k - 1),
                    row_y(row),
                    z_h(n - k - 1, row),
                    sn,
                );
            }
            seg(p, col_x(0), row_y(row), z_h(0, row), col_x(0) - dp, row_y(row), 0.0, sn);
        }
    } else {
        let col = idx;
        if fwd {
            seg(p, col_x(col), row_y(0) - dp, 0.0, col_x(col), row_y(0), z_v(col, 0), sn);
            for k in 1..n {
                seg(
                    p,
                    col_x(col),
                    row_y(k - 1),
                    z_v(col, k - 1),
                    col_x(col),
                    row_y(k),
                    z_v(col, k),
                    sn,
                );
            }
            seg(
                p,
                col_x(col),
                row_y(n - 1),
                z_v(col, n - 1),
                col_x(col),
                row_y(n - 1) + dp,
                0.0,
                sn,
            );
        } else {
            seg(
                p,
                col_x(col),
                row_y(n - 1) + dp,
                0.0,
                col_x(col),
                row_y(n - 1),
                z_v(col, n - 1),
                sn,
            );
            for k in 1..n {
                seg(
                    p,
                    col_x(col),
                    row_y(n - k),
                    z_v(col, n - k),
                    col_x(col),
                    row_y(n - k - 1),
                    z_v(col, n - k - 1),
                    sn,
                );
            }
            seg(p, col_x(col), row_y(0), z_v(col, 0), col_x(col), row_y(0) - dp, 0.0, sn);
        }
    }
}

fn pass_exit(n: usize, gs: f32, is_h: bool, idx: usize, fwd: bool, dp: f32) -> (f32, f32, f32, f32) {
    let col_x = |i: usize| -> f32 { (i as f32 - (n - 1) as f32 / 2.0) * gs };
    let row_y = |j: usize| -> f32 { (j as f32 - (n - 1) as f32 / 2.0) * gs };
    match (is_h, fwd) {
        (true, true) => (col_x(n - 1) + dp, row_y(idx), 1.0, 0.0),
        (true, false) => (col_x(0) - dp, row_y(idx), -1.0, 0.0),
        (false, true) => (col_x(idx), row_y(n - 1) + dp, 0.0, 1.0),
        (false, false) => (col_x(idx), row_y(0) - dp, 0.0, -1.0),
    }
}

fn pass_entry(n: usize, gs: f32, is_h: bool, idx: usize, fwd: bool, dp: f32) -> (f32, f32, f32, f32) {
    let col_x = |i: usize| -> f32 { (i as f32 - (n - 1) as f32 / 2.0) * gs };
    let row_y = |j: usize| -> f32 { (j as f32 - (n - 1) as f32 / 2.0) * gs };
    match (is_h, fwd) {
        (true, true) => (col_x(0) - dp, row_y(idx), 1.0, 0.0),
        (true, false) => (col_x(n - 1) + dp, row_y(idx), -1.0, 0.0),
        (false, true) => (col_x(idx), row_y(0) - dp, 0.0, 1.0),
        (false, false) => (col_x(idx), row_y(n - 1) + dp, 0.0, -1.0),
    }
}

fn build_ring(
    cfg: &PanChangConfig,
    passes: &[(bool, usize, bool, usize)],
    corner_arcs: bool,
    transition_z: f32,
) -> Vec<V3> {
    let n = cfg.grid_w;
    let gs = cfg.grid_spacing;
    let d = cfg.lobe_offset;
    let oh = cfg.crossing_height;
    let sn = cfg.seg_samples;
    let cn = cfg.corner_samples;
    let ear_cn = cn * 3;
    let bn = cn * 2;
    let np = passes.len();

    let dp_for = |layer: usize| -> f32 {
        if layer == 0 {
            d
        } else {
            d * cfg.inner_lobe_factor
        }
    };

    let mut p: Vec<V3> = Vec::with_capacity(8192);

    for pi in 0..np {
        let (is_h, idx, fwd, layer) = passes[pi];
        let dp = dp_for(layer);

        gen_pass(&mut p, n, gs, oh, sn, is_h, idx, fwd, dp);

        let ni = (pi + 1) % np;
        let (nh, ni_idx, nf, nl) = passes[ni];
        let dp_next = dp_for(nl);

        let (ex, ey, etx, ety) = pass_exit(n, gs, is_h, idx, fwd, dp);
        let (nx, ny, ntx, nty) = pass_entry(n, gs, nh, ni_idx, nf, dp_next);

        if corner_arcs && cfg.corner_ears {
            let (cx, cy) = if is_h { (ex, ny) } else { (nx, ey) };
            let a0 = (ey - cy).atan2(ex - cx);
            let a1 = a0 - 3.0 * PI / 2.0;
            arc(&mut p, cx, cy, d, a0, a1, ear_cn);
        } else {
            let dist = ((nx - ex) * (nx - ex) + (ny - ey) * (ny - ey)).sqrt();
            let k = dist * cfg.side_ear_factor;
            bezier(
                &mut p,
                v3(ex, ey, 0.0),
                v3(ex + k * etx, ey + k * ety, transition_z),
                v3(nx - k * ntx, ny - k * nty, transition_z),
                v3(nx, ny, 0.0),
                bn * 2,
            );
        }
    }

    p
}

fn build_serpentine_path(cfg: &PanChangConfig) -> Vec<V3> {
    let gw = cfg.grid_w;
    let gh = cfg.grid_h;
    let gs = cfg.grid_spacing;
    let d = cfg.lobe_offset;
    let oh = cfg.crossing_height;
    let sn = cfg.seg_samples;
    let an = cfg.arc_samples;
    let cn = cfg.corner_samples;

    let col_x = |i: usize| -> f32 { (i as f32 - (gw - 1) as f32 / 2.0) * gs };
    let row_y = |j: usize| -> f32 { (j as f32 - (gh - 1) as f32 / 2.0) * gs };
    let z_h = |i: usize, j: usize| -> f32 {
        if (i + j) % 2 == 0 {
            oh
        } else {
            0.0
        }
    };
    let z_v = |i: usize, j: usize| -> f32 {
        if (i + j) % 2 == 0 {
            0.0
        } else {
            oh
        }
    };
    let arc_r = gs / 2.0;
    let ear_cn = cn * 3;

    let mut p: Vec<V3> = Vec::with_capacity(16384);

    seg(&mut p, col_x(0) - d, row_y(0), 0.0, col_x(0), row_y(0), z_h(0, 0), sn);
    for k in 1..gw {
        seg(
            &mut p,
            col_x(k - 1),
            row_y(0),
            z_h(k - 1, 0),
            col_x(k),
            row_y(0),
            z_h(k, 0),
            sn,
        );
    }

    for row in 1..gh {
        let prev = row - 1;
        let y_prev = row_y(prev);
        let y_cur = row_y(row);
        let going_left = (row % 2) == 1;

        if going_left {
            seg(
                &mut p,
                col_x(gw - 1),
                y_prev,
                z_h(gw - 1, prev),
                col_x(gw - 1) + d,
                y_prev,
                0.0,
                sn,
            );
            arc(
                &mut p,
                col_x(gw - 1) + d,
                (y_prev + y_cur) / 2.0,
                arc_r,
                -PI / 2.0,
                PI / 2.0,
                an,
            );
            seg(
                &mut p,
                col_x(gw - 1) + d,
                y_cur,
                0.0,
                col_x(gw - 1),
                y_cur,
                z_h(gw - 1, row),
                sn,
            );
            for k in 1..gw {
                let ci = gw - k;
                let cj = gw - k - 1;
                seg(
                    &mut p,
                    col_x(ci),
                    y_cur,
                    z_h(ci, row),
                    col_x(cj),
                    y_cur,
                    z_h(cj, row),
                    sn,
                );
            }
        } else {
            seg(&mut p, col_x(0), y_prev, z_h(0, prev), col_x(0) - d, y_prev, 0.0, sn);
            arc(
                &mut p,
                col_x(0) - d,
                (y_prev + y_cur) / 2.0,
                arc_r,
                -PI / 2.0,
                -3.0 * PI / 2.0,
                an,
            );
            seg(&mut p, col_x(0) - d, y_cur, 0.0, col_x(0), y_cur, z_h(0, row), sn);
            for k in 1..gw {
                seg(
                    &mut p,
                    col_x(k - 1),
                    y_cur,
                    z_h(k - 1, row),
                    col_x(k),
                    y_cur,
                    z_h(k, row),
                    sn,
                );
            }
        }
    }

    seg(
        &mut p,
        col_x(gw - 1),
        row_y(gh - 1),
        z_h(gw - 1, gh - 1),
        col_x(gw - 1) + d,
        row_y(gh - 1),
        0.0,
        sn,
    );
    if cfg.corner_ears {
        arc(&mut p, col_x(gw - 1) + d, row_y(gh - 1) + d, d, -PI / 2.0, PI, ear_cn);
    } else {
        arc(&mut p, col_x(gw - 1) + d, row_y(gh - 1) + d, d, -PI / 2.0, -PI, cn);
    }
    seg(
        &mut p,
        col_x(gw - 1),
        row_y(gh - 1) + d,
        0.0,
        col_x(gw - 1),
        row_y(gh - 1),
        z_v(gw - 1, gh - 1),
        sn,
    );

    for k in 1..gh {
        let jp = gh - k;
        let jc = gh - k - 1;
        seg(
            &mut p,
            col_x(gw - 1),
            row_y(jp),
            z_v(gw - 1, jp),
            col_x(gw - 1),
            row_y(jc),
            z_v(gw - 1, jc),
            sn,
        );
    }

    for ci in 1..gw {
        let col = gw - 1 - ci;
        let prev_col = col + 1;
        let x_prev = col_x(prev_col);
        let x_cur = col_x(col);
        let going_up = (ci % 2) == 1;

        if going_up {
            seg(
                &mut p,
                x_prev,
                row_y(0),
                z_v(prev_col, 0),
                x_prev,
                row_y(0) - d,
                0.0,
                sn,
            );
            arc(&mut p, (x_prev + x_cur) / 2.0, row_y(0) - d, arc_r, 0.0, -PI, an);
            seg(&mut p, x_cur, row_y(0) - d, 0.0, x_cur, row_y(0), z_v(col, 0), sn);
            for k in 1..gh {
                seg(
                    &mut p,
                    x_cur,
                    row_y(k - 1),
                    z_v(col, k - 1),
                    x_cur,
                    row_y(k),
                    z_v(col, k),
                    sn,
                );
            }
        } else {
            seg(
                &mut p,
                x_prev,
                row_y(gh - 1),
                z_v(prev_col, gh - 1),
                x_prev,
                row_y(gh - 1) + d,
                0.0,
                sn,
            );
            arc(&mut p, (x_prev + x_cur) / 2.0, row_y(gh - 1) + d, arc_r, 0.0, PI, an);
            seg(
                &mut p,
                x_cur,
                row_y(gh - 1) + d,
                0.0,
                x_cur,
                row_y(gh - 1),
                z_v(col, gh - 1),
                sn,
            );
            for k in 1..gh {
                let jp = gh - k;
                let jc = gh - k - 1;
                seg(
                    &mut p,
                    x_cur,
                    row_y(jp),
                    z_v(col, jp),
                    x_cur,
                    row_y(jc),
                    z_v(col, jc),
                    sn,
                );
            }
        }
    }

    let v_ends_at_bottom = ((gw - 1) % 2) == 0;
    if v_ends_at_bottom {
        seg(&mut p, col_x(0), row_y(0), z_v(0, 0), col_x(0), row_y(0) - d, 0.0, sn);
        if cfg.corner_ears {
            arc(&mut p, col_x(0) - d, row_y(0) - d, d, 0.0, -3.0 * PI / 2.0, ear_cn);
        } else {
            arc(&mut p, col_x(0) - d, row_y(0) - d, d, 0.0, PI / 2.0, cn);
        }
    } else {
        seg(
            &mut p,
            col_x(0),
            row_y(gh - 1),
            z_v(0, gh - 1),
            col_x(0),
            row_y(gh - 1) + d,
            0.0,
            sn,
        );
        if cfg.corner_ears {
            arc(&mut p, col_x(0) - d, row_y(gh - 1) + d, d, 0.0, 3.0 * PI / 2.0, ear_cn);
        } else {
            arc(&mut p, col_x(0) - d, row_y(gh - 1) + d, d, 0.0, -PI / 2.0, cn);
        }
        seg(
            &mut p,
            col_x(0) - d,
            row_y(gh - 1) + d,
            0.0,
            col_x(0) - d,
            row_y(0) - d,
            0.0,
            sn,
        );
        if cfg.corner_ears {
            arc(&mut p, col_x(0) - d, row_y(0) - d, d, PI / 2.0, -PI, ear_cn);
        } else {
            arc(&mut p, col_x(0) - d, row_y(0) - d, d, PI / 2.0, 0.0, cn);
        }
    }

    p
}

fn write_svg_paths(paths: &[(&[V3], bool)], filename: &str) {
    let svg_path = format!("/Users/adduser/meshdump/{}.svg", filename);
    let mut file = match File::create(&svg_path) {
        Ok(f) => f,
        Err(_) => return,
    };

    let mut min_x = f32::MAX;
    let mut max_x = f32::MIN;
    let mut min_y = f32::MAX;
    let mut max_y = f32::MIN;
    for &(path, _) in paths {
        for pt in path {
            min_x = min_x.min(pt.x);
            max_x = max_x.max(pt.x);
            min_y = min_y.min(pt.y);
            max_y = max_y.max(pt.y);
        }
    }
    let margin = 0.5;
    let w = max_x - min_x + 2.0 * margin;
    let h = max_y - min_y + 2.0 * margin;
    let scale = 100.0;

    writeln!(
        file,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"{} {} {} {}\">",
        (min_x - margin) * scale,
        -(max_y + margin) * scale,
        w * scale,
        h * scale
    )
    .unwrap();
    writeln!(
        file,
        "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"#222\"/>",
        (min_x - margin) * scale,
        -(max_y + margin) * scale,
        w * scale,
        h * scale
    )
    .unwrap();

    let colors = ["rgb(255,100,100)", "rgb(100,200,255)"];
    for (pi, &(path, is_open)) in paths.iter().enumerate() {
        let seg_count = if is_open { path.len() - 1 } else { path.len() };
        let color = colors[pi % colors.len()];
        for i in 0..seg_count {
            let j = (i + 1) % path.len();
            writeln!(
                file,
                "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{}\" stroke-width=\"1.5\"/>",
                path[i].x * scale,
                -path[i].y * scale,
                path[j].x * scale,
                -path[j].y * scale,
                color
            )
            .unwrap();
        }
    }

    if let Some(&(path, _)) = paths.first() {
        if !path.is_empty() {
            writeln!(
                file,
                "<circle cx=\"{}\" cy=\"{}\" r=\"3\" fill=\"lime\"/>",
                path[0].x * scale,
                -path[0].y * scale
            )
            .unwrap();
        }
    }

    writeln!(file, "</svg>").unwrap();
    println!("    SVG: {}.svg", filename);
}

fn write_svg(path: &[V3], filename: &str) {
    let svg_path = format!("/Users/adduser/meshdump/{}.svg", filename);
    let mut file = match File::create(&svg_path) {
        Ok(f) => f,
        Err(_) => return,
    };

    let mut min_x = f32::MAX;
    let mut max_x = f32::MIN;
    let mut min_y = f32::MAX;
    let mut max_y = f32::MIN;
    for pt in path {
        min_x = min_x.min(pt.x);
        max_x = max_x.max(pt.x);
        min_y = min_y.min(pt.y);
        max_y = max_y.max(pt.y);
    }
    let margin = 0.5;
    let w = max_x - min_x + 2.0 * margin;
    let h = max_y - min_y + 2.0 * margin;
    let scale = 100.0;

    writeln!(
        file,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"{} {} {} {}\">",
        (min_x - margin) * scale,
        -(max_y + margin) * scale,
        w * scale,
        h * scale
    )
    .unwrap();
    writeln!(
        file,
        "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"#222\"/>",
        (min_x - margin) * scale,
        -(max_y + margin) * scale,
        w * scale,
        h * scale
    )
    .unwrap();

    for i in 0..path.len() {
        let j = (i + 1) % path.len();
        let z_avg = (path[i].z + path[j].z) / 2.0;
        let r_val = (128.0 + 127.0 * z_avg * 10.0).clamp(0.0, 255.0) as u8;
        let b_val = (128.0 - 127.0 * z_avg * 10.0).clamp(0.0, 255.0) as u8;
        writeln!(
            file,
            "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"rgb({},80,{})\" stroke-width=\"1.5\"/>",
            path[i].x * scale,
            -path[i].y * scale,
            path[j].x * scale,
            -path[j].y * scale,
            r_val,
            b_val
        )
        .unwrap();
    }

    writeln!(
        file,
        "<circle cx=\"{}\" cy=\"{}\" r=\"3\" fill=\"lime\"/>",
        path[0].x * scale,
        -path[0].y * scale
    )
    .unwrap();

    writeln!(file, "</svg>").unwrap();
    println!("    SVG: {}.svg", filename);
}

fn rotate_path(path: &mut [V3], deg: f32) {
    let a = deg * PI / 180.0;
    let ca = a.cos();
    let sa = a.sin();
    for pt in path.iter_mut() {
        let x = pt.x;
        let y = pt.y;
        pt.x = x * ca - y * sa;
        pt.y = x * sa + y * ca;
    }
}

fn sweep_tube(
    path: &[V3],
    tube_sides: usize,
    strand_radius: f32,
    closed: bool,
    verts: &mut Vec<[f32; 3]>,
    norms: &mut Vec<[f32; 3]>,
    faces: &mut Vec<[usize; 3]>,
) {
    let n = path.len();
    let m = tube_sides;
    let base = verts.len();
    let cr = strand_radius;

    let tangents: Vec<V3> = (0..n)
        .map(|i| {
            if closed {
                v3_norm(v3_sub(path[(i + 1) % n], path[(i + n - 1) % n]))
            } else if i == 0 {
                v3_norm(v3_sub(path[1], path[0]))
            } else if i == n - 1 {
                v3_norm(v3_sub(path[n - 1], path[n - 2]))
            } else {
                v3_norm(v3_sub(path[i + 1], path[i - 1]))
            }
        })
        .collect();

    let ref_vec = if tangents[0].z.abs() < 0.9 {
        v3(0.0, 0.0, 1.0)
    } else {
        v3(1.0, 0.0, 0.0)
    };
    let mut normal = v3_norm(v3_cross(tangents[0], ref_vec));

    let mut normals_vec: Vec<V3> = Vec::with_capacity(n);
    let mut binormals_vec: Vec<V3> = Vec::with_capacity(n);

    normals_vec.push(normal);
    binormals_vec.push(v3_norm(v3_cross(normal, tangents[0])));

    for i in 1..n {
        let cross = v3_cross(tangents[i - 1], tangents[i]);
        let cross_len = v3_len(cross);
        if cross_len > 1e-8 {
            let axis = v3_scale(cross, 1.0 / cross_len);
            let dot = v3_dot(tangents[i - 1], tangents[i]).clamp(-1.0, 1.0);
            normal = rotate_around_axis(normal, axis, dot.acos());
        }
        let nt = v3_dot(normal, tangents[i]);
        normal = v3_norm(v3_sub(normal, v3_scale(tangents[i], nt)));
        let binormal = v3_norm(v3_cross(normal, tangents[i]));
        normals_vec.push(normal);
        binormals_vec.push(binormal);
    }

    if closed {
        let mut transported = normals_vec[n - 1];
        let cross = v3_cross(tangents[n - 1], tangents[0]);
        let cross_len = v3_len(cross);
        if cross_len > 1e-8 {
            let axis = v3_scale(cross, 1.0 / cross_len);
            let dot = v3_dot(tangents[n - 1], tangents[0]).clamp(-1.0, 1.0);
            transported = rotate_around_axis(transported, axis, dot.acos());
        }
        let nt = v3_dot(transported, tangents[0]);
        transported = v3_norm(v3_sub(transported, v3_scale(tangents[0], nt)));

        let cos_twist = v3_dot(transported, normals_vec[0]).clamp(-1.0, 1.0);
        let cross_twist = v3_cross(transported, normals_vec[0]);
        let sign = if v3_dot(cross_twist, tangents[0]) >= 0.0 {
            1.0
        } else {
            -1.0
        };
        let twist_total = sign * cos_twist.acos();

        for i in 0..n {
            let correction = -twist_total * (i as f32) / (n as f32);
            normals_vec[i] = rotate_around_axis(normals_vec[i], tangents[i], correction);
            binormals_vec[i] = v3_norm(v3_cross(normals_vec[i], tangents[i]));
        }
    }

    for i in 0..n {
        for j in 0..m {
            let angle = 2.0 * PI * j as f32 / m as f32;
            let off = v3_add(
                v3_scale(normals_vec[i], angle.cos() * cr),
                v3_scale(binormals_vec[i], angle.sin() * cr),
            );
            let pos = v3_add(path[i], off);
            let nm = v3_norm(off);
            verts.push([pos.x, pos.y, pos.z]);
            norms.push([nm.x, nm.y, nm.z]);
        }
    }

    let face_count = if closed { n } else { n - 1 };
    for i in 0..face_count {
        let i1 = if closed { (i + 1) % n } else { i + 1 };
        for j in 0..m {
            let j1 = (j + 1) % m;
            let a = base + i * m + j;
            let b = base + i * m + j1;
            let c = base + i1 * m + j;
            let dd = base + i1 * m + j1;
            faces.push([a + 1, c + 1, dd + 1]);
            faces.push([a + 1, dd + 1, b + 1]);
        }
    }
}

#[allow(dead_code)]
fn gen_uv_sphere(
    cx: f32,
    cy: f32,
    cz: f32,
    r: f32,
    stacks: usize,
    slices: usize,
    verts: &mut Vec<[f32; 3]>,
    norms: &mut Vec<[f32; 3]>,
    faces: &mut Vec<[usize; 3]>,
) {
    let base = verts.len();
    // Top pole
    verts.push([cx, cy + r, cz]);
    norms.push([0.0, 1.0, 0.0]);
    // Latitude rings
    for i in 1..stacks {
        let phi = PI * i as f32 / stacks as f32;
        let sp = phi.sin();
        let cp = phi.cos();
        for j in 0..slices {
            let theta = 2.0 * PI * j as f32 / slices as f32;
            let nx = sp * theta.cos();
            let ny = cp;
            let nz = sp * theta.sin();
            verts.push([cx + r * nx, cy + r * ny, cz + r * nz]);
            norms.push([nx, ny, nz]);
        }
    }
    // Bottom pole
    verts.push([cx, cy - r, cz]);
    norms.push([0.0, -1.0, 0.0]);

    // Top cap
    for j in 0..slices {
        let j1 = (j + 1) % slices;
        faces.push([base + 1, base + 1 + j + 1, base + 1 + j1 + 1]);
    }
    // Middle quads
    for i in 0..(stacks - 2) {
        for j in 0..slices {
            let j1 = (j + 1) % slices;
            let a = base + 1 + i * slices + j;
            let b = base + 1 + i * slices + j1;
            let c = base + 1 + (i + 1) * slices + j;
            let d = base + 1 + (i + 1) * slices + j1;
            faces.push([a + 1, c + 1, d + 1]);
            faces.push([a + 1, d + 1, b + 1]);
        }
    }
    // Bottom cap
    let bot = base + 1 + (stacks - 1) * slices;
    let last_ring = base + 1 + (stacks - 2) * slices;
    for j in 0..slices {
        let j1 = (j + 1) % slices;
        faces.push([last_ring + j + 1, bot + 1, last_ring + j1 + 1]);
    }
}

fn gen_cylinder(
    cx: f32,
    cy: f32,
    cz: f32,
    r: f32,
    h: f32,
    slices: usize,
    verts: &mut Vec<[f32; 3]>,
    norms: &mut Vec<[f32; 3]>,
    faces: &mut Vec<[usize; 3]>,
) {
    let base = verts.len();
    let hh = h * 0.5;
    // Top center
    verts.push([cx, cy + hh, cz]);
    norms.push([0.0, 1.0, 0.0]);
    // Top ring
    for j in 0..slices {
        let theta = 2.0 * PI * j as f32 / slices as f32;
        let nx = theta.cos();
        let nz = theta.sin();
        verts.push([cx + r * nx, cy + hh, cz + r * nz]);
        norms.push([nx, 0.0, nz]);
    }
    // Bottom ring
    for j in 0..slices {
        let theta = 2.0 * PI * j as f32 / slices as f32;
        let nx = theta.cos();
        let nz = theta.sin();
        verts.push([cx + r * nx, cy - hh, cz + r * nz]);
        norms.push([nx, 0.0, nz]);
    }
    // Bottom center
    verts.push([cx, cy - hh, cz]);
    norms.push([0.0, -1.0, 0.0]);

    let tc = base;
    let tr = base + 1;
    let br = base + 1 + slices;
    let bc = base + 1 + 2 * slices;
    // Top cap
    for j in 0..slices {
        let j1 = (j + 1) % slices;
        faces.push([tc + 1, (tr + j) + 1, (tr + j1) + 1]);
    }
    // Side quads
    for j in 0..slices {
        let j1 = (j + 1) % slices;
        faces.push([(tr + j) + 1, (br + j1) + 1, (br + j) + 1]);
        faces.push([(tr + j) + 1, (tr + j1) + 1, (br + j1) + 1]);
    }
    // Bottom cap
    for j in 0..slices {
        let j1 = (j + 1) % slices;
        faces.push([bc + 1, (br + j1) + 1, (br + j) + 1]);
    }
}

#[allow(dead_code)]
fn build_button_knot_path(cx: f32, cy: f32, cz: f32, r: f32, samples: usize) -> Vec<V3> {
    // Trefoil knot — the Chinese button knot (纽扣结) is a woven ball
    // that closely resembles a tightened trefoil. This parametric form
    // produces a 3-crossing closed loop that, when tube-swept, gives
    // the characteristic over-under woven sphere appearance.
    let scale = r / 3.0;
    let mut path = Vec::with_capacity(samples);
    for i in 0..samples {
        let t = 2.0 * PI * i as f32 / samples as f32;
        path.push(v3(
            cx + scale * (t.sin() + 2.0 * (2.0 * t).sin()),
            cy + scale * (t.cos() - 2.0 * (2.0 * t).cos()),
            cz + scale * (-(3.0 * t).sin()),
        ));
    }
    path
}

fn write_obj(file: &mut File, name: &str, verts: &[[f32; 3]], norms: &[[f32; 3]], faces: &[[usize; 3]]) {
    writeln!(file, "o {}", name).unwrap();
    writeln!(file, "s off").unwrap();
    for v in verts {
        writeln!(file, "v {:.6} {:.6} {:.6}", v[0], v[1], v[2]).unwrap();
    }
    for n in norms {
        writeln!(file, "vn {:.4} {:.4} {:.4}", n[0], n[1], n[2]).unwrap();
    }
    for f in faces {
        writeln!(file, "f {}//{} {}//{} {}//{}", f[0], f[0], f[1], f[1], f[2], f[2]).unwrap();
    }
}

/// Returns the rotated knot strand paths without sweeping.
/// For two-ring (even grid): (outer_path, Some(inner_path))
/// For serpentine (odd grid): (single_path, None)
fn build_knot_paths(cfg: &PanChangConfig) -> (Vec<V3>, Option<Vec<V3>>) {
    if cfg.grid_w % 2 == 0 {
        let n = cfg.grid_w;
        let mut outer_passes = Vec::new();
        let mut inner_passes = Vec::new();
        for layer in 0..n {
            let lo = layer;
            let hi = n - 1 - layer;
            if lo > hi {
                break;
            }
            if lo == hi {
                if layer == 0 {
                    outer_passes.push((true, lo, true, layer));
                    outer_passes.push((false, lo, true, layer));
                } else {
                    inner_passes.push((true, lo, true, layer));
                    inner_passes.push((false, lo, true, layer));
                }
                break;
            }
            let group = [
                (true, lo, true, layer),
                (false, hi, true, layer),
                (true, hi, false, layer),
                (false, lo, false, layer),
            ];
            if layer == 0 {
                outer_passes.extend_from_slice(&group);
            } else {
                inner_passes.extend_from_slice(&group);
            }
        }
        let mut outer = build_ring(cfg, &outer_passes, true, 0.0);
        let mut inner = build_ring(cfg, &inner_passes, false, -cfg.crossing_height);
        rotate_path(&mut outer, cfg.rotate_deg);
        rotate_path(&mut inner, cfg.rotate_deg);
        (outer, Some(inner))
    } else {
        let mut path = build_serpentine_path(cfg);
        if cfg.rotate_deg.abs() > 0.001 {
            rotate_path(&mut path, cfg.rotate_deg);
        }
        (path, None)
    }
}

#[allow(dead_code)]
fn build_knot_mesh(cfg: &PanChangConfig) -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[usize; 3]>) {
    let mut verts = Vec::new();
    let mut norms = Vec::new();
    let mut faces = Vec::new();
    let (main_path, inner_path) = build_knot_paths(cfg);
    sweep_tube(
        &main_path,
        cfg.tube_sides,
        cfg.strand_radius,
        true,
        &mut verts,
        &mut norms,
        &mut faces,
    );
    if let Some(ref inner) = inner_path {
        sweep_tube(
            inner,
            cfg.tube_sides,
            cfg.strand_radius,
            true,
            &mut verts,
            &mut norms,
            &mut faces,
        );
    }
    (verts, norms, faces)
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
        println!("    gltfpack OBJ->GLTF: SUCCESS");
    } else {
        eprintln!("    gltfpack OBJ->GLTF: FAILED");
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
        println!("    gltfpack GLTF->GLB: SUCCESS");
    } else {
        eprintln!("    gltfpack GLTF->GLB: FAILED");
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
    }
}

fn fill_vertex_colors_gltf(gltf_path_str: &str) {
    let (gltf, buffers, _) = gltf::import(gltf_path_str).unwrap();

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
                all_colors.push(colors);
            }
        }
    }

    let gltf_path = std::path::Path::new(gltf_path_str);
    let mut root: Root = from_str(&fs::read_to_string(gltf_path).unwrap()).unwrap();
    let bin_path = gltf_path.with_extension("bin");
    let mut bin_data = fs::read(&bin_path).unwrap_or_default();

    for (mesh_idx, colors) in all_colors.iter().enumerate() {
        let color_bytes: Vec<u8> = colors.iter().flat_map(|c| c.iter().copied()).collect();
        let color_offset = bin_data.len();
        bin_data.extend_from_slice(&color_bytes);

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
            }
        }
    }

    if let Some(buffer) = root.buffers.get_mut(0) {
        buffer.byte_length = USize64::from(bin_data.len());
    }

    fs::write(&bin_path, &bin_data).unwrap();
    fs::write(gltf_path, to_string_pretty(&root).unwrap()).unwrap();
}

fn generate_glb(obj_path: &str, name: &str) {
    let assets_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/meshes");
    let _ = fs::create_dir_all(assets_dir);

    let tmp_dir = "/tmp/panchang_glb";
    let _ = fs::create_dir_all(tmp_dir);

    let gltf_path = format!("{}/{}.gltf", tmp_dir, name);
    let glb_path = format!("{}/{}_PREBAKE.glb", assets_dir, name);

    // OBJ -> GLTF+BIN (preserving vertices)
    run_gltfpack(obj_path, &gltf_path);

    // Inject COLOR_0 vertex colors
    fill_vertex_colors_gltf(&gltf_path);

    // GLTF+BIN -> GLB
    convert_to_glb(&gltf_path, &glb_path);

    if let Ok(meta) = fs::metadata(&glb_path) {
        println!("    GLB: {}_PREBAKE.glb ({} bytes)", name, meta.len());
    }

    // Cleanup temp files
    let _ = fs::remove_file(&gltf_path);
    let _ = fs::remove_file(format!("{}/{}.bin", tmp_dir, name));
}

fn generate_knot(cfg: &PanChangConfig, svg: bool) -> Option<String> {
    if cfg.grid_w % 2 == 0 {
        let n = cfg.grid_w;
        let mut outer_passes: Vec<(bool, usize, bool, usize)> = Vec::new();
        let mut inner_passes: Vec<(bool, usize, bool, usize)> = Vec::new();
        for layer in 0..n {
            let lo = layer;
            let hi = n - 1 - layer;
            if lo > hi {
                break;
            }
            if lo == hi {
                if layer == 0 {
                    outer_passes.push((true, lo, true, layer));
                    outer_passes.push((false, lo, true, layer));
                } else {
                    inner_passes.push((true, lo, true, layer));
                    inner_passes.push((false, lo, true, layer));
                }
                break;
            }
            let group = [
                (true, lo, true, layer),
                (false, hi, true, layer),
                (true, hi, false, layer),
                (false, lo, false, layer),
            ];
            if layer == 0 {
                outer_passes.extend_from_slice(&group);
            } else {
                inner_passes.extend_from_slice(&group);
            }
        }

        let mut outer = build_ring(cfg, &outer_passes, true, 0.0);
        let mut inner = build_ring(cfg, &inner_passes, false, -cfg.crossing_height);

        rotate_path(&mut outer, cfg.rotate_deg);
        rotate_path(&mut inner, cfg.rotate_deg);

        if svg {
            write_svg_paths(&[(&outer, false), (&inner, false)], cfg.name);
        }

        let mut verts: Vec<[f32; 3]> = Vec::new();
        let mut norms: Vec<[f32; 3]> = Vec::new();
        let mut faces: Vec<[usize; 3]> = Vec::new();

        sweep_tube(
            &outer,
            cfg.tube_sides,
            cfg.strand_radius,
            true,
            &mut verts,
            &mut norms,
            &mut faces,
        );
        sweep_tube(
            &inner,
            cfg.tube_sides,
            cfg.strand_radius,
            true,
            &mut verts,
            &mut norms,
            &mut faces,
        );

        let dump = format!("/Users/adduser/meshdump/{}_{}v.obj", cfg.name, verts.len());
        if let Ok(mut file) = File::create(&dump) {
            write_obj(&mut file, cfg.name, &verts, &norms, &faces);
            println!(
                "  {} ({}x{}) -> {} + {} pts, {} verts, {} tris",
                cfg.name,
                cfg.grid_w,
                cfg.grid_h,
                outer.len(),
                inner.len(),
                verts.len(),
                faces.len()
            );
            return Some(dump);
        }
    } else {
        let mut path = build_serpentine_path(cfg);

        if cfg.rotate_deg.abs() > 0.001 {
            rotate_path(&mut path, cfg.rotate_deg);
        }

        if svg {
            write_svg(&path, cfg.name);
        }

        let mut verts: Vec<[f32; 3]> = Vec::new();
        let mut norms: Vec<[f32; 3]> = Vec::new();
        let mut faces: Vec<[usize; 3]> = Vec::new();

        sweep_tube(
            &path,
            cfg.tube_sides,
            cfg.strand_radius,
            true,
            &mut verts,
            &mut norms,
            &mut faces,
        );

        let dump = format!("/Users/adduser/meshdump/{}_{}v.obj", cfg.name, verts.len());
        if let Ok(mut file) = File::create(&dump) {
            write_obj(&mut file, cfg.name, &verts, &norms, &faces);
            println!(
                "  {} ({}x{}) -> {} pts, {} verts, {} tris",
                cfg.name,
                cfg.grid_w,
                cfg.grid_h,
                path.len(),
                verts.len(),
                faces.len()
            );
            return Some(dump);
        }
    }
    None
}

fn generate_charm(charm_name: &str, cfg: &PanChangConfig) -> Option<String> {
    let (main_path, inner_path) = build_knot_paths(cfg);

    let mut verts = Vec::new();
    let mut norms = Vec::new();
    let mut faces = Vec::new();
    let cord_r = cfg.strand_radius;
    let d = 2.0 * cord_r;
    let ts = cfg.tube_sides.max(4);

    // Sweep inner ring unchanged (two-strand knots only)
    if let Some(ref inner) = inner_path {
        sweep_tube(inner, ts, cord_r, true, &mut verts, &mut norms, &mut faces);
    }

    // Find top ear tip (max y) and bottom ear tip (min y) on the main path.
    // After 45° rotation these are at x≈0 with horizontal tangent.
    let n = main_path.len();
    let top_ix = (0..n)
        .max_by(|&a, &b| main_path[a].y.partial_cmp(&main_path[b].y).unwrap())
        .unwrap();
    let bot_ix = (0..n)
        .min_by(|&a, &b| main_path[a].y.partial_cmp(&main_path[b].y).unwrap())
        .unwrap();

    let top_pt = main_path[top_ix];
    let bot_pt = main_path[bot_ix];
    let k = top_pt.y - bot_pt.y;

    // Traditional proportions
    let button_h = 3.5 * d;
    let cord_gap = 2.0 * d;
    let cord_gap_short = 1.5 * d;
    let loop_height = 0.15 * k;
    let tassel_len = 1.2 * k;
    let tassel_body_r = 0.10 * k;
    let wrap_r = tassel_body_r + d * 0.5;
    let wrap_h = 2.5 * d;
    let btn_scale = 1.75 * d / 3.0;
    let offset = 1.5 * cord_r;
    let transition_h = 3.0 * d;
    let kappa = 0.5517f32;

    let cord_samples = 8usize;
    let btn_samples = 48usize;
    let loop_samples = 24usize;
    let gather_samples = 8usize;
    let transition_samples = 16usize;

    // ======== SPLIT KNOT PATH AT EAR TIPS ========
    // The main_path is a closed loop. We split it at top_ix and bot_ix into
    // two segments that each traverse half the knot body.
    //
    // Segment A: top_ix → ... → bot_ix (following original path direction)
    //   At top_ix: tangent ≈ (-1, 0) = LEFT
    //   At bot_ix: tangent ≈ (+1, 0) = RIGHT
    //
    // Segment B: bot_ix → ... → top_ix (following original path direction, wrapping)
    //   At bot_ix: tangent ≈ (+1, 0) = RIGHT
    //   At top_ix: tangent ≈ (-1, 0) = LEFT
    //
    // For the charm we use reversed copies:
    //   Seg B reversed: top→bot, starts heading RIGHT, ends heading LEFT
    //   Seg A reversed: bot→top, starts heading LEFT, ends heading RIGHT
    let seg_a: Vec<V3> = if top_ix <= bot_ix {
        main_path[top_ix..=bot_ix].to_vec()
    } else {
        let mut s = main_path[top_ix..].to_vec();
        s.extend_from_slice(&main_path[..=bot_ix]);
        s
    };
    let seg_b: Vec<V3> = if bot_ix <= top_ix {
        main_path[bot_ix..=top_ix].to_vec()
    } else {
        let mut s = main_path[bot_ix..].to_vec();
        s.extend_from_slice(&main_path[..=top_ix]);
        s
    };
    let seg_b_rev: Vec<V3> = seg_b.iter().rev().cloned().collect();
    let seg_a_rev: Vec<V3> = seg_a.iter().rev().cloned().collect();

    // ======== BUILD SINGLE CONTINUOUS CHARM PATH (closed loop) ========
    //
    // The ENTIRE charm — loop, cords, buttons, knot body, gathering, tassel
    // region — is ONE continuous closed path. No separate overlapping tubes.
    //
    //   LOOP APEX → right cord DOWN → right top button DOWN →
    //   transition → Seg B reversed (top→bot) → transition →
    //   left cord DOWN → left bottom button DOWN →
    //   gathering arc →
    //   right cord UP → right bottom button UP →
    //   transition → Seg A reversed (bot→top) → transition →
    //   left cord UP → left top button UP → LOOP APEX
    //
    let top_y = top_pt.y;
    let bot_y = bot_pt.y;

    // Upper Y layout
    let y_up_trans = top_y + transition_h;
    let y_up_btn_lo = y_up_trans + cord_gap;
    let y_up_btn_hi = y_up_btn_lo + button_h;
    let y_up_loop = y_up_btn_hi + cord_gap_short;

    // Lower Y layout
    let y_dn_trans = bot_y - transition_h;
    let y_dn_btn_hi = y_dn_trans - cord_gap;
    let y_dn_btn_lo = y_dn_btn_hi - button_h;
    let y_dn_gather = y_dn_btn_lo - cord_gap_short;

    let mut charm: Vec<V3> = Vec::with_capacity(n + 1024);

    // === 1. Loop arch: left → top → right ===
    for i in 0..loop_samples {
        let t = i as f32 / loop_samples as f32;
        let angle = PI * (1.0 - t); // π → 0 (left → top → right)
        charm.push(v3(offset * angle.cos(), y_up_loop + loop_height * angle.sin(), 0.0));
    }

    // === 2. Right cord DOWN from loop to button ===
    for i in 0..=cord_samples {
        let t = i as f32 / cord_samples as f32;
        charm.push(v3(offset, y_up_loop - (y_up_loop - y_up_btn_hi) * t, 0.0));
    }

    // === 3. Right top button DOWN ===
    for i in 1..=btn_samples {
        let t = 2.0 * PI * i as f32 / btn_samples as f32;
        let progress = i as f32 / btn_samples as f32;
        let window = (PI * progress).sin();
        charm.push(v3(
            offset + btn_scale * (t.sin() + 2.0 * (2.0 * t).sin()) * window,
            y_up_btn_hi - (y_up_btn_hi - y_up_btn_lo) * progress,
            btn_scale * ((3.0 * t).sin()) * window,
        ));
    }

    // === 4. Right cord DOWN from button to transition top ===
    for i in 1..=cord_samples {
        let t = i as f32 / cord_samples as f32;
        charm.push(v3(offset, y_up_btn_lo - (y_up_btn_lo - y_up_trans) * t, 0.0));
    }

    // === 5. Transition: (offset, y_up_trans) vert-down → top_pt heading RIGHT ===
    // Seg B reversed starts at top_pt heading RIGHT
    bezier(
        &mut charm,
        v3(offset, y_up_trans, 0.0),
        v3(offset, top_y + transition_h * (1.0 - kappa), 0.0),
        v3(top_pt.x + kappa * offset, top_pt.y, top_pt.z),
        top_pt,
        transition_samples,
    );

    // === 6. Seg B reversed: top→bot (starts heading RIGHT, ends heading LEFT) ===
    // Skip first point (already placed by transition bezier endpoint vicinity)
    for i in 1..seg_b_rev.len() {
        charm.push(seg_b_rev[i]);
    }

    // === 7. Transition: bot_pt heading LEFT → (-offset, y_dn_trans) vert-down ===
    bezier(
        &mut charm,
        bot_pt,
        v3(bot_pt.x - kappa * offset, bot_pt.y, bot_pt.z),
        v3(-offset, bot_y - transition_h * (1.0 - kappa), 0.0),
        v3(-offset, y_dn_trans, 0.0),
        transition_samples,
    );

    // === 8. Left cord DOWN from transition to button ===
    for i in 0..=cord_samples {
        let t = i as f32 / cord_samples as f32;
        charm.push(v3(-offset, y_dn_trans - (y_dn_trans - y_dn_btn_hi) * t, 0.0));
    }

    // === 9. Left bottom button DOWN ===
    for i in 1..=btn_samples {
        let t = 2.0 * PI * i as f32 / btn_samples as f32;
        let progress = i as f32 / btn_samples as f32;
        let window = (PI * progress).sin();
        charm.push(v3(
            -offset + btn_scale * (t.sin() + 2.0 * (2.0 * t).sin()) * window,
            y_dn_btn_hi - (y_dn_btn_hi - y_dn_btn_lo) * progress,
            btn_scale * (-(3.0 * t).sin()) * window,
        ));
    }

    // === 10. Left cord DOWN to gathering ===
    for i in 1..=cord_samples {
        let t = i as f32 / cord_samples as f32;
        charm.push(v3(-offset, y_dn_btn_lo - (y_dn_btn_lo - y_dn_gather) * t, 0.0));
    }

    // === 11. Gathering arc (束口): left → bottom → right ===
    for i in 1..=gather_samples {
        let t = i as f32 / gather_samples as f32;
        let angle = PI * (1.0 - t); // π → 0 (left → bottom → right)
        charm.push(v3(offset * angle.cos(), y_dn_gather - offset * angle.sin(), 0.0));
    }

    // === 12. Right cord UP from gathering to button ===
    for i in 1..=cord_samples {
        let t = i as f32 / cord_samples as f32;
        charm.push(v3(offset, y_dn_gather + (y_dn_btn_lo - y_dn_gather) * t, 0.0));
    }

    // === 13. Right bottom button UP (opposite z) ===
    for i in 1..=btn_samples {
        let t = 2.0 * PI * i as f32 / btn_samples as f32;
        let progress = i as f32 / btn_samples as f32;
        let window = (PI * progress).sin();
        charm.push(v3(
            offset + btn_scale * (t.sin() + 2.0 * (2.0 * t).sin()) * window,
            y_dn_btn_lo + (y_dn_btn_hi - y_dn_btn_lo) * progress,
            btn_scale * ((3.0 * t).sin()) * window,
        ));
    }

    // === 14. Right cord UP from button to transition bottom ===
    for i in 1..=cord_samples {
        let t = i as f32 / cord_samples as f32;
        charm.push(v3(offset, y_dn_btn_hi + (y_dn_trans - y_dn_btn_hi) * t, 0.0));
    }

    // === 15. Transition: (offset, y_dn_trans) vert-up → bot_pt heading LEFT ===
    // Seg A reversed starts at bot_pt heading LEFT
    bezier(
        &mut charm,
        v3(offset, y_dn_trans, 0.0),
        v3(offset, bot_y - transition_h * (1.0 - kappa), 0.0),
        v3(bot_pt.x - kappa * offset, bot_pt.y, bot_pt.z),
        bot_pt,
        transition_samples,
    );

    // === 16. Seg A reversed: bot→top (starts heading LEFT, ends heading RIGHT) ===
    for i in 1..seg_a_rev.len() {
        charm.push(seg_a_rev[i]);
    }

    // === 17. Transition: top_pt heading RIGHT → (-offset, y_up_trans) vert-up ===
    bezier(
        &mut charm,
        top_pt,
        v3(top_pt.x + kappa * offset, top_pt.y, top_pt.z),
        v3(-offset, top_y + transition_h * (1.0 - kappa), 0.0),
        v3(-offset, y_up_trans, 0.0),
        transition_samples,
    );

    // === 18. Left cord UP from transition to button ===
    for i in 0..=cord_samples {
        let t = i as f32 / cord_samples as f32;
        charm.push(v3(-offset, y_up_trans + (y_up_btn_lo - y_up_trans) * t, 0.0));
    }

    // === 19. Left top button UP (opposite z) ===
    for i in 1..=btn_samples {
        let t = 2.0 * PI * i as f32 / btn_samples as f32;
        let progress = i as f32 / btn_samples as f32;
        let window = (PI * progress).sin();
        charm.push(v3(
            -offset + btn_scale * (t.sin() + 2.0 * (2.0 * t).sin()) * window,
            y_up_btn_lo + (y_up_btn_hi - y_up_btn_lo) * progress,
            btn_scale * (-(3.0 * t).sin()) * window,
        ));
    }

    // === 20. Left cord UP to loop base ===
    for i in 1..=cord_samples {
        let t = i as f32 / cord_samples as f32;
        charm.push(v3(-offset, y_up_btn_hi + (y_up_loop - y_up_btn_hi) * t, 0.0));
    }

    // (Loop arch at step 1 closes the path)

    // Sweep the entire charm as ONE closed tube — truly continuous
    let knot_verts = verts.len();
    sweep_tube(&charm, ts, cord_r, true, &mut verts, &mut norms, &mut faces);

    // ======== TASSEL (simplified — fringe only) ========
    let tassel_top_y = y_dn_gather - offset;
    let tassel_center_y = tassel_top_y - tassel_len * 0.5;
    gen_cylinder(
        0.0,
        tassel_center_y,
        0.0,
        tassel_body_r,
        tassel_len,
        ts,
        &mut verts,
        &mut norms,
        &mut faces,
    );
    let wrap_y = tassel_top_y - 0.08 * tassel_len;
    gen_cylinder(0.0, wrap_y, 0.0, wrap_r, wrap_h, ts, &mut verts, &mut norms, &mut faces);

    let fringe_bot_y = tassel_top_y - tassel_len;
    let charm_top = y_up_loop + loop_height;
    let total_h = charm_top - fringe_bot_y;

    let dump = format!("/Users/adduser/meshdump/{}_{}v.obj", charm_name, verts.len());
    if let Ok(mut file) = File::create(&dump) {
        write_obj(&mut file, charm_name, &verts, &norms, &faces);
        println!(
            "  {} -> inner: {} verts, charm strand: {} pts, total: {} verts, {} tris, h: {:.2} ({:.1}K)",
            charm_name,
            knot_verts,
            charm.len(),
            verts.len(),
            faces.len(),
            total_h,
            total_h / k,
        );
        return Some(dump);
    }
    None
}

fn leak(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

fn main() {
    let _ = std::fs::create_dir_all("/Users/adduser/meshdump");

    let default_sef = 1.0 + std::f32::consts::FRAC_1_SQRT_2;

    let specs: &[(&str, usize, f32, f32, f32, f32, f32, f32)] = &[
        ("panchang_erhui", 3, 1.00, 0.65, 0.10, 0.35, default_sef, 1.0),
        ("panchang_sanhui", 4, 0.80, 0.52, 0.10, 0.30, 0.35, 0.5),
        ("panchang_sihui", 5, 0.65, 0.42, 0.10, 0.26, default_sef, 1.0),
        ("panchang_wuhui", 6, 0.54, 0.35, 0.10, 0.22, 0.35, 0.5),
    ];

    let res_levels: &[(f32, &str)] = &[(0.2, "ultra"), (0.4, "low"), (0.7, "med")];

    println!("=== Pan Chang Knots ===\n");
    for &(name_base, grid_n, gs, lo, sr, ch, sef, ilf) in specs {
        for (ri, &(res, _label)) in res_levels.iter().enumerate() {
            let cfg = PanChangConfig {
                name: leak(name_base.to_string()),
                grid_w: grid_n,
                grid_h: grid_n,
                grid_spacing: gs,
                lobe_offset: lo,
                strand_radius: sr,
                crossing_height: ch,
                seg_samples: (20.0 * res).max(3.0) as usize,
                arc_samples: (16.0 * res).max(3.0) as usize,
                corner_samples: (10.0 * res).max(3.0) as usize,
                tube_sides: (10.0 * res).max(3.0) as usize,
                rotate_deg: 45.0,
                corner_ears: true,
                side_ear_factor: sef,
                inner_lobe_factor: ilf,
            };
            let obj_path = generate_knot(&cfg, ri == res_levels.len() - 1);

            // Generate GLB for ultra-low poly only (first resolution level)
            if ri == 0 {
                if let Some(ref obj) = obj_path {
                    generate_glb(obj, name_base);
                }
            }
        }
    }

    // ======== Charm assemblies ========
    println!("\n=== Charm Assemblies ===\n");

    let charm_specs: &[(&str, usize, f32, f32, f32, f32, f32, f32)] = &[
        // (charm_name, grid_n, gs, lo, sr, ch, sef, ilf)
        ("charm_sanhui", 4, 0.80, 0.52, 0.10, 0.30, 0.35, 0.5),
        ("charm_sihui", 5, 0.65, 0.42, 0.10, 0.26, default_sef, 1.0),
    ];

    let charm_res: f32 = 0.4; // low poly for charm GLBs
    for &(charm_name, grid_n, gs, lo, sr, ch, sef, ilf) in charm_specs {
        let cfg = PanChangConfig {
            name: leak(charm_name.to_string()),
            grid_w: grid_n,
            grid_h: grid_n,
            grid_spacing: gs,
            lobe_offset: lo,
            strand_radius: sr,
            crossing_height: ch,
            seg_samples: (20.0 * charm_res).max(3.0) as usize,
            arc_samples: (16.0 * charm_res).max(3.0) as usize,
            corner_samples: (10.0 * charm_res).max(3.0) as usize,
            tube_sides: (10.0 * charm_res).max(3.0) as usize,
            rotate_deg: 45.0,
            corner_ears: true,
            side_ear_factor: sef,
            inner_lobe_factor: ilf,
        };
        if let Some(ref obj) = generate_charm(charm_name, &cfg) {
            generate_glb(obj, charm_name);
        }
    }

    println!("\n=== done ===");
}
