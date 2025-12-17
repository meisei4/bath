use std::collections::{HashSet, VecDeque};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

const TILE_SIZE: f32 = 6.0;

#[derive(Clone, Copy)]
struct Opening {
    x0: f32,
    z0: f32,
    x1: f32,
    z1: f32,
    nx: f32,
    nz: f32,
}

fn floor_i(v: f32) -> i32 {
    v.floor() as i32
}

fn interior_hit(interior: &HashSet<(i32, i32)>, x: f32, z: f32) -> bool {
    interior.contains(&(floor_i(x), floor_i(z)))
}

fn bake_on_perimeter(o: &mut Opening, interior: &HashSet<(i32, i32)>) {
    let mut set_normal = |oo: &mut Opening| -> bool {
        let dx = oo.x1 - oo.x0;
        let dz = oo.z1 - oo.z0;
        let len = (dx * dx + dz * dz).sqrt();
        if len < 1e-6 {
            oo.nx = 0.0;
            oo.nz = 0.0;
            return false;
        }
        let tx = dx / len;
        let tz = dz / len;
        oo.nx = tz;
        oo.nz = -tx;
        true
    };

    if !set_normal(o) {
        return;
    }

    let mut dx = o.x1 - o.x0;
    let mut dz = o.z1 - o.z0;
    let mut len = (dx * dx + dz * dz).sqrt();
    if len < 1e-6 {
        o.nx = 0.0;
        o.nz = 0.0;
        return;
    }

    let mut tx = dx / len;
    let mut tz = dz / len;
    let mut nx = tz;
    let mut nz = -tx;

    const DISTS: [f32; 8] = [0.25, 0.50, 0.75, 1.00, 1.25, 1.50, 1.75, 2.00];
    const SAMPLES: [f32; 3] = [0.20, 0.50, 0.80];

    let score_side = |sign: f32| -> i32 {
        let mut score = 0i32;
        for s in SAMPLES {
            let px = o.x0 + (o.x1 - o.x0) * s;
            let pz = o.z0 + (o.z1 - o.z0) * s;
            for (j, d) in DISTS.iter().enumerate() {
                let sx = px + nx * sign * (*d);
                let sz = pz + nz * sign * (*d);
                if interior_hit(interior, sx, sz) {
                    score += (DISTS.len() - j) as i32;
                    break;
                }
            }
        }
        score
    };

    if score_side(-1.0) > score_side(1.0) {
        std::mem::swap(&mut o.x0, &mut o.x1);
        std::mem::swap(&mut o.z0, &mut o.z1);

        dx = o.x1 - o.x0;
        dz = o.z1 - o.z0;
        len = (dx * dx + dz * dz).sqrt();
        if len < 1e-6 {
            o.nx = 0.0;
            o.nz = 0.0;
            return;
        }
        tx = dx / len;
        tz = dz / len;
        nx = tz;
        nz = -tx;
    }

    let midx = 0.5 * (o.x0 + o.x1);
    let midz = 0.5 * (o.z0 + o.z1);

    let mut interior_tile = None;
    for d in DISTS {
        let sx = midx + nx * d;
        let sz = midz + nz * d;
        let ix = floor_i(sx);
        let iz = floor_i(sz);
        if interior.contains(&(ix, iz)) {
            interior_tile = Some((ix, iz));
            break;
        }
    }
    let Some((ix_in, iz_in)) = interior_tile else {
        set_normal(o);
        return;
    };

    let step_x: i32 = if nx > 0.0 {
        -1
    } else if nx < 0.0 {
        1
    } else {
        0
    };
    let step_z: i32 = if nz > 0.0 {
        -1
    } else if nz < 0.0 {
        1
    } else {
        0
    };

    let vertical = dx.abs() < dz.abs();

    if vertical {
        if step_x == 0 {
            set_normal(o);
            return;
        }
        let mut ix = ix_in;
        while interior.contains(&(ix + step_x, iz_in)) {
            ix += step_x;
        }
        let x_boundary = if step_x == 1 { (ix + 1) as f32 } else { ix as f32 };
        o.x0 = x_boundary;
        o.x1 = x_boundary;
    } else {
        if step_z == 0 {
            set_normal(o);
            return;
        }
        let mut iz = iz_in;
        while interior.contains(&(ix_in, iz + step_z)) {
            iz += step_z;
        }
        let z_boundary = if step_z == 1 { (iz + 1) as f32 } else { iz as f32 };
        o.z0 = z_boundary;
        o.z1 = z_boundary;
    }

    set_normal(o);
}

