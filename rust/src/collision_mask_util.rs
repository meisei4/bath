use godot::builtin::Vector2;
use std::collections::VecDeque;

pub fn compute_tile_grid_size(
    image_width_pixels: usize,
    image_height_pixels: usize,
    tile_edge_length: usize,
) -> (usize, usize) {
    let number_of_tile_columns = (image_width_pixels + tile_edge_length - 1) / tile_edge_length;
    let number_of_tile_rows    = (image_height_pixels + tile_edge_length - 1) / tile_edge_length;
    (number_of_tile_columns, number_of_tile_rows)
}

pub fn generate_collision_polygons(
    pixel_mask_slice: &[u8],
    (image_width_pixels, image_height_pixels): (usize, usize),
    (number_of_tile_columns, number_of_tile_rows): (usize, usize),
    tile_edge_length: usize,
) -> Vec<Vec<Vector2>> {
    let solidness_map = create_solidness_map(
        pixel_mask_slice,
        image_width_pixels,
        image_height_pixels,
        number_of_tile_columns,
        number_of_tile_rows,
        tile_edge_length,
    );

    let connected_regions = find_all_connected_regions(
        &solidness_map,
        number_of_tile_columns,
        number_of_tile_rows,
    );

    let mut polygons: Vec<Vec<Vector2>> = Vec::new();
    for region_tiles in connected_regions {
        let contour = trace_region_contour(&region_tiles, tile_edge_length, image_height_pixels);
        let hull    = compute_convex_hull_monotone_chain(&contour);
        if hull.len() >= 3 {
            polygons.push(hull);
        }
    }

    polygons
}

pub fn create_solidness_map(
    pixel_mask_slice: &[u8],
    image_width_pixels: usize,
    image_height_pixels: usize,
    number_of_tile_columns: usize,
    number_of_tile_rows: usize,
    tile_edge_length: usize,
) -> Vec<u8> {
    let total_tiles = number_of_tile_columns * number_of_tile_rows;
    let mut map = vec![0u8; total_tiles];

    for row in 0..number_of_tile_rows {
        for col in 0..number_of_tile_columns {
            if check_if_tile_has_solid_pixel(
                pixel_mask_slice,
                image_width_pixels,
                image_height_pixels,
                col,
                row,
                tile_edge_length,
            ) {
                map[row * number_of_tile_columns + col] = 1;
            }
        }
    }

    map
}

pub fn find_all_connected_regions(
    solidness_map_slice: &[u8],
    number_of_tile_columns: usize,
    number_of_tile_rows: usize,
) -> Vec<Vec<(usize, usize)>> {
    let total_tiles = number_of_tile_columns * number_of_tile_rows;
    let mut visited = vec![false; total_tiles];
    let mut regions = Vec::new();

    for row in 0..number_of_tile_rows {
        for col in 0..number_of_tile_columns {
            let idx = row * number_of_tile_columns + col;
            if solidness_map_slice[idx] == 1 && !visited[idx] {
                let mut region = Vec::new();
                flood_fill_connected_region(
                    solidness_map_slice,
                    number_of_tile_columns,
                    number_of_tile_rows,
                    col,
                    row,
                    &mut visited,
                    &mut region,
                );
                regions.push(region);
            }
        }
    }

    regions
}

fn check_if_tile_has_solid_pixel(
    pixel_mask_slice: &[u8],
    image_width_pixels: usize,
    image_height_pixels: usize,
    tile_column: usize,
    tile_row: usize,
    tile_edge_length: usize,
) -> bool {
    let start_x = tile_column * tile_edge_length;
    let end_x   = ((tile_column + 1) * tile_edge_length).min(image_width_pixels);
    let start_y = tile_row    * tile_edge_length;
    let end_y   = ((tile_row + 1)  * tile_edge_length).min(image_height_pixels);

    for y in start_y..end_y {
        for x in start_x..end_x {
            let pixel_index = y * image_width_pixels + x;
            if pixel_mask_slice[pixel_index] == 1 {
                return true;
            }
        }
    }
    false
}

