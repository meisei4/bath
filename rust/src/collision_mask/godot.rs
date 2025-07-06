use godot::builtin::Vector2;

pub fn generate_concave_collision_polygons_pixel_perfect(
    pixel_mask_slice: &[u8],
    (w, h): (usize, usize),
    _tile_edge_length: usize,
) -> Vec<Vec<Vector2>> {
    let mut polygons = Vec::new();
    let mut visited = vec![false; w * h];
    for y in 0..h {
        for x in 0..w {
            let idx = y * w + x;
            if pixel_mask_slice[idx] == 1 && !visited[idx] && is_boundary_pixel(pixel_mask_slice, w, h, x, y) {
                let raw = trace_boundary(pixel_mask_slice, w, h, x, y, &mut visited);
                let simple = filter_colinear(&raw);
                if simple.len() >= 3 {
                    polygons.push(simple);
                }
            }
        }
    }
    polygons
}

fn filter_colinear(contour: &[Vector2]) -> Vec<Vector2> {
    let n = contour.len();
    if n < 3 {
        return contour.to_vec();
    }
    let eps = 0.001;
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let a = contour[(i + n - 1) % n];
        let b = contour[i];
        let c = contour[(i + 1) % n];
        let cross = (b.x - a.x) * (c.y - b.y) - (b.y - a.y) * (c.x - b.x);
        if cross.abs() > eps {
            out.push(b);
        }
    }
    out
}

fn is_boundary_pixel(mask: &[u8], w: usize, h: usize, x: usize, y: usize) -> bool {
    let neighbors = [
        (x as isize + 1, y as isize),
        (x as isize - 1, y as isize),
        (x as isize, y as isize + 1),
        (x as isize, y as isize - 1),
    ];
    for (nx, ny) in neighbors {
        if nx < 0 || ny < 0 || nx >= w as isize || ny >= h as isize {
            return true;
        }
        if mask[ny as usize * w + nx as usize] == 0 {
            return true;
        }
    }
    false
}

fn trace_boundary(mask: &[u8], w: usize, h: usize, sx: usize, sy: usize, visited: &mut [bool]) -> Vec<Vector2> {
    let mut contour = Vec::new();
    let mut x = sx;
    let mut y = sy;
    let mut dir = 3;
    loop {
        let idx = y * w + x;
        visited[idx] = true;
        contour.push(Vector2::new(x as f32 + 0.5, y as f32 + 0.5));

        let mut found = false;
        for i in 0..4 {
            let nd = (dir + 3 + i) % 4;
            let (dx, dy) = match nd {
                0 => (1isize, 0),
                1 => (0, 1),
                2 => (-1, 0),
                3 => (0, -1),
                _ => unreachable!(),
            };
            let nx = x as isize + dx;
            let ny = y as isize + dy;
            if nx >= 0 && ny >= 0 && (nx as usize) < w && (ny as usize) < h {
                let nidx = ny as usize * w + nx as usize;
                if mask[nidx] == 1 {
                    x = nx as usize;
                    y = ny as usize;
                    dir = nd;
                    found = true;
                    break;
                }
            }
        }
        if !found || (x == sx && y == sy) {
            break;
        }
    }

    contour
}
