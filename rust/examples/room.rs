use std::collections::{HashSet, VecDeque};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

const TILE_SIZE: f32 = 6.0; // pixels per tile

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

    let mut out = File::create(output_path).expect("Failed to create output");
    writeln!(out, "ROOM_V0").unwrap();
    writeln!(out, "SEED {} 3 {}", w, d).unwrap();
    writeln!(out, "ORIGIN CENTERED").unwrap();

    for (ix, iz) in &final_tiles {
        writeln!(out, "FLOOR {} {}", ix, iz).unwrap();
    }

    writeln!(out, "END").unwrap();

    println!("Wrote {} floor tiles to {}", final_tiles.len(), output_path);
}
