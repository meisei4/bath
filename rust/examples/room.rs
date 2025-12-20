use asset_payload::FLOORPLAN_PATH;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::io::{BufRead, BufReader};

const TILE: f32 = 6.0;

fn main() {
    let input_path = FLOORPLAN_PATH;
    let output_path = "/home/adduser/fu4seoi3/src/fu4seoi3/romdisk/assets/room_down_v0.txt";

    let file = std::fs::File::open(input_path).unwrap();
    let mut lines = BufReader::new(file).lines();

    let first = lines.next().unwrap().unwrap();
    let dims: Vec<i32> = first.split_whitespace().map(|s| s.parse().unwrap()).collect();
    let (px_w, px_h) = (dims[0], dims[1]);

    let n: usize = lines.next().unwrap().unwrap().parse().unwrap();

    let mut walls = Vec::new();
    for _ in 0..n {
        let line = lines.next().unwrap().unwrap();
        let vals: Vec<f32> = line.split_whitespace().take(4).map(|s| s.parse().unwrap()).collect();
        walls.push((vals[0], vals[1], vals[2], vals[3]));
    }

    let mut doors = Vec::new();
    for line in lines {
        let line = line.unwrap();
        let toks: Vec<&str> = line.split_whitespace().collect();
        if toks.len() >= 5 && toks[4].eq_ignore_ascii_case("door") {
            let vals: Vec<f32> = toks[..4].iter().map(|s| s.parse().unwrap()).collect();
            doors.push((vals[0], vals[1], vals[2], vals[3]));
        }
    }

    let gw = ((px_w as f32) / TILE).ceil() as i32;
    let gh = ((px_h as f32) / TILE).ceil() as i32;

    let mut is_wall = vec![false; (gw * gh) as usize];
    let idx = |x: i32, y: i32| (y * gw + x) as usize;

    for (x0, y0, x1, y1) in &walls {
        let ix0 = (x0 / TILE) as i32;
        let iy0 = (y0 / TILE) as i32;
        let ix1 = (x1 / TILE) as i32;
        let iy1 = (y1 / TILE) as i32;

        let steps = (ix1 - ix0).abs().max((iy1 - iy0).abs()).max(1);
        for step in 0..=steps {
            let t = step as f32 / steps as f32;
            let x = (ix0 as f32 + t * (ix1 - ix0) as f32).round() as i32;
            let y = (iy0 as f32 + t * (iy1 - iy0) as f32).round() as i32;
            if x >= 0 && x < gw && y >= 0 && y < gh {
                is_wall[idx(x, y)] = true;
            }
        }
    }

    let mut exterior = vec![false; (gw * gh) as usize];
    let mut q = VecDeque::new();
    for y in 0..gh {
        for x in 0..gw {
            if (x == 0 || x == gw - 1 || y == 0 || y == gh - 1) && !is_wall[idx(x, y)] {
                exterior[idx(x, y)] = true;
                q.push_back((x, y));
            }
        }
    }

    while let Some((x, y)) = q.pop_front() {
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let nx = x + dx;
            let ny = y + dy;
            if nx >= 0 && nx < gw && ny >= 0 && ny < gh {
                let i = idx(nx, ny);
                if !is_wall[i] && !exterior[i] {
                    exterior[i] = true;
                    q.push_back((nx, ny));
                }
            }
        }
    }

    let mut tiles = Vec::new();
    for y in 0..gh {
        for x in 0..gw {
            if !is_wall[idx(x, y)] && !exterior[idx(x, y)] {
                tiles.push((x, y));
            }
        }
    }
    if tiles.is_empty() {
        tiles.push((0, 0));
    }

    let islands = split_islands(&tiles);

    let min_x = tiles.iter().map(|(x, _)| *x).min().unwrap() - 1;
    let max_x = tiles.iter().map(|(x, _)| *x).max().unwrap() + 1;
    let min_z = tiles.iter().map(|(_, z)| *z).min().unwrap() - 1;
    let max_z = tiles.iter().map(|(_, z)| *z).max().unwrap() + 1;

    let seed_w = max_x - min_x + 1;
    let seed_d = max_z - min_z + 1;

    let mut islands_local: Vec<HashSet<(i32, i32)>> = Vec::new();
    for island in &islands {
        let mut local = HashSet::new();
        for (x, z) in island {
            local.insert((x - min_x, z - min_z));
        }
        islands_local.push(local);
    }

    let mut openings = Vec::new();
    for (door_id, (x0, y0, x1, y1)) in doors.iter().enumerate() {
        let cx0 = x0 / TILE - min_x as f32;
        let cz0 = y0 / TILE - min_z as f32;
        let cx1 = x1 / TILE - min_x as f32;
        let cz1 = y1 / TILE - min_z as f32;

        for interior in &islands_local {
            if let Some(opening) = bake_to_perimeter(cx0, cz0, cx1, cz1, interior) {
                openings.push((opening, door_id));
            }
        }
    }

    let mut rows: HashMap<i32, Vec<i32>> = HashMap::new();
    for (x, z) in &tiles {
        rows.entry(z - min_z).or_default().push(x - min_x);
    }

    let mut floor_runs = Vec::new();
    for (z, xs) in rows.iter_mut() {
        xs.sort_unstable();
        let mut a = xs[0];
        let mut b = xs[0];
        for &x in xs.iter().skip(1) {
            if x == b + 1 {
                b = x;
            } else {
                floor_runs.push((*z, a, b));
                a = x;
                b = x;
            }
        }
        floor_runs.push((*z, a, b));
    }
    floor_runs.sort();

    let mut out = String::new();
    out.push_str("ROOM_V0\n");
    out.push_str(&format!("SEED {} 3 {}\n", seed_w, seed_d));
    out.push_str("ORIGIN CENTERED\n");

    for ((axis, fixed, a0, a1, nx, nz), door_id) in &openings {
        let axis_str = if *axis == 1 { "X" } else { "Z" };
        out.push_str(&format!(
            "OPEN DOOR {} {} {} {} {} NX {} NZ {} EMIT\n",
            axis_str, fixed, a0, a1, door_id, nx, nz
        ));
    }

    let wall_segments = derive_walls(&islands_local, &openings);

    for (wall_id, (axis, fixed, a0, a1, nx, nz)) in wall_segments.iter().enumerate() {
        let axis_str = if *axis == 1 { "X" } else { "Z" };
        out.push_str(&format!(
            "OPEN WALL {} {} {} {} {} NX {} NZ {} SCATTER\n",
            axis_str,
            fixed,
            a0,
            a1,
            1000 + wall_id,
            nx,
            nz
        ));
    }

    for (z, x0, x1) in &floor_runs {
        out.push_str(&format!("FLOOR_RUN {} {} {}\n", z, x0, x1));
    }

    out.push_str("END\n");

    fs::write(output_path, &out).unwrap();
    println!(
        "Wrote {} doors, {} walls to {}",
        openings.len(),
        wall_segments.len(),
        output_path
    );
}