fn flood_fill_connected_region(
    solidness_map_slice: &[u8],
    number_of_tile_columns: usize,
    number_of_tile_rows: usize,
    start_column: usize,
    start_row: usize,
    visited_flags: &mut [bool],
    out_region_tiles: &mut Vec<(usize, usize)>,
) {
    let mut queue = VecDeque::new();
    queue.push_back((start_column, start_row));
    visited_flags[start_row * number_of_tile_columns + start_column] = true;

    while let Some((current_x, current_y)) = queue.pop_front() {
        out_region_tiles.push((current_x, current_y));

        for (dx, dy) in &[(1isize,0),(-1,0),(0,1),(0,-1)] {
            let nx = current_x as isize + dx;
            let ny = current_y as isize + dy;
            if nx >= 0 && ny >= 0 {
                let ux = nx as usize;
                let uy = ny as usize;
                if ux < number_of_tile_columns && uy < number_of_tile_rows {
                    let nidx = uy * number_of_tile_columns + ux;
                    if solidness_map_slice[nidx] == 1 && !visited_flags[nidx] {
                        visited_flags[nidx] = true;
                        queue.push_back((ux, uy));
                    }
                }
            }
        }
    }
}


const CARDINAL_DIRECTIONS: [(isize, isize); 4] = [
    (1, 0),    // right
    (0, 1),    // down
    (-1, 0),   // left
    (0, -1),   // up
];

struct RegionBounds {
    min_column: usize,
    min_row: usize,
    max_column: usize,
    max_row: usize,
}

pub fn trace_region_contour(
    region_tiles: &[(usize, usize)],
    tile_edge_length: usize,
    image_height_pixels: usize,
) -> Vec<Vector2> {
    if region_tiles.is_empty() {
        return Vec::new();
    }

    let region_bounds = compute_region_bounds(region_tiles);
    let (grid_width, grid_height) =
        compute_grid_dimensions(&region_bounds);
    let region_mask =
        build_region_mask(region_tiles, &region_bounds, grid_width, grid_height);
    let (start_column, start_row) =
        find_contour_start_tile(region_tiles, &region_bounds, grid_width, grid_height, &region_mask);

    follow_region_contour(
        start_column,
        start_row,
        &region_bounds,
        grid_width,
        grid_height,
        tile_edge_length,
        image_height_pixels,
        &region_mask,
    )
}

fn compute_region_bounds(
    region_tiles: &[(usize, usize)],
) -> RegionBounds {
    let min_column = region_tiles.iter().map(|&(c, _)| c).min().unwrap();
    let max_column = region_tiles.iter().map(|&(c, _)| c).max().unwrap();
    let min_row    = region_tiles.iter().map(|&(_, r)| r).min().unwrap();
    let max_row    = region_tiles.iter().map(|&(_, r)| r).max().unwrap();
    RegionBounds {
        min_column,
        min_row,
        max_column,
        max_row,
    }
}

fn compute_grid_dimensions(
    bounds: &RegionBounds,
) -> (usize /*width*/, usize /*height*/) {
    let width_in_tiles  = bounds.max_column - bounds.min_column + 1;
    let height_in_tiles = bounds.max_row    - bounds.min_row    + 1;
    (width_in_tiles, height_in_tiles)
}

fn build_region_mask(
    region_tiles: &[(usize, usize)],
    bounds: &RegionBounds,
    grid_width: usize,
    grid_height: usize,
) -> Vec<u8> {
    let mut mask_vector = vec![0u8; grid_width * grid_height];
    for &(tile_column, tile_row) in region_tiles {
        let local_x = tile_column - bounds.min_column;
        let local_y = tile_row    - bounds.min_row;
        mask_vector[local_y * grid_width + local_x] = 1;
    }
    mask_vector
}

