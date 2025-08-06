use raylib::ffi::MemAlloc;
use raylib::math::glam::Vec3;
use raylib::models::{Mesh, RaylibMesh, WeakMesh};
use std::collections::HashMap;
use std::f32::consts::PI;
use std::mem::{size_of, zeroed};
use std::ptr::null_mut;
use std::slice::from_raw_parts;

const GORE_COUNT: usize = 8;
const MERIDIAN_BAND_FRACTION: f32 = 0.1;
const PADDING: f32 = 0.0;
const PAGE_WIDTH: f32 = 1.5;
const AUTO_SCALE: bool = true;
const TARGET_MAX_EXTENT: f32 = 2.0;
const RECENTER: bool = true;
const ANGLE_LIMIT: f32 = f32::INFINITY;

//TODO: THIS IS BATSHIT. FIX IT

#[inline]
fn normalize_or_zero(v: Vec3) -> Vec3 {
    let l = v.length();
    if l > 1e-12 {
        v / l
    } else {
        v
    }
}

fn ensure_indices(mesh: &mut WeakMesh) {
    if mesh.indices.is_null() {
        let vc = mesh.vertexCount as usize;
        unsafe {
            let ib = MemAlloc((vc * size_of::<u16>()) as u32) as *mut u16;
            for i in 0..vc {
                *ib.add(i) = i as u16;
            }
            mesh.indices = ib;
            mesh.triangleCount = (vc / 3) as i32;
        }
    }
}

