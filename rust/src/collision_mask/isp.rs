use godot::builtin::Vector2;
use godot::prelude::{
    godot_print, real, Array, PackedFloat32Array, PackedInt32Array, PackedVector2Array,
};

pub const MAX_POLYGONS: usize = 24;

const PARALLAX_PROJECTION_ASYMPTOTIC_DEPTH_SCALAR: f32 = 6.0;
const NOISE_SCROLL_VELOCITY_Y: f32 = 0.025;

pub fn compute_quantized_vertical_pixel_coord(i_time: f32, i_resolution: Vector2) -> i32 {
    let base_normalized_y = -1.0;
    let projected_base_y =
        base_normalized_y / (PARALLAX_PROJECTION_ASYMPTOTIC_DEPTH_SCALAR - base_normalized_y);
    let initial_screen_y = (projected_base_y * i_resolution.y + i_resolution.y) * 0.5;
    let projected_scrolled_y = projected_base_y + i_time * NOISE_SCROLL_VELOCITY_Y;
    let scrolled_screen_y = (projected_scrolled_y * i_resolution.y + i_resolution.y) * 0.5;
    (scrolled_screen_y - initial_screen_y).floor() as i32
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

fn shift_polygon_verticies_down_by_vertical_scroll_1_pixel(
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

fn shift_polygon_verticies_down_by_vertical_scroll_projected(
    collision_polygons: &mut Array<PackedVector2Array>,
    polygon_logical_y: &mut PackedFloat32Array,
    i_resolution: Vector2,
) {
    let slice: &mut [f32] = polygon_logical_y.as_mut_slice();
    for polygon_index in 0..MAX_POLYGONS {
        if polygon_index >= slice.len() {
            continue;
        }
        if slice[polygon_index] == 0.0 {
            let base_normalized_y = -1.0;
            slice[polygon_index] = base_normalized_y
                / (PARALLAX_PROJECTION_ASYMPTOTIC_DEPTH_SCALAR - base_normalized_y);
        }
        slice[polygon_index] += NOISE_SCROLL_VELOCITY_Y;
        let logical_y = slice[polygon_index];
        let normalized_y =
            (PARALLAX_PROJECTION_ASYMPTOTIC_DEPTH_SCALAR * logical_y) / (1.0 + logical_y);
        let projection_factor = PARALLAX_PROJECTION_ASYMPTOTIC_DEPTH_SCALAR
            / (PARALLAX_PROJECTION_ASYMPTOTIC_DEPTH_SCALAR - normalized_y);
        let projected_normalized_y = normalized_y * projection_factor;
        let screen_y = (projected_normalized_y * i_resolution.y + i_resolution.y) * 0.5;
        let mut polygon_vertices = collision_polygons.get(polygon_index).unwrap();
        let slice_vertices: &mut [Vector2] = polygon_vertices.as_mut_slice();
        for vertex in slice_vertices.iter_mut() {
            vertex.y = screen_y;
        }
        collision_polygons.set(polygon_index, &polygon_vertices);
    }
}

pub fn update_polygons_with_scanline_alpha_buckets(
    i_resolution: Vector2,
    collision_polygons: &mut Array<PackedVector2Array>,
    polygon_logical_y: &mut PackedFloat32Array,
    scanline_alpha_buckets: &PackedVector2Array,
    scanline_count_per_polygon: &mut PackedInt32Array,
) {
    shift_polygon_verticies_down_by_vertical_scroll_1_pixel(collision_polygons);
    //shift_polygon_verticies_down_by_vertical_scroll_projected(collision_polygons, polygon_logical_y, i_resolution);
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
            let count = scanline_count_per_polygon.get(idx).unwrap();
            godot_print!(
                "poly:{} scanlines:{} verts:{}",
                idx,
                count,
                polygon_vertices.len()
            );
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
            let projection_factor = PARALLAX_PROJECTION_ASYMPTOTIC_DEPTH_SCALAR
                / (PARALLAX_PROJECTION_ASYMPTOTIC_DEPTH_SCALAR - normalized_y);
            let normalized_x = (2.0 * vertex.x - i_resolution.x) / i_resolution.y;
            let projected_normalized_x = normalized_x * projection_factor;
            let projected_screen_x = projected_normalized_x * i_resolution.x + i_resolution.x * 0.5;
            projected_poly.push(Vector2::new(projected_screen_x, vertex.y));
        }
        projected_polygons.push(&projected_poly);
    }
    projected_polygons
}
