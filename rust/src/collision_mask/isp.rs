use godot::builtin::Vector2;
use godot::prelude::{godot_print, real, Array, PackedInt32Array, PackedVector2Array};

pub const MAX_POLYGONS: usize = 24;

const PARALLAX_PROJECTION: f32 = 6.0;
const NOISE_SCROLL_VELOCITY_Y: f32 = 0.025;

pub fn compute_quantized_vertical_pixel_coord(i_time: f32, i_resolution: Vector2) -> i32 {
    let base_normalized_y = -1.0;
    let projected_base_y = base_normalized_y / (PARALLAX_PROJECTION - base_normalized_y);
    let initial_screen_y = (projected_base_y * i_resolution.y + i_resolution.y) * 0.5;
    let projected_scrolled_y = projected_base_y + i_time * NOISE_SCROLL_VELOCITY_Y;
    let scrolled_screen_y = (projected_scrolled_y * i_resolution.y + i_resolution.y) * 0.5;
    (scrolled_screen_y - initial_screen_y).floor() as i32
}

pub fn update_polygons_with_scanline_alpha_buckets(
    collision_polygons: &mut Array<PackedVector2Array>,
    scanline_alpha_buckets: &PackedVector2Array,
    scanline_count_per_polygon: &mut PackedInt32Array,
) {
    //shift_polygon_vertices_down_by_vertical_scroll_1_pixel(collision_polygons);
    for alpha_bucket_index in 0..scanline_alpha_buckets.len() {
        let bucket = scanline_alpha_buckets.get(alpha_bucket_index).unwrap();
        let bucket_start = bucket.x;
        let bucket_end = bucket.y;
        let mut updated_polygon = false;
        for idx in 0..MAX_POLYGONS {
            if !polygon_found(scanline_count_per_polygon, idx) {
                continue;
            }
            let mut polygon_vertices = collision_polygons.get(idx).unwrap();
            let polygon_top_right_vertex = polygon_vertices.get(1).unwrap().x;
            let polygon_top_left_vertex = polygon_vertices.get(0).unwrap().x;
            if scanline_bucket_overlaps_polygon(
                bucket_start,
                bucket_end,
                polygon_top_right_vertex,
                polygon_top_left_vertex,
            ) {
                let new_vertex_start = Vector2::new(bucket_start, 0.0);
                let new_vertex_end = Vector2::new(bucket_end, 0.0);
                polygon_vertices.insert(0, new_vertex_end);
                polygon_vertices.insert(0, new_vertex_start);
                collision_polygons.set(idx, &polygon_vertices);
                let scanline_count = scanline_count_per_polygon.get(idx).unwrap() + 1;
                let slice: &mut [i32] = scanline_count_per_polygon.as_mut_slice();
                slice[idx] = scanline_count;
                updated_polygon = true;
                break;
            }
        }
        if updated_polygon {
            continue;
        }
        for polygon_index in 0..MAX_POLYGONS {
            if polygon_found(scanline_count_per_polygon, polygon_index) {
                continue;
            }
            scanline_count_per_polygon.insert(polygon_index, 1);
            let mut collision_polygon_vertices = PackedVector2Array::new();
            let new_vertex_start = Vector2::new(bucket_start, 0.0);
            let new_vertex_end = Vector2::new(bucket_end, 0.0);
            collision_polygon_vertices.push(new_vertex_start);
            collision_polygon_vertices.push(new_vertex_end);
            collision_polygons.set(polygon_index, &collision_polygon_vertices);
            break;
        }
    }
}

fn polygon_found(scanline_count_per_polygon: &PackedInt32Array, polygon_index: usize) -> bool {
    let row_count = scanline_count_per_polygon.get(polygon_index).unwrap();
    row_count != 0
}

fn scanline_bucket_overlaps_polygon(
    bucket_start: real,
    bucket_end: real,
    polygon_top_right_vertex: real,
    polygon_top_left_vertex: real,
) -> bool {
    bucket_start <= polygon_top_right_vertex && bucket_end >= polygon_top_left_vertex
}

pub fn shift_polygon_vertices_down_by_vertical_scroll_1_pixel(
    collision_polygons: &mut Array<PackedVector2Array>,
) {
    for i in 0..MAX_POLYGONS {
        let mut collision_polygon_vertices = collision_polygons.get(i).unwrap();
        let slice: &mut [Vector2] = collision_polygon_vertices.as_mut_slice();
        for vertex in slice.iter_mut() {
            vertex.y += 1.0; //TODO: this is always 1.0 pixel in screen space
        }
        collision_polygons.set(i, &collision_polygon_vertices);
    }
}