fn split_islands(tiles: &[(i32, i32)]) -> Vec<HashSet<(i32, i32)>> {
    let mut remaining: HashSet<(i32, i32)> = tiles.iter().copied().collect();
    let mut islands = Vec::new();

    while let Some(&start) = remaining.iter().next() {
        let mut island = HashSet::new();
        let mut q = VecDeque::new();
        remaining.remove(&start);
        q.push_back(start);

        while let Some((x, z)) = q.pop_front() {
            island.insert((x, z));
            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let n = (x + dx, z + dz);
                if remaining.remove(&n) {
                    q.push_back(n);
                }
            }
        }
        islands.push(island);
    }
    islands
}

fn bake_to_perimeter(
    cx0: f32,
    cz0: f32,
    cx1: f32,
    cz1: f32,
    interior: &HashSet<(i32, i32)>,
) -> Option<(i32, i32, i32, i32, i32, i32)> {
    let dx = cx1 - cx0;
    let dz = cz1 - cz0;
    let len = (dx * dx + dz * dz).sqrt();
    if len < 1e-6 {
        return None;
    }

    let tx = dx / len;
    let tz = dz / len;
    let mut nx = tz;
    let mut nz = -tx;

    let mut score_pos = 0;
    let mut score_neg = 0;

    for frac in [0.25, 0.5, 0.75] {
        let px = cx0 + dx * frac;
        let pz = cz0 + dz * frac;

        for dist in [0.3, 0.6, 1.0, 1.5] {
            let sx_pos = px + nx * dist;
            let sz_pos = pz + nz * dist;
            let sx_neg = px - nx * dist;
            let sz_neg = pz - nz * dist;

            if interior.contains(&(sx_pos.floor() as i32, sz_pos.floor() as i32)) {
                score_pos += 1;
            }
            if interior.contains(&(sx_neg.floor() as i32, sz_neg.floor() as i32)) {
                score_neg += 1;
            }
        }
    }

    if score_neg > score_pos {
        nx = -nx;
        nz = -nz;
    }

    if score_pos == 0 && score_neg == 0 {
        return None;
    }

    let midx = 0.5 * (cx0 + cx1);
    let midz = 0.5 * (cz0 + cz1);

    let mut interior_tile = None;
    for dist in [0.3, 0.6, 1.0, 1.5, 2.0, 2.5, 3.0] {
        let sx = midx + nx * dist;
        let sz = midz + nz * dist;
        let ix = sx.floor() as i32;
        let iz = sz.floor() as i32;
        if interior.contains(&(ix, iz)) {
            interior_tile = Some((ix, iz));
            break;
        }
    }

    let (ix_in, iz_in) = interior_tile?;

    let vertical = dx.abs() < dz.abs();

    if vertical {
        let step_x = if nx > 0.0 { -1 } else { 1 };
        let mut ix = ix_in;
        while interior.contains(&(ix + step_x, iz_in)) {
            ix += step_x;
        }
        let x_snap = if step_x == 1 { ix + 1 } else { ix };

        let z0 = cz0.min(cz1);
        let z1 = cz0.max(cz1);
        let a0 = (z0 * 2.0).round() as i32;
        let a1 = (z1 * 2.0).round() as i32;

        Some((1, x_snap * 2, a0, a1, nx.round() as i32, 0))
    } else {
        let step_z = if nz > 0.0 { -1 } else { 1 };
        let mut iz = iz_in;
        while interior.contains(&(ix_in, iz + step_z)) {
            iz += step_z;
        }
        let z_snap = if step_z == 1 { iz + 1 } else { iz };

        let x0 = cx0.min(cx1);
        let x1 = cx0.max(cx1);
        let a0 = (x0 * 2.0).round() as i32;
        let a1 = (x1 * 2.0).round() as i32;

        Some((0, z_snap * 2, a0, a1, 0, nz.round() as i32))
    }
}
fn carve_segment(a0: i32, a1: i32, holes: &[(i32, i32)]) -> Vec<(i32, i32)> {
    if holes.is_empty() {
        return vec![(a0, a1)];
    }

    let mut sorted_holes: Vec<(i32, i32)> = holes.iter().map(|&(a, b)| (a.min(b), a.max(b))).collect();
    sorted_holes.sort_by_key(|(a, _)| *a);

    let mut out = Vec::new();
    let mut cur = a0;

    for (hole_a, hole_b) in sorted_holes {
        let hole_a = hole_a.max(a0);
        let hole_b = hole_b.min(a1);

        if hole_b <= cur {
            continue;
        }

        if hole_a > cur {
            out.push((cur, hole_a));
        }

        cur = cur.max(hole_b);
    }

    if cur < a1 {
        out.push((cur, a1));
    }

    out
}
fn derive_walls(
    islands: &[HashSet<(i32, i32)>],
    openings: &[((i32, i32, i32, i32, i32, i32), usize)],
) -> Vec<(i32, i32, i32, i32, i32, i32)> {
    let mut all_interior: HashSet<(i32, i32)> = HashSet::new();
    for island in islands {
        all_interior.extend(island.iter());
    }

    let mut walls = Vec::new();

    for interior in islands {
        let mut edges = Vec::new();

        for (x, z) in interior.iter() {
            if !all_interior.contains(&(x - 1, *z)) {
                edges.push((1, x * 2, z * 2, (z + 1) * 2, 1, 0));
            }
            if !all_interior.contains(&(x + 1, *z)) {
                edges.push((1, (x + 1) * 2, z * 2, (z + 1) * 2, -1, 0));
            }
            if !all_interior.contains(&(*x, z - 1)) {
                edges.push((0, z * 2, x * 2, (x + 1) * 2, 0, 1));
            }
            if !all_interior.contains(&(*x, z + 1)) {
                edges.push((0, (z + 1) * 2, x * 2, (x + 1) * 2, 0, -1));
            }
        }

        edges.sort_by_key(|(axis, fixed, a0, _a1, nx, nz)| (*axis, *fixed, *nx, *nz, *a0));

        let mut merged = Vec::new();
        for edge in edges {
            if let Some(last) = merged.last_mut() {
                let (l_axis, l_fixed, _l_a0, l_a1, l_nx, l_nz): &mut (i32, i32, i32, i32, i32, i32) = last;
                if *l_axis == edge.0 && *l_fixed == edge.1 && *l_nx == edge.4 && *l_nz == edge.5 && *l_a1 == edge.2 {
                    *l_a1 = edge.3;
                    continue;
                }
            }
            merged.push(edge);
        }

        for (axis, fixed, a0, a1, nx, nz) in merged {
            let mut holes = Vec::new();

            for ((o_axis, o_fixed, o_a0, o_a1, _, _), _) in openings {
                if axis == *o_axis && fixed == *o_fixed {
                    holes.push((*o_a0, *o_a1));
                }
            }

            let segments = carve_segment(a0, a1, &holes);

            for (seg_a0, seg_a1) in segments {
                walls.push((axis, fixed, seg_a0, seg_a1, nx, nz));
            }
        }
    }

    walls
}
