use raylib::camera::Camera3D;
use raylib::color::Color;
use raylib::consts::{CameraProjection, KeyboardKey, TraceLogLevel};
use raylib::drawing::{RaylibDraw, RaylibMode3DExt};
use raylib::ffi;
use raylib::init;
use raylib::math::Vector3;
use std::f32::consts::PI;

const PATH_SAMPLES: usize = 128;
const TUBE_SIDES: usize = 3;
const MORPH_SPEED: f32 = 0.8;
const SCREEN_WIDTH: i32 = 800;
const SCREEN_HEIGHT: i32 = 600;
const FONT_SIZE: i32 = 20;

const SUNFLOWER: Color = Color::new(255, 204, 153, 255);
const ANAKIWA: Color = Color::new(153, 204, 255, 255);
const MARINER: Color = Color::new(51, 102, 204, 255);
const NEON_CARROT: Color = Color::new(255, 153, 51, 255);
const CHESTNUT_ROSE: Color = Color::new(204, 102, 102, 255);
const LILAC: Color = Color::new(204, 153, 204, 255);

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
fn v3_len(a: V3) -> f32 {
    (a.x * a.x + a.y * a.y + a.z * a.z).sqrt()
}
fn v3_lerp(a: V3, b: V3, t: f32) -> V3 {
    v3(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t, a.z + (b.z - a.z) * t)
}

fn v3_norm(a: V3) -> V3 {
    let l = v3_len(a);
    if l < 1e-8 {
        return v3(0.0, 0.0, 1.0);
    }
    v3_scale(a, 1.0 / l)
}