pub fn apply_vertical_projection(
    projected_polygons: &mut Array<PackedVector2Array>,
    i_resolution: Vector2,
    scanline_count_per_polygon: &PackedInt32Array,
    i_time: f32,
) {
    for i in 0..projected_polygons.len() {
        let mut screen_space_polygon = projected_polygons.get(i).unwrap();
        if screen_space_polygon.is_empty() {
            continue;
        }
        let vert_count = screen_space_polygon.len();
        let pair_count = vert_count / 2;
        let scanlines = scanline_count_per_polygon.get(i).unwrap_or(0);
        godot_print!("======== POLYGON_ID: {} ========", i);
        godot_print!(
            "pairs: {}   scanlines: {}   verts: {}",
            pair_count,
            scanlines,
            vert_count
        );
        godot_print!("-- BEFORE--");
        print_polygon_vert_pairs(&screen_space_polygon);
        let slice = screen_space_polygon.as_mut_slice();
        for vert_idx in 0..slice.len() {
            let vertex = slice[vert_idx];

            let projected_y = project_vertex_y(vertex.y, i_resolution, i_time)
                - project_vertex_y(0.0, i_resolution, i_time);
            slice[vert_idx] = Vector2::new(vertex.x, projected_y);
        }
        projected_polygons.set(i, &screen_space_polygon);
        godot_print!("-- AFTER Y PROJECTION--");
        print_polygon_vert_pairs(&screen_space_polygon);
        godot_print!("===============================\n");
    }
}

fn print_polygon_vert_pairs(polygon: &PackedVector2Array) {
    let vert_count = polygon.len();
    if vert_count < 2 {
        return;
    }
    let pair_count = vert_count / 2;
    let v0 = polygon.get(0).unwrap();
    let v1 = polygon.get(1).unwrap();
    print_vert_pair("top_verts:", v0, v1);
    if pair_count == 1 {
        return;
    }
    if pair_count == 2 {
        let vb0 = polygon.get(vert_count - 2).unwrap();
        let vb1 = polygon.get(vert_count - 1).unwrap();
        print_vert_pair("bot_verts:", vb0, vb1);
        return;
    }
    if pair_count >= 3 {
        let middle_pair_index = pair_count / 2;
        let middle_vert_index = middle_pair_index * 2;
        let vm0 = polygon.get(middle_vert_index).unwrap();
        let vm1 = polygon.get(middle_vert_index + 1).unwrap();
        print_vert_pair("mid_verts:", vm0, vm1);
        let vb0 = polygon.get(vert_count - 2).unwrap();
        let vb1 = polygon.get(vert_count - 1).unwrap();
        print_vert_pair("bot_verts:", vb0, vb1);
    }
}

fn print_vert_pair(label: &str, v0: Vector2, v1: Vector2) {
    godot_print!(
        "{:<10} [{:>5}, {:>9.5}]   [{:>5}, {:>9.5}]",
        label,
        v0.x,
        v0.y,
        v1.x,
        v1.y
    );
}

fn project_vertex_y(cpu_y: f32, i_resolution: Vector2, i_time: f32) -> f32 {
    let res_y = i_resolution.y;
    let frag_y = res_y - cpu_y;
    let norm_y = (2.0 * frag_y - res_y) / res_y;
    let projected_scrolled_y = norm_y + i_time * NOISE_SCROLL_VELOCITY_Y;
    let proj_norm_y = projected_scrolled_y / (PARALLAX_PROJECTION + projected_scrolled_y);
    //let proj_norm_y = norm_y / (PARALLAX_PROJECTION + norm_y);
    let frag_y_proj = (proj_norm_y * res_y + res_y) * 0.5;
    res_y - frag_y_proj
}

pub fn apply_horizontal_projection(
    collision_polygons: &Array<PackedVector2Array>,
    i_resolution: Vector2,
) -> Array<PackedVector2Array> {
    let mut projected_polygons: Array<PackedVector2Array> = Array::new();
    for i in 0..collision_polygons.len() {
        let screen_space_vertices = collision_polygons.get(i).unwrap();
        let screen_space_slice = screen_space_vertices.as_slice();
        let mut projected_poly = PackedVector2Array::new();
        for vertex in screen_space_slice {
            let normalized_y = (2.0 * vertex.y - i_resolution.y) / i_resolution.y;
            let projection_factor = PARALLAX_PROJECTION / (PARALLAX_PROJECTION - normalized_y);
            let normalized_x = (2.0 * vertex.x - i_resolution.x) / i_resolution.y;
            let projected_normalized_x = normalized_x * projection_factor;
            let projected_screen_x =
                (projected_normalized_x * i_resolution.y + i_resolution.x) * 0.5;
            projected_poly.push(Vector2::new(projected_screen_x, vertex.y));
        }
        projected_polygons.push(&projected_poly);
    }
    projected_polygons
}
