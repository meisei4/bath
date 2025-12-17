use std::collections::{HashSet, VecDeque};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

const TILE_SIZE: f32 = 6.0; // pixels per tile

#[derive(Clone, Copy, Debug)]
struct Door {
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

fn interior_hit(interior: &HashSet<(i32, i32)>, x: f32, z: f32) -> Option<(i32, i32)> {
    let ix = floor_i(x);
    let iz = floor_i(z);
    if interior.contains(&(ix, iz)) {
        Some((ix, iz))
    } else {
        None
    }
}

fn bake_door_on_perimeter(door: &mut Door, interior: &HashSet<(i32, i32)>) {
    let mut set_baked_normal = |d: &mut Door| {
        let dx = d.x1 - d.x0;
        let dz = d.z1 - d.z0;
        let len = (dx * dx + dz * dz).sqrt();
        if len < 1e-6 {
            d.nx = 0.0;
            d.nz = 0.0;
            return false;
        }
        let tx = dx / len;
        let tz = dz / len;
        d.nx = tz;
        d.nz = -tx;
        true
    };

    if !set_baked_normal(door) {
        return;
    }

    let mut dx = door.x1 - door.x0;
    let mut dz = door.z1 - door.z0;
    let mut len = (dx * dx + dz * dz).sqrt();
    if len < 1e-6 {
        door.nx = 0.0;
        door.nz = 0.0;
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
            let px = door.x0 + (door.x1 - door.x0) * s;
            let pz = door.z0 + (door.z1 - door.z0) * s;

            for (j, d) in DISTS.iter().enumerate() {
                let sx = px + nx * sign * (*d);
                let sz = pz + nz * sign * (*d);
                if interior_hit(interior, sx, sz).is_some() {
                    score += (DISTS.len() - j) as i32;
                    break;
                }
            }
        }
        score
    };

    let score_plus = score_side(1.0);
    let score_minus = score_side(-1.0);

    if score_minus > score_plus {
        std::mem::swap(&mut door.x0, &mut door.x1);
        std::mem::swap(&mut door.z0, &mut door.z1);

        dx = door.x1 - door.x0;
        dz = door.z1 - door.z0;
        len = (dx * dx + dz * dz).sqrt();
        if len < 1e-6 {
            door.nx = 0.0;
            door.nz = 0.0;
            return;
        }
        tx = dx / len;
        tz = dz / len;
        nx = tz;
        nz = -tx;
    }

    let midx = 0.5 * (door.x0 + door.x1);
    let midz = 0.5 * (door.z0 + door.z1);

    let mut interior_tile: Option<(i32, i32)> = None;
    for d in DISTS {
        if let Some(t) = interior_hit(interior, midx + nx * d, midz + nz * d) {
            interior_tile = Some(t);
            break;
        }
    }
    let Some((ix_in, iz_in)) = interior_tile else {
        set_baked_normal(door);
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
            set_baked_normal(door);
            return;
        }
        let mut ix = ix_in;
        while interior.contains(&(ix + step_x, iz_in)) {
            ix += step_x;
        }
        let x_boundary = if step_x == 1 { (ix + 1) as f32 } else { ix as f32 };
        door.x0 = x_boundary;
        door.x1 = x_boundary;
    } else {
        if step_z == 0 {
            set_baked_normal(door);
            return;
        }
        let mut iz = iz_in;
        while interior.contains(&(ix_in, iz + step_z)) {
            iz += step_z;
        }
        let z_boundary = if step_z == 1 { (iz + 1) as f32 } else { iz as f32 };
        door.z0 = z_boundary;
        door.z1 = z_boundary;
    }

    set_baked_normal(door);
}