pub fn unfold_sphere_like(mesh: &mut WeakMesh) -> Mesh {
    ensure_indices(mesh);
    let triangle_count = mesh.triangleCount as usize;
    let vertex_count = mesh.vertexCount as usize;
    if triangle_count == 0 {
        return unsafe { zeroed() };
    }
    let pos = unsafe { from_raw_parts(mesh.vertices, vertex_count * 3) };
    let idx = unsafe { from_raw_parts(mesh.indices, triangle_count * 3) };

    let mut phi = Vec::with_capacity(vertex_count);
    let mut theta = Vec::with_capacity(vertex_count);
    for i in 0..vertex_count {
        let x = pos[i * 3];
        let y = pos[i * 3 + 1];
        let z = pos[i * 3 + 2];
        let r = (x * x + y * y + z * z).sqrt().max(1e-9);
        let t = (y / r).clamp(-1.0, 1.0).acos();
        let mut p = z.atan2(x);
        if p < 0.0 {
            p += 2.0 * PI;
        }
        theta.push(t);
        phi.push(p);
    }

    let mut ring = vec![0usize; vertex_count];
    {
        let mut uniq = theta.clone();
        uniq.sort_by(|a, b| a.partial_cmp(b).unwrap());
        uniq.dedup_by(|a, b| (*a - *b).abs() < 1e-5);
        for i in 0..vertex_count {
            for (k, v) in uniq.iter().enumerate() {
                if (theta[i] - v).abs() < 1e-5 {
                    ring[i] = k;
                    break;
                }
            }
        }
    }

    let mut meridians = Vec::with_capacity(GORE_COUNT);
    for k in 0..GORE_COUNT {
        meridians.push(k as f32 * (2.0 * PI / GORE_COUNT as f32));
    }
    let meridian_half = (PI / GORE_COUNT as f32) * MERIDIAN_BAND_FRACTION;

    let max_ring = *ring.iter().max().unwrap();
    let mut ring_min: Vec<Option<usize>> = vec![None; max_ring + 1];
    for v in 0..vertex_count {
        let r = ring[v];
        if let Some(c) = ring_min[r] {
            if phi[v] < phi[c] {
                ring_min[r] = Some(v);
            }
        } else {
            ring_min[r] = Some(v);
        }
    }

    #[inline]
    fn v3(p: &[f32], i: usize) -> Vec3 {
        Vec3::new(p[i * 3], p[i * 3 + 1], p[i * 3 + 2])
    }

    let mut locals = Vec::with_capacity(triangle_count);
    for f in 0..triangle_count {
        let t = &idx[f * 3..f * 3 + 3];
        let (a, b, c) = (t[0] as usize, t[1] as usize, t[2] as usize);
        let pa = v3(pos, a);
        let pb = v3(pos, b);
        let pc = v3(pos, c);
        let e = pb - pa;
        let x = normalize_or_zero(e);
        let n = normalize_or_zero(e.cross(pc - pa));
        let y = n.cross(x);
        let la = [0.0, 0.0];
        let lb = [e.length(), 0.0];
        let rc = pc - pa;
        let lc = [rc.dot(x), rc.dot(y)];
        locals.push([la, lb, lc]);
    }

    let mut edges = Vec::<(usize, usize, u16, u16, f32)>::new();
    {
        let mut owner: HashMap<(u16, u16), usize> = HashMap::new();
        for f in 0..triangle_count {
            let t = &idx[f * 3..f * 3 + 3];
            for e in 0..3 {
                let a = t[e];
                let b = t[(e + 1) % 3];
                let key = if a < b { (a, b) } else { (b, a) };
                if let Some(&o) = owner.get(&key) {
                    let tn = &idx[f * 3..f * 3 + 3];
                    let on = &idx[o * 3..o * 3 + 3];
                    let nf = normalize_or_zero(
                        (v3(pos, tn[1] as usize) - v3(pos, tn[0] as usize))
                            .cross(v3(pos, tn[2] as usize) - v3(pos, tn[0] as usize)),
                    );
                    let no = normalize_or_zero(
                        (v3(pos, on[1] as usize) - v3(pos, on[0] as usize))
                            .cross(v3(pos, on[2] as usize) - v3(pos, on[0] as usize)),
                    );
                    let ang = nf.dot(no).clamp(-1.0, 1.0).acos();
                    edges.push((o, f, key.0, key.1, ang));
                } else {
                    owner.insert(key, f);
                }
            }
        }
    }
    edges.sort_by(|a, b| a.4.partial_cmp(&b.4).unwrap());

    let mut parent = vec![None::<(usize, u16, u16)>; triangle_count];
    let mut root = vec![true; triangle_count];
    let mut dsu_p: Vec<usize> = (0..triangle_count).collect();
    let mut dsu_r: Vec<u8> = vec![0; triangle_count];
    fn dsu_find(p: &mut [usize], x: usize) -> usize {
        if p[x] == x {
            x
        } else {
            let r = dsu_find(p, p[x]);
            p[x] = r;
            r
        }
    }
    fn dsu_union(p: &mut [usize], r: &mut [u8], a: usize, b: usize) {
        let mut a = dsu_find(p, a);
        let mut b = dsu_find(p, b);
        if a == b {
            return;
        }
        if r[a] < r[b] {
            std::mem::swap(&mut a, &mut b);
        }
        p[b] = a;
        if r[a] == r[b] {
            r[a] += 1;
        }
    }

    for (fa, fb, va, vb, ang) in &edges {
        let cut = {
            let a = *va as usize;
            let b = *vb as usize;
            let mut avg = (phi[a] + phi[b]) * 0.5;
            if (phi[a] - phi[b]).abs() > PI {
                if phi[a] > phi[b] {
                    avg = ((phi[a] - 2.0 * PI) + phi[b]) * 0.5;
                } else {
                    avg = (phi[a] + (phi[b] - 2.0 * PI)) * 0.5;
                }
                if avg < 0.0 {
                    avg += 2.0 * PI;
                }
            }
            let mut mer = false;
            for m in &meridians {
                let mut d = (avg - m).abs();
                if d > PI {
                    d = 2.0 * PI - d;
                }
                if d < meridian_half {
                    mer = true;
                    break;
                }
            }
            let same_ring = ring[a] == ring[b];
            let ring_break = same_ring && ring_min[ring[a]].map(|mv| mv == a || mv == b).unwrap_or(false);
            mer || ring_break || *ang > ANGLE_LIMIT
        };
        if cut {
            continue;
        }
        let ra = dsu_find(&mut dsu_p, *fa);
        let rb = dsu_find(&mut dsu_p, *fb);
        if ra == rb {
            continue;
        }
        parent[*fb] = Some((*fa, *va, *vb));
        root[*fb] = false;
        dsu_union(&mut dsu_p, &mut dsu_r, ra, rb);
    }

    let mut children = vec![Vec::<usize>::new(); triangle_count];
    for f in 0..triangle_count {
        if let Some((p, _, _)) = parent[f] {
            children[p].push(f);
        }
    }

    let mut placed = vec![[[0.0; 2]; 3]; triangle_count];
    let mut done = vec![false; triangle_count];
    let mut stack = Vec::new();
    for f in 0..triangle_count {
        if root[f] {
            placed[f] = locals[f];
            done[f] = true;
            stack.push(f);
            while let Some(cur) = stack.pop() {
                for &ch in &children[cur] {
                    if done[ch] {
                        continue;
                    }
                    if let Some((p, va, vb)) = parent[ch] {
                        let pt = &idx[p * 3..p * 3 + 3];
                        let ct = &idx[ch * 3..ch * 3 + 3];
                        let mut pa = [0.0; 2];
                        let mut pb = [0.0; 2];
                        for i in 0..3 {
                            if pt[i] == va {
                                pa = placed[p][i];
                            }
                            if pt[i] == vb {
                                pb = placed[p][i];
                            }
                        }
                        let loc = locals[ch];
                        let mut la = [0.0; 2];
                        let mut lb = [0.0; 2];
                        for i in 0..3 {
                            let v = ct[i];
                            if v == va {
                                la = loc[i];
                            } else if v == vb {
                                lb = loc[i];
                            }
                        }
                        let le = [lb[0] - la[0], lb[1] - la[1]];
                        let ge = [pb[0] - pa[0], pb[1] - pa[1]];
                        let ll = (le[0] * le[0] + le[1] * le[1]).sqrt().max(1e-12);
                        let gl = (ge[0] * ge[0] + ge[1] * ge[1]).sqrt().max(1e-12);
                        let ld = [le[0] / ll, le[1] / ll];
                        let gd = [ge[0] / gl, ge[1] / gl];
                        let c = ld[0] * gd[0] + ld[1] * gd[1];
                        let s = ld[0] * gd[1] - ld[1] * gd[0];
                        let sc = gl / ll;
                        let mut placed_corners = [[0.0; 2]; 3];
                        for i in 0..3 {
                            let cp = loc[i];
                            let x = (cp[0] - la[0]) * sc;
                            let y = (cp[1] - la[1]) * sc;
                            let xr = x * c - y * s;
                            let yr = x * s + y * c;
                            placed_corners[i] = [pa[0] + xr, pa[1] + yr];
                        }
                        placed[ch] = placed_corners;
                        done[ch] = true;
                        stack.push(ch);
                    }
                }
            }
        }
    }

    let mut root_of = vec![usize::MAX; triangle_count];
    for f in 0..triangle_count {
        let mut r = f;
        while !root[r] {
            r = parent[r].unwrap().0;
        }
        root_of[f] = r;
    }

    let mut bounds: HashMap<usize, ([f32; 2], [f32; 2])> = HashMap::new();
    for f in 0..triangle_count {
        let r = root_of[f];
        let e = bounds.entry(r).or_insert(([f32::MAX; 2], [f32::MIN; 2]));
        for c in 0..3 {
            let p = placed[f][c];
            if p[0] < e.0[0] {
                e.0[0] = p[0];
            }
            if p[1] < e.0[1] {
                e.0[1] = p[1];
            }
            if p[0] > e.1[0] {
                e.1[0] = p[0];
            }
            if p[1] > e.1[1] {
                e.1[1] = p[1];
            }
        }
    }

    let mut order: Vec<usize> = bounds.keys().copied().collect();
    order.sort_by(|a, b| {
        let aa = bounds.get(a).unwrap();
        let bb = bounds.get(b).unwrap();
        let a_area = (aa.1[0] - aa.0[0]) * (aa.1[1] - aa.0[1]);
        let b_area = (bb.1[0] - bb.0[0]) * (bb.1[1] - bb.0[1]);
        b_area.partial_cmp(&a_area).unwrap()
    });

    let page_width = if PAGE_WIDTH <= 0.0 {
        let mut total = 0.0;
        for r in &order {
            let (min, max) = bounds[r];
            let w = (max[0] - min[0]) + PADDING;
            let h = (max[1] - min[1]) + PADDING;
            total += w * h;
        }
        total.sqrt().max(1e-3)
    } else {
        PAGE_WIDTH
    };

    let mut offset: HashMap<usize, [f32; 2]> = HashMap::new();
    let mut x = 0.0;
    let mut y = 0.0;
    let mut row_h = 0.0;
    for r in &order {
        let (min, max) = bounds[r];
        let w = max[0] - min[0];
        let h = max[1] - min[1];
        let need = w + PADDING;
        if x > 0.0 && x + need > page_width {
            x = 0.0;
            y += row_h + PADDING;
            row_h = 0.0;
        }
        offset.insert(*r, [x - min[0], y - min[1]]);
        x += need;
        if h > row_h {
            row_h = h;
        }
    }

    let mut out_pos = Vec::<f32>::with_capacity(triangle_count * 9);
    let mut out_idx = Vec::<u16>::with_capacity(triangle_count * 3);
    let mut remap: HashMap<(u32, usize), u16> = HashMap::new();
    for f in 0..triangle_count {
        let r = root_of[f];
        let off = offset[&r];
        let tri = &idx[f * 3..f * 3 + 3];
        for c in 0..3 {
            let v = tri[c] as u32;
            let key = (v, r);
            let id = *remap.entry(key).or_insert_with(|| {
                let p = placed[f][c];
                let id = (out_pos.len() / 3) as u16;
                out_pos.extend_from_slice(&[p[0] + off[0], p[1] + off[1], 0.0]);
                id
            });
            out_idx.push(id);
        }
    }

    if AUTO_SCALE || RECENTER {
        let mut minx = f32::MAX;
        let mut maxx = f32::MIN;
        let mut miny = f32::MAX;
        let mut maxy = f32::MIN;
        for i in 0..(out_pos.len() / 3) {
            let xx = out_pos[i * 3];
            let yy = out_pos[i * 3 + 1];
            if xx < minx {
                minx = xx;
            }
            if xx > maxx {
                maxx = xx;
            }
            if yy < miny {
                miny = yy;
            }
            if yy > maxy {
                maxy = yy;
            }
        }
        let mut s = 1.0;
        if AUTO_SCALE {
            let w = (maxx - minx).max(1e-6);
            let h = (maxy - miny).max(1e-6);
            s = TARGET_MAX_EXTENT / w.max(h);
        }
        let cx = 0.5 * (minx + maxx);
        let cy = 0.5 * (miny + maxy);
        for i in 0..(out_pos.len() / 3) {
            if RECENTER {
                out_pos[i * 3] = (out_pos[i * 3] - cx) * s;
                out_pos[i * 3 + 1] = (out_pos[i * 3 + 1] - cy) * s;
            } else if AUTO_SCALE {
                out_pos[i * 3] *= s;
                out_pos[i * 3 + 1] *= s;
            }
        }
    }

    let mut m: Mesh = unsafe { zeroed() };
    m.vertexCount = (out_pos.len() / 3) as i32;
    m.triangleCount = (out_idx.len() / 3) as i32;
    unsafe {
        m.vertices = MemAlloc((out_pos.len() * size_of::<f32>()) as u32) as *mut f32;
        std::ptr::copy_nonoverlapping(out_pos.as_ptr(), m.vertices, out_pos.len());
        m.indices = MemAlloc((out_idx.len() * size_of::<u16>()) as u32) as *mut u16;
        std::ptr::copy_nonoverlapping(out_idx.as_ptr(), m.indices, out_idx.len());
        m.texcoords = null_mut();
        m.normals = null_mut();
        m.tangents = null_mut();
        m.colors = null_mut();
        m.upload(false);
    }
    m
}
