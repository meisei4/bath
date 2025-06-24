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

pub fn generate_convex_collision_polygons_pixel_perfect(
    pixel_mask_slice: &[u8],
    (w, h): (usize, usize),
    tile_edge_length: usize,
) -> Vec<Vec<Vector2>> {
    let contours = generate_concave_collision_polygons_pixel_perfect(pixel_mask_slice, (w, h), tile_edge_length);
    let mut hulls = Vec::with_capacity(contours.len());
    for contour in contours {
        let hull = compute_convex_hull_monotone_chain(&contour);
        if hull.len() >= 3 {
            hulls.push(hull);
        }
    }
    hulls
}

fn compute_convex_hull_monotone_chain(boundary_points_slice: &[Vector2]) -> Vec<Vector2> {
    if boundary_points_slice.len() < 3 {
        return boundary_points_slice.to_vec();
    }
    let mut sorted_points = boundary_points_slice.to_vec();
    sort_points_by_x_then_y(&mut sorted_points);
    let lower_hull = build_lower_monotone_chain_hull(&sorted_points);
    let upper_hull = build_upper_monotone_chain_hull(&sorted_points);
    merge_monotone_chain_hulls(lower_hull, upper_hull)
}

fn sort_points_by_x_then_y(points: &mut [Vector2]) {
    points.sort_by(|point_a, point_b| {
        point_a
            .x
            .partial_cmp(&point_b.x)
            .unwrap()
            .then(point_a.y.partial_cmp(&point_b.y).unwrap())
    });
}

fn build_lower_monotone_chain_hull(sorted_points: &[Vector2]) -> Vec<Vector2> {
    let mut lower_chain = Vec::with_capacity(sorted_points.len());
    for &candidate_point in sorted_points {
        while lower_chain.len() >= 2
            && compute_cross_product_z(
                &lower_chain[lower_chain.len() - 2],
                &lower_chain[lower_chain.len() - 1],
                &candidate_point,
            ) <= 0.0
        {
            lower_chain.pop();
        }
        lower_chain.push(candidate_point);
    }
    lower_chain
}

fn build_upper_monotone_chain_hull(sorted_points: &[Vector2]) -> Vec<Vector2> {
    let mut upper_chain = Vec::with_capacity(sorted_points.len());
    for &candidate_point in sorted_points.iter().rev() {
        while upper_chain.len() >= 2
            && compute_cross_product_z(
                &upper_chain[upper_chain.len() - 2],
                &upper_chain[upper_chain.len() - 1],
                &candidate_point,
            ) <= 0.0
        {
            upper_chain.pop();
        }
        upper_chain.push(candidate_point);
    }
    upper_chain
}

fn merge_monotone_chain_hulls(mut lower_chain: Vec<Vector2>, mut upper_chain: Vec<Vector2>) -> Vec<Vector2> {
    lower_chain.pop();
    upper_chain.pop();
    lower_chain.extend(upper_chain);
    lower_chain
}

fn compute_cross_product_z(point_a: &Vector2, point_b: &Vector2, point_c: &Vector2) -> f32 {
    let delta_x_ab = point_b.x - point_a.x;
    let delta_y_ab = point_b.y - point_a.y;
    let delta_x_bc = point_c.x - point_a.x;
    let delta_y_bc = point_c.y - point_a.y;
    delta_x_ab * delta_y_bc - delta_y_ab * delta_x_bc
}