struct PanChangConfig {
    grid_w: usize,
    grid_h: usize,
    grid_spacing: f32,
    lobe_offset: f32,
    crossing_height: f32,
    seg_samples: usize,
    arc_samples: usize,
    corner_samples: usize,
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
        if (row % 2) == 1 {
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
                seg(
                    &mut p,
                    col_x(gw - k),
                    y_cur,
                    z_h(gw - k, row),
                    col_x(gw - k - 1),
                    y_cur,
                    z_h(gw - k - 1, row),
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
        if (ci % 2) == 1 {
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

fn rotate_path(path: &mut [V3], deg: f32) {
    let a = deg * PI / 180.0;
    let (ca, sa) = (a.cos(), a.sin());
    for pt in path.iter_mut() {
        let (x, y) = (pt.x, pt.y);
        pt.x = x * ca - y * sa;
        pt.y = x * sa + y * ca;
    }
}

fn resample_arc_length(path: &[V3], target_count: usize) -> Vec<V3> {
    let n = path.len();
    if n < 2 {
        return vec![path[0]; target_count];
    }
    let mut cum = Vec::with_capacity(n + 1);
    cum.push(0.0f32);
    for i in 0..n {
        cum.push(cum[i] + v3_len(v3_sub(path[(i + 1) % n], path[i])));
    }
    let total = cum[n];
    let mut result = Vec::with_capacity(target_count);
    let mut si = 0usize;
    for k in 0..target_count {
        let s = total * k as f32 / target_count as f32;
        while si < n - 1 && cum[si + 1] < s {
            si += 1;
        }
        let seg_start = cum[si];
        let seg_len = cum[si + 1] - seg_start;
        let t = if seg_len > 1e-8 { (s - seg_start) / seg_len } else { 0.0 };
        result.push(v3_lerp(path[si], path[(si + 1) % n], t));
    }
    result
}

fn sweep_tube(path: &[V3], tube_sides: usize, strand_radius: f32) -> (Vec<V3>, Vec<V3>) {
    let n = path.len();
    let m = tube_sides;
    let cr = strand_radius;
    let mut positions = Vec::with_capacity(n * m);
    let mut norms_out = Vec::with_capacity(n * m);
    for i in 0..n {
        let t = v3_norm(v3_sub(path[(i + 1) % n], path[(i + n - 1) % n]));
        let ref_dir = if t.z.abs() < 0.9 {
            v3(0.0, 0.0, 1.0)
        } else {
            v3(1.0, 0.0, 0.0)
        };
        let normal = v3_norm(v3_sub(ref_dir, v3_scale(t, v3_dot(ref_dir, t))));
        let binormal = v3_norm(v3_cross(normal, t));
        for j in 0..m {
            let angle = 2.0 * PI * j as f32 / m as f32;
            let off = v3_add(v3_scale(normal, angle.cos() * cr), v3_scale(binormal, angle.sin() * cr));
            positions.push(v3_add(path[i], off));
            norms_out.push(v3_norm(off));
        }
    }
    (positions, norms_out)
}

fn sweep_tube_open(path: &[V3], tube_sides: usize, strand_radius: f32) -> (Vec<V3>, Vec<V3>) {
    let n = path.len();
    if n < 2 {
        return (vec![], vec![]);
    }
    let m = tube_sides;
    let cr = strand_radius;
    let mut positions = Vec::with_capacity(n * m);
    let mut norms_out = Vec::with_capacity(n * m);
    for i in 0..n {
        let t = if i == 0 {
            v3_norm(v3_sub(path[1], path[0]))
        } else if i == n - 1 {
            v3_norm(v3_sub(path[n - 1], path[n - 2]))
        } else {
            v3_norm(v3_sub(path[i + 1], path[i - 1]))
        };
        let ref_dir = if t.z.abs() < 0.9 {
            v3(0.0, 0.0, 1.0)
        } else {
            v3(1.0, 0.0, 0.0)
        };
        let normal = v3_norm(v3_sub(ref_dir, v3_scale(t, v3_dot(ref_dir, t))));
        let binormal = v3_norm(v3_cross(normal, t));
        for j in 0..m {
            let angle = 2.0 * PI * j as f32 / m as f32;
            let off = v3_add(v3_scale(normal, angle.cos() * cr), v3_scale(binormal, angle.sin() * cr));
            positions.push(v3_add(path[i], off));
            norms_out.push(v3_norm(off));
        }
    }
    (positions, norms_out)
}

#[allow(dead_code)]
struct Strand {
    path: Vec<V3>,
    positions: Vec<V3>,
    normals: Vec<V3>,
}

struct KnotData {
    name: &'static str,
    strand_radius: f32,
    strands: Vec<Strand>,
}

fn generate_knot(
    name: &'static str,
    grid_n: usize,
    gs: f32,
    lo: f32,
    sr: f32,
    ch: f32,
    sef: f32,
    ilf: f32,
) -> KnotData {
    let cfg = PanChangConfig {
        grid_w: grid_n,
        grid_h: grid_n,
        grid_spacing: gs,
        lobe_offset: lo,
        crossing_height: ch,
        seg_samples: 4,
        arc_samples: 3,
        corner_samples: 3,
        rotate_deg: 45.0,
        corner_ears: true,
        side_ear_factor: sef,
        inner_lobe_factor: ilf,
    };

    if grid_n % 2 == 0 {
        let mut outer_passes = Vec::new();
        let mut inner_passes = Vec::new();
        for layer in 0..grid_n {
            let lo_idx = layer;
            let hi_idx = grid_n - 1 - layer;
            if lo_idx > hi_idx {
                break;
            }
            if lo_idx == hi_idx {
                let target = if layer == 0 {
                    &mut outer_passes
                } else {
                    &mut inner_passes
                };
                target.push((true, lo_idx, true, layer));
                target.push((false, lo_idx, true, layer));
                break;
            }
            let group = [
                (true, lo_idx, true, layer),
                (false, hi_idx, true, layer),
                (true, hi_idx, false, layer),
                (false, lo_idx, false, layer),
            ];
            if layer == 0 {
                outer_passes.extend_from_slice(&group);
            } else {
                inner_passes.extend_from_slice(&group);
            }
        }
        let mut outer_raw = build_ring(&cfg, &outer_passes, true, 0.0);
        let mut inner_raw = build_ring(&cfg, &inner_passes, false, -cfg.crossing_height);
        rotate_path(&mut outer_raw, cfg.rotate_deg);
        rotate_path(&mut inner_raw, cfg.rotate_deg);
        let outer_path = resample_arc_length(&outer_raw, PATH_SAMPLES);
        let inner_path = resample_arc_length(&inner_raw, PATH_SAMPLES);
        let (op, on) = sweep_tube(&outer_path, TUBE_SIDES, sr);
        let (ip, in_) = sweep_tube(&inner_path, TUBE_SIDES, sr);
        KnotData {
            name,
            strand_radius: sr,
            strands: vec![
                Strand {
                    path: outer_path,
                    positions: op,
                    normals: on,
                },
                Strand {
                    path: inner_path,
                    positions: ip,
                    normals: in_,
                },
            ],
        }
    } else {
        let mut raw = build_serpentine_path(&cfg);
        rotate_path(&mut raw, cfg.rotate_deg);
        let path = resample_arc_length(&raw, PATH_SAMPLES);
        let (pos, nrm) = sweep_tube(&path, TUBE_SIDES, sr);
        KnotData {
            name,
            strand_radius: sr,
            strands: vec![Strand {
                path,
                positions: pos,
                normals: nrm,
            }],
        }
    }
}

fn generate_all_knots() -> Vec<KnotData> {
    let default_sef = 1.0 + std::f32::consts::FRAC_1_SQRT_2;
    vec![
        generate_knot("erhui  3x3", 3, 1.00, 0.65, 0.10, 0.35, default_sef, 1.0),
        generate_knot("sanhui 4x4", 4, 0.80, 0.52, 0.10, 0.30, 0.35, 0.5),
        generate_knot("sihui  5x5", 5, 0.65, 0.42, 0.10, 0.26, default_sef, 1.0),
        generate_knot("wuhui  6x6", 6, 0.54, 0.35, 0.10, 0.22, 0.35, 0.5),
    ]
}

#[derive(Clone, Copy, PartialEq)]
enum MorphMode {
    PathLerp,
    CrossingFlatten,
    KnotTie,
}

impl MorphMode {
    fn next(&self) -> MorphMode {
        match self {
            MorphMode::PathLerp => MorphMode::CrossingFlatten,
            MorphMode::CrossingFlatten => MorphMode::KnotTie,
            MorphMode::KnotTie => MorphMode::PathLerp,
        }
    }
}

fn morph_strand_count(src: &KnotData, dst: &KnotData) -> usize {
    src.strands.len().max(dst.strands.len())
}

fn find_best_rotation(src: &[V3], dst: &[V3]) -> (usize, bool) {
    let n = src.len();
    let mut best_shift = 0usize;
    let mut best_reverse = false;
    let mut best_cost = f32::MAX;
    for s in 0..n {
        let mut cost = 0.0f32;
        for k in 0..n {
            let d = v3_sub(src[k], dst[(k + s) % n]);
            cost += v3_dot(d, d);
            if cost >= best_cost {
                break;
            }
        }
        if cost < best_cost {
            best_cost = cost;
            best_shift = s;
            best_reverse = false;
        }
    }
    for s in 0..n {
        let mut cost = 0.0f32;
        for k in 0..n {
            let d = v3_sub(src[k], dst[(n + s - k) % n]);
            cost += v3_dot(d, d);
            if cost >= best_cost {
                break;
            }
        }
        if cost < best_cost {
            best_cost = cost;
            best_shift = s;
            best_reverse = true;
        }
    }
    (best_shift, best_reverse)
}

fn apply_rotation(path: &[V3], shift: usize, reverse: bool) -> Vec<V3> {
    let n = path.len();
    (0..n)
        .map(|k| {
            if reverse {
                path[(n + shift - k) % n]
            } else {
                path[(k + shift) % n]
            }
        })
        .collect()
}

struct DisplayStrand {
    positions: Vec<V3>,
    n_rings: usize,
    closed: bool,
}

struct Display {
    strands: Vec<DisplayStrand>,
}

impl Display {
    fn from_knot(k: &KnotData) -> Self {
        Self {
            strands: k
                .strands
                .iter()
                .map(|s| DisplayStrand {
                    positions: s.positions.clone(),
                    n_rings: s.path.len(),
                    closed: true,
                })
                .collect(),
        }
    }
}

fn morph_path_lerp(src: &KnotData, dst: &KnotData, t: f32) -> Display {
    let sc = morph_strand_count(src, dst);
    let radius = src.strand_radius;
    let mut strands = Vec::new();
    for i in 0..sc {
        let si = i.min(src.strands.len() - 1);
        let di = i.min(dst.strands.len() - 1);
        let (shift, rev) = find_best_rotation(&src.strands[si].path, &dst.strands[di].path);
        let dst_path = apply_rotation(&dst.strands[di].path, shift, rev);
        let blended: Vec<V3> = (0..PATH_SAMPLES)
            .map(|k| v3_lerp(src.strands[si].path[k], dst_path[k], t))
            .collect();
        let (pos, _) = sweep_tube(&blended, TUBE_SIDES, radius);
        strands.push(DisplayStrand {
            positions: pos,
            n_rings: PATH_SAMPLES,
            closed: true,
        });
    }
    Display { strands }
}

fn make_relaxed_rope(path: &[V3]) -> Vec<V3> {
    let n = path.len();
    let mut cx = 0.0f32;
    let mut cy = 0.0f32;
    for p in path {
        cx += p.x;
        cy += p.y;
    }
    cx /= n as f32;
    cy /= n as f32;
    let mut total_len = 0.0f32;
    for i in 1..n {
        total_len += v3_len(v3_sub(path[i], path[i - 1]));
    }
    total_len += v3_len(v3_sub(path[0], path[n - 1]));
    let half = total_len * 0.45;
    (0..n)
        .map(|i| {
            let t = i as f32 / (n - 1) as f32;
            v3(cx - half + 2.0 * half * t, cy, 0.0)
        })
        .collect()
}

fn find_crossing_regions(path: &[V3]) -> Vec<(usize, usize)> {
    let n = path.len();
    let max_z = path.iter().map(|p| p.z.abs()).fold(0.0f32, f32::max);
    if max_z < 1e-6 {
        return vec![];
    }
    let threshold = max_z * 0.15;
    let mut regions = Vec::new();
    let mut in_region = false;
    let mut start = 0;
    for i in 0..n {
        if path[i].z.abs() > threshold {
            if !in_region {
                start = i;
                in_region = true;
            }
        } else if in_region {
            regions.push((start, i));
            in_region = false;
        }
    }
    if in_region {
        regions.push((start, n));
    }
    regions
}

fn crossing_z_scale(k: usize, crossings: &[(usize, usize)], progress: f32, reverse: bool) -> f32 {
    if crossings.is_empty() {
        return 1.0;
    }
    let nc = crossings.len();
    for (ci, &(start, end)) in crossings.iter().enumerate() {
        if k >= start && k < end {
            let order = if reverse { nc - 1 - ci } else { ci };
            let t_start = order as f32 / nc as f32;
            let t_end = (order + 1) as f32 / nc as f32;
            if progress <= t_start {
                return 1.0;
            }
            if progress >= t_end {
                return 0.0;
            }
            return 1.0 - ((progress - t_start) / (t_end - t_start)).clamp(0.0, 1.0);
        }
    }
    1.0
}

fn morph_knot_tie(src: &KnotData, dst: &KnotData, t: f32) -> Display {
    let sc = morph_strand_count(src, dst);
    let mut strands = Vec::new();

    for i in 0..sc {
        let si = i.min(src.strands.len() - 1);
        let di = i.min(dst.strands.len() - 1);
        let sp = &src.strands[si].path;
        let dp = &dst.strands[di].path;
        let n = PATH_SAMPLES;
        let src_rope = make_relaxed_rope(sp);
        let dst_rope = make_relaxed_rope(dp);
        let src_cx = find_crossing_regions(sp);
        let dst_cx = find_crossing_regions(dp);

        let path: Vec<V3> = (0..n)
            .map(|k| {
                if t < 0.20 {
                    let p = t / 0.20;
                    let zs = crossing_z_scale(k, &src_cx, p, true);
                    v3(sp[k].x, sp[k].y, sp[k].z * zs)
                } else if t < 0.35 {
                    let lt = (t - 0.20) / 0.15;
                    v3_lerp(v3(sp[k].x, sp[k].y, 0.0), src_rope[k], lt)
                } else if t < 0.45 {
                    let lt = (t - 0.35) / 0.10;
                    v3_lerp(src_rope[k], dst_rope[k], lt)
                } else if t < 0.60 {
                    let lt = (t - 0.45) / 0.15;
                    v3_lerp(dst_rope[k], v3(dp[k].x, dp[k].y, 0.0), lt)
                } else if t < 0.92 {
                    let p = (t - 0.60) / 0.32;
                    let zs = 1.0 - crossing_z_scale(k, &dst_cx, p, false);
                    v3(dp[k].x, dp[k].y, dp[k].z * zs)
                } else {
                    dp[k]
                }
            })
            .collect();

        let radius = src.strand_radius;
        let is_open = t >= 0.20 && t < 0.60;
        let (pos, _) = if is_open {
            sweep_tube_open(&path, TUBE_SIDES, radius)
        } else {
            sweep_tube(&path, TUBE_SIDES, radius)
        };
        strands.push(DisplayStrand {
            positions: pos,
            n_rings: n,
            closed: !is_open,
        });
    }
    Display { strands }
}

fn morph_crossing_flatten(src: &KnotData, dst: &KnotData, t: f32) -> Display {
    let sc = morph_strand_count(src, dst);
    let radius = src.strand_radius;
    let mut strands = Vec::new();

    for i in 0..sc {
        let si = i.min(src.strands.len() - 1);
        let di = i.min(dst.strands.len() - 1);
        let (shift, rev) = find_best_rotation(&src.strands[si].path, &dst.strands[di].path);
        let dst_path = apply_rotation(&dst.strands[di].path, shift, rev);
        let path: Vec<V3> = if t < 0.4 {
            let st = t / 0.4;
            let z_scale = 1.0 - st;
            src.strands[si]
                .path
                .iter()
                .map(|p| v3(p.x, p.y, p.z * z_scale))
                .collect()
        } else if t < 0.6 {
            let st = (t - 0.4) / 0.2;
            (0..PATH_SAMPLES)
                .map(|k| {
                    let a = v3(src.strands[si].path[k].x, src.strands[si].path[k].y, 0.0);
                    let b = v3(dst_path[k].x, dst_path[k].y, 0.0);
                    v3_lerp(a, b, st)
                })
                .collect()
        } else {
            let st = (t - 0.6) / 0.4;
            dst_path.iter().map(|p| v3(p.x, p.y, p.z * st)).collect()
        };
        let (pos, _) = sweep_tube(&path, TUBE_SIDES, radius);
        strands.push(DisplayStrand {
            positions: pos,
            n_rings: PATH_SAMPLES,
            closed: true,
        });
    }
    Display { strands }
}

fn compute_bounds(knots: &[KnotData]) -> (V3, f32) {
    let mut min = v3(f32::MAX, f32::MAX, f32::MAX);
    let mut max = v3(f32::MIN, f32::MIN, f32::MIN);
    for k in knots {
        for s in &k.strands {
            for p in &s.positions {
                min.x = min.x.min(p.x);
                min.y = min.y.min(p.y);
                min.z = min.z.min(p.z);
                max.x = max.x.max(p.x);
                max.y = max.y.max(p.y);
                max.z = max.z.max(p.z);
            }
        }
    }
    let center = v3((min.x + max.x) * 0.5, (min.y + max.y) * 0.5, (min.z + max.z) * 0.5);
    let half = (max.x - min.x).max(max.y - min.y).max(max.z - min.z) * 0.5;
    (center, half.max(0.01))
}

fn pos_color(p: V3, c: V3, he: f32) -> (u8, u8, u8) {
    let nx = (p.x - c.x) / he;
    let ny = (p.y - c.y) / he;
    let nz = (p.z - c.z) / he;
    let len = (nx * nx + ny * ny + nz * nz).sqrt().max(0.001);
    (
        (127.5 * (nx / len + 1.0)) as u8,
        (127.5 * (ny / len + 1.0)) as u8,
        (127.5 * (nz / len + 1.0)) as u8,
    )
}

unsafe fn render_strand_solid(
    positions: &[V3],
    n_rings: usize,
    n_sides: usize,
    center: V3,
    half_ext: f32,
    closed: bool,
) {
    let limit = if closed {
        n_rings
    } else {
        if n_rings > 1 {
            n_rings - 1
        } else {
            return;
        }
    };
    ffi::rlBegin(ffi::RL_TRIANGLES as i32);
    for i in 0..limit {
        let i1 = if closed { (i + 1) % n_rings } else { i + 1 };
        for j in 0..n_sides {
            let j1 = (j + 1) % n_sides;
            let (a, b, c, d) = (i * n_sides + j, i * n_sides + j1, i1 * n_sides + j, i1 * n_sides + j1);
            let (pa, pb, pc, pd) = (positions[a], positions[b], positions[c], positions[d]);
            let (ra, ga, ba) = pos_color(pa, center, half_ext);
            let (rc, gc, bc) = pos_color(pc, center, half_ext);
            let (rd, gd, bd) = pos_color(pd, center, half_ext);
            let (rb, gb, bb) = pos_color(pb, center, half_ext);
            ffi::rlColor4ub(ra, ga, ba, 255);
            ffi::rlVertex3f(pa.x, pa.y, pa.z);
            ffi::rlColor4ub(rc, gc, bc, 255);
            ffi::rlVertex3f(pc.x, pc.y, pc.z);
            ffi::rlColor4ub(rd, gd, bd, 255);
            ffi::rlVertex3f(pd.x, pd.y, pd.z);
            ffi::rlColor4ub(ra, ga, ba, 255);
            ffi::rlVertex3f(pa.x, pa.y, pa.z);
            ffi::rlColor4ub(rd, gd, bd, 255);
            ffi::rlVertex3f(pd.x, pd.y, pd.z);
            ffi::rlColor4ub(rb, gb, bb, 255);
            ffi::rlVertex3f(pb.x, pb.y, pb.z);
        }
    }
    ffi::rlEnd();
}

unsafe fn render_strand_points(positions: &[V3], n_rings: usize, n_sides: usize, closed: bool) {
    ffi::rlEnablePointMode();
    ffi::rlDisableBackfaceCulling();
    let limit = if closed {
        n_rings
    } else {
        if n_rings > 1 {
            n_rings - 1
        } else {
            return;
        }
    };
    ffi::rlBegin(ffi::RL_TRIANGLES as i32);
    ffi::rlColor4ub(LILAC.r, LILAC.g, LILAC.b, 255);
    for i in 0..limit {
        let i1 = if closed { (i + 1) % n_rings } else { i + 1 };
        for j in 0..n_sides {
            let j1 = (j + 1) % n_sides;
            let a = positions[i * n_sides + j];
            let b = positions[i1 * n_sides + j];
            let c = positions[i * n_sides + j1];
            let d = positions[i1 * n_sides + j1];
            ffi::rlVertex3f(a.x, a.y, a.z);
            ffi::rlVertex3f(b.x, b.y, b.z);
            ffi::rlVertex3f(c.x, c.y, c.z);
            ffi::rlVertex3f(b.x, b.y, b.z);
            ffi::rlVertex3f(d.x, d.y, d.z);
            ffi::rlVertex3f(c.x, c.y, c.z);
        }
    }
    ffi::rlEnd();
    ffi::rlEnableBackfaceCulling();
    ffi::rlDisablePointMode();
}

unsafe fn render_strand_wire(positions: &[V3], n_rings: usize, n_sides: usize, closed: bool) {
    let limit = if closed {
        n_rings
    } else {
        if n_rings > 1 {
            n_rings - 1
        } else {
            return;
        }
    };
    ffi::rlBegin(ffi::RL_LINES as i32);
    ffi::rlColor4ub(MARINER.r, MARINER.g, MARINER.b, 255);
    for i in 0..limit {
        let i1 = if closed { (i + 1) % n_rings } else { i + 1 };
        for j in 0..n_sides {
            let j1 = (j + 1) % n_sides;
            let a = i * n_sides + j;
            ffi::rlVertex3f(positions[a].x, positions[a].y, positions[a].z);
            ffi::rlVertex3f(
                positions[i1 * n_sides + j].x,
                positions[i1 * n_sides + j].y,
                positions[i1 * n_sides + j].z,
            );
            ffi::rlVertex3f(positions[a].x, positions[a].y, positions[a].z);
            ffi::rlVertex3f(
                positions[i * n_sides + j1].x,
                positions[i * n_sides + j1].y,
                positions[i * n_sides + j1].z,
            );
        }
    }
    ffi::rlEnd();
}

unsafe fn render_display(
    display: &Display,
    center: V3,
    half_ext: f32,
    show_pvc: bool,
    show_texture: bool,
    show_wireframe: bool,
    show_points: bool,
) {
    if show_texture || show_pvc {
        for s in &display.strands {
            render_strand_solid(&s.positions, s.n_rings, TUBE_SIDES, center, half_ext, s.closed);
        }
    }
    if show_wireframe {
        ffi::rlSetLineWidth(2.0);
        for s in &display.strands {
            render_strand_wire(&s.positions, s.n_rings, TUBE_SIDES, s.closed);
        }
    }
    if show_points {
        ffi::rlSetPointSize(4.0);
        for s in &display.strands {
            render_strand_points(&s.positions, s.n_rings, TUBE_SIDES, s.closed);
        }
    }
}

fn main() {
    println!("Generating knot data...");
    let knots = generate_all_knots();
    for k in &knots {
        println!("  {} {}s {}v", k.name, k.strands.len(), k.strands[0].positions.len());
    }

    let (bounds_center, bounds_half) = compute_bounds(&knots);

    let (mut handle, thread) = init()
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("knot_morph")
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();
    handle.set_target_fps(60);

    let mut current_knot: usize = 0;
    let mut morph_mode = MorphMode::PathLerp;
    let mut morph_active = false;
    let mut morph_t: f32 = 0.0;
    let mut morph_source: usize = 0;
    let mut morph_target: usize = 0;
    let mut show_pvc = true;
    let mut show_wireframe = true;
    let mut show_points = false;
    let mut show_texture = false;
    let mut ortho = false;

    let mut display = Display::from_knot(&knots[0]);
    let aspect = SCREEN_WIDTH as f32 / SCREEN_HEIGHT as f32;
    let cam_dist = bounds_half * 4.0;
    let mut camera = Camera3D {
        position: Vector3::new(0.0, 0.0, cam_dist),
        target: Vector3::new(bounds_center.x, bounds_center.y, bounds_center.z),
        up: Vector3::Y,
        fovy: 45.0,
        projection: CameraProjection::CAMERA_PERSPECTIVE,
    };

    while !handle.window_should_close() {
        let dt = handle.get_frame_time();
        let mut trigger_to: Option<usize> = None;

        if handle.is_key_pressed(KeyboardKey::KEY_RIGHT) {
            trigger_to = Some((current_knot + 1) % knots.len());
        }
        if handle.is_key_pressed(KeyboardKey::KEY_LEFT) {
            trigger_to = Some((current_knot + knots.len() - 1) % knots.len());
        }
        if handle.is_key_pressed(KeyboardKey::KEY_ONE) {
            morph_mode = MorphMode::PathLerp;
        }
        if handle.is_key_pressed(KeyboardKey::KEY_TWO) {
            morph_mode = MorphMode::CrossingFlatten;
        }
        if handle.is_key_pressed(KeyboardKey::KEY_THREE) {
            morph_mode = MorphMode::KnotTie;
        }
        if handle.is_key_pressed(KeyboardKey::KEY_SPACE) {
            trigger_to = Some((current_knot + 1) % knots.len());
        }
        if handle.is_key_pressed(KeyboardKey::KEY_TAB) {
            morph_mode = morph_mode.next();
            trigger_to = Some((current_knot + 1) % knots.len());
        }
        if handle.is_key_pressed(KeyboardKey::KEY_C) {
            show_pvc = !show_pvc;
        }
        if handle.is_key_pressed(KeyboardKey::KEY_T) {
            show_texture = !show_texture;
        }
        if handle.is_key_pressed(KeyboardKey::KEY_X) {
            show_wireframe = !show_wireframe;
        }
        if handle.is_key_pressed(KeyboardKey::KEY_P) {
            show_points = !show_points;
        }
        if handle.is_key_pressed(KeyboardKey::KEY_O) {
            ortho = !ortho;
            camera.projection = if ortho {
                CameraProjection::CAMERA_ORTHOGRAPHIC
            } else {
                CameraProjection::CAMERA_PERSPECTIVE
            };
            camera.fovy = if ortho { bounds_half * 2.5 } else { 45.0 };
        }
        if handle.is_key_pressed(KeyboardKey::KEY_R) {
            let dist = if ortho {
                bounds_half * 4.0
            } else {
                bounds_half / (camera.fovy.to_radians() * 0.5).tan() / aspect.min(1.0)
            };
            camera.position = Vector3::new(bounds_center.x, bounds_center.y, bounds_center.z + dist);
        }

        if let Some(target) = trigger_to {
            if morph_active {
                current_knot = morph_target;
            }
            if target != current_knot {
                morph_source = current_knot;
                morph_target = target;
                morph_t = 0.0;
                morph_active = true;
            }
        }

        if morph_active {
            display = match morph_mode {
                MorphMode::PathLerp => morph_path_lerp(&knots[morph_source], &knots[morph_target], morph_t),
                MorphMode::CrossingFlatten => {
                    morph_crossing_flatten(&knots[morph_source], &knots[morph_target], morph_t)
                },
                MorphMode::KnotTie => morph_knot_tie(&knots[morph_source], &knots[morph_target], morph_t),
            };
            let speed = if morph_mode == MorphMode::KnotTie {
                MORPH_SPEED * 0.2
            } else {
                MORPH_SPEED
            };
            morph_t += speed * dt;
            if morph_t >= 1.0 {
                morph_active = false;
                current_knot = morph_target;
                display = match morph_mode {
                    MorphMode::PathLerp => morph_path_lerp(&knots[morph_source], &knots[morph_target], 1.0),
                    MorphMode::CrossingFlatten => {
                        morph_crossing_flatten(&knots[morph_source], &knots[morph_target], 1.0)
                    },
                    MorphMode::KnotTie => morph_knot_tie(&knots[morph_source], &knots[morph_target], 1.0),
                };
            }
        }

        let rot_speed = 2.0 * dt;
        let mut orbit_yaw = 0.0f32;
        let mut orbit_pitch = 0.0f32;
        if handle.is_key_down(KeyboardKey::KEY_A) {
            orbit_yaw -= rot_speed;
        }
        if handle.is_key_down(KeyboardKey::KEY_D) {
            orbit_yaw += rot_speed;
        }
        if handle.is_key_down(KeyboardKey::KEY_W) {
            orbit_pitch -= rot_speed;
        }
        if handle.is_key_down(KeyboardKey::KEY_S) {
            orbit_pitch += rot_speed;
        }
        if orbit_yaw != 0.0 || orbit_pitch != 0.0 {
            let mut offset = Vector3::new(
                camera.position.x - camera.target.x,
                camera.position.y - camera.target.y,
                camera.position.z - camera.target.z,
            );
            let dist = (offset.x * offset.x + offset.y * offset.y + offset.z * offset.z).sqrt();
            if orbit_yaw != 0.0 {
                let cos_y = orbit_yaw.cos();
                let sin_y = orbit_yaw.sin();
                let new_x = offset.x * cos_y + offset.z * sin_y;
                let new_z = -offset.x * sin_y + offset.z * cos_y;
                offset.x = new_x;
                offset.z = new_z;
            }
            if orbit_pitch != 0.0 {
                let horiz = (offset.x * offset.x + offset.z * offset.z).sqrt();
                let mut elev = offset.y.atan2(horiz);
                elev += orbit_pitch;
                elev = elev.clamp(-1.4, 1.4);
                let new_horiz = dist * elev.cos();
                offset.y = dist * elev.sin();
                if horiz > 0.001 {
                    let scale = new_horiz / horiz;
                    offset.x *= scale;
                    offset.z *= scale;
                }
            }
            camera.position.x = camera.target.x + offset.x;
            camera.position.y = camera.target.y + offset.y;
            camera.position.z = camera.target.z + offset.z;
        }
        let wheel = handle.get_mouse_wheel_move();
        if wheel != 0.0 {
            if ortho {
                camera.fovy = (camera.fovy - wheel * 0.2).clamp(0.1, bounds_half * 10.0);
            } else {
                let dir = Vector3::new(
                    camera.position.x - camera.target.x,
                    camera.position.y - camera.target.y,
                    camera.position.z - camera.target.z,
                );
                let len = (dir.x * dir.x + dir.y * dir.y + dir.z * dir.z).sqrt();
                let new_len = (len - wheel * bounds_half * 0.3).clamp(bounds_half * 0.5, bounds_half * 10.0);
                let scale = new_len / len;
                camera.position.x = camera.target.x + dir.x * scale;
                camera.position.y = camera.target.y + dir.y * scale;
                camera.position.z = camera.target.z + dir.z * scale;
            }
        }

        let mut dh = handle.begin_drawing(&thread);
        dh.clear_background(Color::BLACK);

        dh.draw_mode3D(camera, |mut _rl3d| unsafe {
            render_display(
                &display,
                bounds_center,
                bounds_half,
                show_pvc,
                show_texture,
                show_wireframe,
                show_points,
            );
        });

        let knot_label = if morph_active {
            format!("[</>]: {} -> {}", knots[morph_source].name, knots[morph_target].name)
        } else {
            format!("[</>]: {}", knots[current_knot].name)
        };
        dh.draw_text(&knot_label, 12, 12, FONT_SIZE, NEON_CARROT);
        dh.draw_text("TX [ T ]:", 570, 12, FONT_SIZE, SUNFLOWER);
        dh.draw_text(
            if show_texture { "ON" } else { "OFF" },
            740,
            12,
            FONT_SIZE,
            if show_texture { ANAKIWA } else { CHESTNUT_ROSE },
        );
        dh.draw_text("CLR [ C ]:", 570, 38, FONT_SIZE, SUNFLOWER);
        dh.draw_text(
            if show_pvc { "ON" } else { "OFF" },
            740,
            38,
            FONT_SIZE,
            if show_pvc { ANAKIWA } else { CHESTNUT_ROSE },
        );
        dh.draw_text("WR [ X ]:", 12, SCREEN_HEIGHT - 52, FONT_SIZE, SUNFLOWER);
        dh.draw_text(
            if show_wireframe { "ON" } else { "OFF" },
            140,
            SCREEN_HEIGHT - 52,
            FONT_SIZE,
            if show_wireframe { ANAKIWA } else { CHESTNUT_ROSE },
        );
        dh.draw_text("PT [ P ]:", 12, SCREEN_HEIGHT - 26, FONT_SIZE, SUNFLOWER);
        dh.draw_text(
            if show_points { "ON" } else { "OFF" },
            140,
            SCREEN_HEIGHT - 26,
            FONT_SIZE,
            if show_points { ANAKIWA } else { CHESTNUT_ROSE },
        );
        dh.draw_text("ORTHO [ O ]:", 570, SCREEN_HEIGHT - 26, FONT_SIZE, SUNFLOWER);
        dh.draw_text(
            if ortho { "ON" } else { "OFF" },
            740,
            SCREEN_HEIGHT - 26,
            FONT_SIZE,
            if ortho { ANAKIWA } else { CHESTNUT_ROSE },
        );
    }
}