fn find_contour_start_tile(
    region_tiles: &[(usize, usize)],
    bounds: &RegionBounds,
    grid_width: usize,
    grid_height: usize,
    mask_vector: &[u8],
) -> (usize /*start_column*/, usize /*start_row*/) {
    for &(tile_column, tile_row) in region_tiles {
        let local_x = tile_column - bounds.min_column;
        let local_y = tile_row    - bounds.min_row;
        for &(delta_x, delta_y) in &CARDINAL_DIRECTIONS {
            let neighbor_x = local_x as isize + delta_x;
            let neighbor_y = local_y as isize + delta_y;
            if neighbor_x < 0
                || neighbor_y < 0
                || neighbor_x as usize >= grid_width
                || neighbor_y as usize >= grid_height
                || mask_vector[neighbor_y as usize * grid_width + neighbor_x as usize] == 0
            {
                return (tile_column, tile_row);
            }
        }
    }
    (bounds.min_column, bounds.min_row)
}

fn follow_region_contour(
    start_tile_column: usize,
    start_tile_row: usize,
    bounds: &RegionBounds,
    grid_width: usize,
    grid_height: usize,
    tile_edge_length: usize,
    image_height_pixels: usize,
    mask_vector: &[u8],
) -> Vec<Vector2> {
    let mut contour_points = Vec::new();
    let mut current_local_x = start_tile_column - bounds.min_column;
    let mut current_local_y = start_tile_row    - bounds.min_row;
    let mut previous_direction_index = CARDINAL_DIRECTIONS.len() - 1;

    loop {
        let mut moved_to_next_cell = false;

        for step_offset in 0..CARDINAL_DIRECTIONS.len() {
            let candidate_direction_index =
                (previous_direction_index + step_offset) % CARDINAL_DIRECTIONS.len();
            let (delta_x, delta_y) = CARDINAL_DIRECTIONS[candidate_direction_index];

            let next_local_x_isize = current_local_x as isize + delta_x;
            let next_local_y_isize = current_local_y as isize + delta_y;

            if next_local_x_isize >= 0
                && next_local_y_isize >= 0
                && (next_local_x_isize as usize) < grid_width
                && (next_local_y_isize as usize) < grid_height
            {
                let next_index = next_local_y_isize as usize * grid_width
                    + next_local_x_isize as usize;
                if mask_vector[next_index] == 1 {
                    let offset_for_x = if delta_x > 0 { 1 } else { 0 };
                    let offset_for_y = if delta_y < 0 { 1 } else { 0 };
                    let world_x = ((bounds.min_column + current_local_x) + offset_for_x)
                        * tile_edge_length;
                    let world_y = image_height_pixels
                        - ((bounds.min_row + current_local_y) + offset_for_y)
                            * tile_edge_length;

                    contour_points.push(Vector2::new(world_x as f32, world_y as f32));

                    current_local_x = next_local_x_isize as usize;
                    current_local_y = next_local_y_isize as usize;
                    previous_direction_index = (candidate_direction_index
                        + CARDINAL_DIRECTIONS.len()
                        - 1)
                        % CARDINAL_DIRECTIONS.len();

                    moved_to_next_cell = true;
                    break;
                }
            }
        }

        if !moved_to_next_cell
            || (current_local_x == start_tile_column - bounds.min_column
                && current_local_y == start_tile_row    - bounds.min_row)
        {
            break;
        }
    }

    contour_points
}

pub fn compute_convex_hull_monotone_chain(
    boundary_points_slice: &[Vector2],
) -> Vec<Vector2> {
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
        point_a.x
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

fn merge_monotone_chain_hulls(
    mut lower_chain: Vec<Vector2>,
    mut upper_chain: Vec<Vector2>,
) -> Vec<Vector2> {
    lower_chain.pop();
    upper_chain.pop();
    lower_chain.extend(upper_chain);
    lower_chain
}

fn compute_cross_product_z(
    point_a: &Vector2,
    point_b: &Vector2,
    point_c: &Vector2,
) -> f32 {
    let delta_x_ab = point_b.x - point_a.x;
    let delta_y_ab = point_b.y - point_a.y;
    let delta_x_bc = point_c.x - point_a.x;
    let delta_y_bc = point_c.y - point_a.y;
    delta_x_ab * delta_y_bc - delta_y_ab * delta_x_bc
}