fn split_into_islands(tiles: &[(i32, i32)]) -> Vec<HashSet<(i32, i32)>> {
    let mut all: HashSet<(i32, i32)> = tiles.iter().copied().collect();
    let mut out = Vec::new();

    while let Some(&start) = all.iter().next() {
        let mut q = VecDeque::new();
        let mut comp = HashSet::new();
        all.remove(&start);
        q.push_back(start);

        while let Some((x, z)) = q.pop_front() {
            comp.insert((x, z));
            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let n = (x + dx, z + dz);
                if all.remove(&n) {
                    q.push_back(n);
                }
            }
        }

        out.push(comp);
    }

    out
}

fn seg_mid(x0: f32, y0: f32, x1: f32, y1: f32) -> (f32, f32) {
    (0.5 * (x0 + x1), 0.5 * (y0 + y1))
}

fn main() {
    let input_path = asset_payload::FLOORPLAN_PATH;
    let output_path = "/home/adduser/fu4seoi3/src/fu4seoi3/romdisk/assets/room_v0.txt";

    let file = File::open(input_path).unwrap();
    let mut lines = BufReader::new(file).lines();

    let dims: Vec<i32> = lines
        .next()
        .unwrap()
        .unwrap()
        .split_whitespace()
        .map(|s| s.parse().unwrap())
        .collect();
    let (px_w, px_h) = (dims[0], dims[1]);

    let n: usize = lines.next().unwrap().unwrap().trim().parse().unwrap();

    let mut walls: Vec<(f32, f32, f32, f32)> = Vec::with_capacity(n);
    for _ in 0..n {
        let p: Vec<f32> = lines
            .next()
            .unwrap()
            .unwrap()
            .split_whitespace()
            .take(4)
            .map(|s| s.parse().unwrap())
            .collect();
        walls.push((p[0], p[1], p[2], p[3]));
    }

    let mut doors_px: Vec<(f32, f32, f32, f32)> = Vec::new();
    let mut entrance_px: Option<(f32, f32, f32, f32)> = None;

    for line in lines.flatten() {
        let toks: Vec<&str> = line.split_whitespace().collect();
        if toks.len() < 5 {
            continue;
        }
        let x0: f32 = toks[0].parse().unwrap();
        let y0: f32 = toks[1].parse().unwrap();
        let x1: f32 = toks[2].parse().unwrap();
        let y1: f32 = toks[3].parse().unwrap();

        if toks[4].eq_ignore_ascii_case("door") {
            doors_px.push((x0, y0, x1, y1));
            walls.push((x0, y0, x1, y1));
        } else if toks[4].eq_ignore_ascii_case("entrance") {
            if entrance_px.is_none() {
                entrance_px = Some((x0, y0, x1, y1));
            }
        }
    }

    let gw = (px_w as f32 / TILE_SIZE).ceil() as i32;
    let gh = (px_h as f32 / TILE_SIZE).ceil() as i32;

    let mut is_wall = vec![vec![false; gw as usize]; gh as usize];
    for &(x0, y0, x1, y1) in &walls {
        let ix0 = (x0 / TILE_SIZE) as i32;
        let iy0 = (y0 / TILE_SIZE) as i32;
        let ix1 = (x1 / TILE_SIZE) as i32;
        let iy1 = (y1 / TILE_SIZE) as i32;

        let dx = ix1 - ix0;
        let dy = iy1 - iy0;
        let steps = dx.abs().max(dy.abs()).max(1);

        for step in 0..=steps {
            let t = step as f32 / steps as f32;
            let x = (ix0 as f32 + t * dx as f32).round() as i32;
            let y = (iy0 as f32 + t * dy as f32).round() as i32;
            if (0..gw).contains(&x) && (0..gh).contains(&y) {
                is_wall[y as usize][x as usize] = true;
            }
        }
    }

    let mut exterior = vec![vec![false; gw as usize]; gh as usize];
    let mut q = VecDeque::new();
    for y in 0..gh {
        for x in 0..gw {
            if (x == 0 || x == gw - 1 || y == 0 || y == gh - 1) && !is_wall[y as usize][x as usize] {
                exterior[y as usize][x as usize] = true;
                q.push_back((x, y));
            }
        }
    }

    while let Some((x, y)) = q.pop_front() {
        for (dx, dy) in [(0, 1), (1, 0), (0, -1), (-1, 0)] {
            let nx = x + dx;
            let ny = y + dy;
            if (0..gw).contains(&nx) && (0..gh).contains(&ny) {
                let (ux, uy) = (nx as usize, ny as usize);
                if !is_wall[uy][ux] && !exterior[uy][ux] {
                    exterior[uy][ux] = true;
                    q.push_back((nx, ny));
                }
            }
        }
    }

    let mut final_tiles: Vec<(i32, i32)> = Vec::new();
    for y in 0..gh {
        for x in 0..gw {
            if !is_wall[y as usize][x as usize] && !exterior[y as usize][x as usize] {
                final_tiles.push((x, y));
            }
        }
    }
    if final_tiles.is_empty() {
        final_tiles.push((0, 0));
    }

    let ix_min = final_tiles.iter().map(|(x, _)| *x).min().unwrap() - 1;
    let ix_max = final_tiles.iter().map(|(x, _)| *x).max().unwrap() + 1;
    let iz_min = final_tiles.iter().map(|(_, z)| *z).min().unwrap() - 1;
    let iz_max = final_tiles.iter().map(|(_, z)| *z).max().unwrap() + 1;

    let w = ix_max - ix_min + 1;
    let d = iz_max - iz_min + 1;

    let islands_global = split_into_islands(&final_tiles);

    let mut islands_local: Vec<HashSet<(i32, i32)>> = Vec::with_capacity(islands_global.len());
    for comp in &islands_global {
        let mut set = HashSet::with_capacity(comp.len());
        for &(gx, gz) in comp {
            set.insert((gx - ix_min, gz - iz_min));
        }
        islands_local.push(set);
    }

    let (entr_cx, entr_cy) = entrance_px
        .map(|(x0, y0, x1, y1)| seg_mid(x0, y0, x1, y1))
        .unwrap_or((0.0, 0.0));

    let entr_tile_global = (floor_i(entr_cx / TILE_SIZE), floor_i(entr_cy / TILE_SIZE));
    let entr_tile_local = (entr_tile_global.0 - ix_min, entr_tile_global.1 - iz_min);

    let mut entrance_island = 0usize;
    for (i, set) in islands_local.iter().enumerate() {
        if set.contains(&entr_tile_local) {
            entrance_island = i;
            break;
        }
    }

    let mut primary_did = 0i32;
    if !doors_px.is_empty() {
        let mut best = (0usize, f32::INFINITY);
        for (i, &(x0, y0, x1, y1)) in doors_px.iter().enumerate() {
            let (mx, my) = seg_mid(x0, y0, x1, y1);
            let dx = mx - entr_cx;
            let dy = my - entr_cy;
            let dist2 = dx * dx + dy * dy;
            if dist2 < best.1 {
                best = (i, dist2);
            }
        }
        primary_did = best.0 as i32;
    }

    let mut baked: Vec<(i32, Opening, bool)> = Vec::new();

    for (did, &(x0, y0, x1, y1)) in doors_px.iter().enumerate() {
        let did = did as i32;

        let base = Opening {
            x0: x0 / TILE_SIZE - ix_min as f32,
            z0: y0 / TILE_SIZE - iz_min as f32,
            x1: x1 / TILE_SIZE - ix_min as f32,
            z1: y1 / TILE_SIZE - iz_min as f32,
            nx: 0.0,
            nz: 0.0,
        };

        for (island_i, interior_set) in islands_local.iter().enumerate() {
            let mut o = base;
            bake_on_perimeter(&mut o, interior_set);

            let midx = 0.5 * (o.x0 + o.x1);
            let midz = 0.5 * (o.z0 + o.z1);
            let eps = 0.25;

            if interior_hit(interior_set, midx + o.nx * eps, midz + o.nz * eps) {
                let is_primary = did == primary_did && island_i == entrance_island;
                baked.push((did, o, is_primary));
            }
        }
    }

    baked.sort_by(|a, b| b.2.cmp(&a.2).then_with(|| a.0.cmp(&b.0)));

    let mut out = File::create(output_path).unwrap();
    writeln!(out, "ROOM_V0").unwrap();
    writeln!(out, "SEED {} 3 {}", w, d).unwrap();
    writeln!(out, "ORIGIN CENTERED").unwrap();

    for (did, o, _) in &baked {
        writeln!(
            out,
            "DOOR {:.6} {:.6} {:.6} {:.6} {:.6} {:.6} {}",
            o.x0, o.z0, o.x1, o.z1, o.nx, o.nz, did
        )
        .unwrap();
    }

    for &(ix, iz) in &final_tiles {
        writeln!(out, "FLOOR {} {}", ix - ix_min, iz - iz_min).unwrap();
    }

    writeln!(out, "END").unwrap();
}