fn main() {
    let input_path = asset_payload::FLOORPLAN_PATH;
    let output_path = "/home/adduser/fu4seoi3/src/fu4seoi3/romdisk/assets/room_v0.txt";

    let file = File::open(input_path).expect("Failed to open input");
    let mut lines = BufReader::new(file).lines();

    let first = lines.next().unwrap().unwrap();
    let dims: Vec<i32> = first.split_whitespace().map(|s| s.parse().unwrap()).collect();
    let (px_w, px_h) = (dims[0], dims[1]);

    let n: usize = lines.next().unwrap().unwrap().trim().parse().unwrap();

    let mut walls = Vec::new();
    for _ in 0..n {
        let line = lines.next().unwrap().unwrap();
        let p: Vec<f32> = line.split_whitespace().take(4).map(|s| s.parse().unwrap()).collect();
        walls.push((p[0], p[1], p[2], p[3]));
    }

    let mut doors_px = Vec::new();
    for line in lines.flatten() {
        let toks: Vec<&str> = line.split_whitespace().collect();
        if toks.len() >= 5 && toks[4].eq_ignore_ascii_case("door") {
            let x0: f32 = toks[0].parse().unwrap();
            let y0: f32 = toks[1].parse().unwrap();
            let x1: f32 = toks[2].parse().unwrap();
            let y1: f32 = toks[3].parse().unwrap();
            doors_px.push((x0, y0, x1, y1));

            walls.push((x0, y0, x1, y1));
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
            if x >= 0 && x < gw && y >= 0 && y < gh {
                is_wall[y as usize][x as usize] = true;
            }
        }
    }

    let mut exterior = vec![vec![false; gw as usize]; gh as usize];
    let mut queue = VecDeque::new();

    for y in 0..gh {
        for x in 0..gw {
            if (x == 0 || x == gw - 1 || y == 0 || y == gh - 1) && !is_wall[y as usize][x as usize] {
                queue.push_back((x, y));
                exterior[y as usize][x as usize] = true;
            }
        }
    }

    while let Some((x, y)) = queue.pop_front() {
        for (dx, dy) in [(0, 1), (1, 0), (0, -1), (-1, 0)] {
            let nx = x + dx;
            let ny = y + dy;
            if nx >= 0 && nx < gw && ny >= 0 && ny < gh {
                let (ux, uy) = (nx as usize, ny as usize);
                if !is_wall[uy][ux] && !exterior[uy][ux] {
                    exterior[uy][ux] = true;
                    queue.push_back((nx, ny));
                }
            }
        }
    }

    let mut final_tiles = Vec::new();
    for y in 0..gh {
        for x in 0..gw {
            let (ux, uy) = (x as usize, y as usize);
            if !is_wall[uy][ux] && !exterior[uy][ux] {
                final_tiles.push((x, y));
            }
        }
    }

    if final_tiles.is_empty() {
        final_tiles.push((0, 0));
    }

    let ix_min = final_tiles.iter().map(|(x, _)| x).min().unwrap() - 1;
    let ix_max = final_tiles.iter().map(|(x, _)| x).max().unwrap() + 1;
    let iz_min = final_tiles.iter().map(|(_, z)| z).min().unwrap() - 1;
    let iz_max = final_tiles.iter().map(|(_, z)| z).max().unwrap() + 1;

    let w = ix_max - ix_min + 1;
    let d = iz_max - iz_min + 1;

    let mut interior_set: HashSet<(i32, i32)> = HashSet::new();
    for &(ix, iz) in &final_tiles {
        interior_set.insert((ix - ix_min, iz - iz_min));
    }

    let mut baked_doors: Vec<Door> = Vec::new();
    for &(x0, y0, x1, y1) in &doors_px {
        let mut door = Door {
            x0: x0 / TILE_SIZE - ix_min as f32,
            z0: y0 / TILE_SIZE - iz_min as f32,
            x1: x1 / TILE_SIZE - ix_min as f32,
            z1: y1 / TILE_SIZE - iz_min as f32,
            nx: 0.0,
            nz: 0.0,
        };

        bake_door_on_perimeter(&mut door, &interior_set);
        baked_doors.push(door);
    }

    let mut out = File::create(output_path).expect("Failed to create output");
    writeln!(out, "ROOM_V0").unwrap();
    writeln!(out, "SEED {} 3 {}", w, d).unwrap();
    writeln!(out, "ORIGIN CENTERED").unwrap();

    for dd in &baked_doors {
        writeln!(
            out,
            "DOOR {:.6} {:.6} {:.6} {:.6} {:.6} {:.6}",
            dd.x0, dd.z0, dd.x1, dd.z1, dd.nx, dd.nz
        )
        .unwrap();
    }

    for &(ix, iz) in &final_tiles {
        writeln!(out, "FLOOR {} {}", ix - ix_min, iz - iz_min).unwrap();
    }

    writeln!(out, "END").unwrap();

    println!(
        "Wrote {} floor tiles + {} doors to {} (SEED {}x{}x3)",
        final_tiles.len(),
        baked_doors.len(),
        output_path,
        w,
        d
    );
}
