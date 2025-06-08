use godot::builtin::Vector2;
use godot::prelude::{godot_print, real, Array, PackedFloat32Array, PackedInt32Array, PackedVector2Array};

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

fn shift_polygon_verticies_down_by_vertical_scroll(collision_polygons: &mut Array<PackedVector2Array>, ) {
    for i in 0..MAX_POLYGONS {
        let mut collision_polygon_vertices = collision_polygons.get(i).unwrap();
        let slice: &mut [Vector2] = collision_polygon_vertices.as_mut_slice();
        for vertex in slice.iter_mut() {
            vertex.y += 1.0; //TODO: this is always 1.0 pixel in screen space
        }
        collision_polygons.set(i, &collision_polygon_vertices);
    }
}

pub fn update_polygons_with_scanline_alpha_buckets(
    i_resolution: Vector2,
    collision_polygons: &mut Array<PackedVector2Array>,
    scanline_alpha_buckets: &PackedVector2Array,
    scanline_count_per_polygon: &mut PackedInt32Array,
) {
    shift_polygon_verticies_down_by_vertical_scroll(collision_polygons);
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
            godot_print!("poly:{} scanlines:{} verts:{}", idx, count, polygon_vertices.len());
            let polygon_top_right_vertex = polygon_vertices.get(1).unwrap().x;
            let polygon_top_left_vertex = polygon_vertices.get(0).unwrap().x;
            if scanline_bucket_overlaps_polygon(
                bucket_start,
                bucket_end,
                polygon_top_right_vertex,
                polygon_top_left_vertex,
            ) {
                let new_vertex_start = Vector2::new(bucket_start, 1.0);
                let new_vertex_end = Vector2::new(bucket_end, 1.0);
                polygon_vertices.insert(0, new_vertex_start);
                polygon_vertices.insert(0, new_vertex_end);
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
    projected_polygons: &mut Array<PackedVector2Array>,
    i_resolution: Vector2,
) {
    let half_height    = i_resolution.y * 0.5;
    let half_width     = i_resolution.x * 0.5;
    let screen_width   = i_resolution.x;
    let screen_height  = i_resolution.y;
    for i in 0..collision_polygons.len() {
        let screen_space_vertices = collision_polygons.get(i).unwrap();
        let screen_space_slice: &[Vector2] = screen_space_vertices.as_slice();
        let mut projected_normal_space_vertices = PackedVector2Array::new();
        projected_normal_space_vertices.resize(screen_space_vertices.len());
        for &v in screen_space_slice {
            projected_normal_space_vertices.push(v);
        }
        let projected_slice: &mut [Vector2] = projected_normal_space_vertices.as_mut_slice();
        for j in 0..screen_space_slice.len() {
            let v = screen_space_slice[j];
            let ndc_y = (2.0 * v.y - screen_height)  / screen_height;
            let ndc_x = (2.0 * v.x - screen_width)  / screen_width;
            let inv = 1.0 / (PARALLAX_PROJECTION_ASYMPTOTIC_DEPTH_SCALAR - ndc_y);
            let x_ndc_proj = ndc_x * inv;
            let x_pix_proj = x_ndc_proj * half_width + half_width;
            projected_slice[j].x = x_pix_proj;
        }
        projected_polygons.set(i, &projected_normal_space_vertices);
    }
}
