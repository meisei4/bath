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
    let screen_y = (projected_scrolled_y * i_resolution.y + i_resolution.y) * 0.5;
    (screen_y - initial_screen_y).floor() as i32
}

fn polygon_found(scanline_count_per_polygon: &PackedInt32Array, polygon_index: usize) -> bool {
    let row_count = scanline_count_per_polygon.get(polygon_index).unwrap();
    row_count != 0
}

fn scanline_bucket_overlaps_polygon(
    bucket_start: real,
    collision_polygon_top_right_vertex: real,
    bucket_end: real,
    collision_polygon_top_left_vertex: real,
) -> bool {
    bucket_start <= collision_polygon_top_right_vertex
        && bucket_end >= collision_polygon_top_left_vertex
}

fn shift_polygon_verticies_down_by_vertical_scroll(
    vertical_scroll_per_scanline: f32,
    on_screen_collision_polygon_vertices: &mut Array<PackedVector2Array>,
) {
    for i in 0..MAX_POLYGONS {
        let mut collision_polygon_vertices = on_screen_collision_polygon_vertices.get(i).unwrap();
        let slice: &mut [Vector2] = collision_polygon_vertices.as_mut_slice();
        for vertex in slice.iter_mut() {
            vertex.y += vertical_scroll_per_scanline;
        }
        on_screen_collision_polygon_vertices.set(i, &collision_polygon_vertices);
    }
}

pub fn update_polygons_with_scanline_alpha_buckets(
    i_resolution: Vector2,
    on_screen_collision_polygon_vertices: &mut Array<PackedVector2Array>,
    scanline_alpha_buckets: &PackedVector2Array,
    scanline_count_per_polygon: &mut PackedInt32Array,
    vertical_scroll_per_scanline: f32,
) {
    shift_polygon_verticies_down_by_vertical_scroll(
        vertical_scroll_per_scanline,
        on_screen_collision_polygon_vertices,
    );
    for alpha_bucket_index in 0..scanline_alpha_buckets.len() {
        let bucket = scanline_alpha_buckets.get(alpha_bucket_index).unwrap();
        let bucket_start = bucket.x;
        let bucket_end = bucket.y;
        let mut updated_polygon = false;
        for polygon_index in 0..MAX_POLYGONS {
            if !polygon_found(scanline_count_per_polygon, polygon_index) {
                continue;
            }
            let mut collision_polygon_vertices = on_screen_collision_polygon_vertices
                .get(polygon_index)
                .unwrap();
            let count = scanline_count_per_polygon.get(polygon_index).unwrap();
            godot_print!("slot {} active_count={} vert_count={}", polygon_index, count, collision_polygon_vertices.len());
            if collision_polygon_vertices.len() < 2 {
                continue;
            }
            let collision_polygon_top_right_vertex = collision_polygon_vertices.get(1).unwrap().x;
            let collision_polygon_top_left_vertex = collision_polygon_vertices.get(0).unwrap().x;
            if scanline_bucket_overlaps_polygon(
                bucket_start,
                collision_polygon_top_right_vertex,
                bucket_end,
                collision_polygon_top_left_vertex,
            ) {
                collision_polygon_vertices
                    .insert(0, Vector2::new(bucket_end, vertical_scroll_per_scanline));
                collision_polygon_vertices
                    .insert(0, Vector2::new(bucket_start, vertical_scroll_per_scanline));
                on_screen_collision_polygon_vertices
                    .set(polygon_index, &collision_polygon_vertices);
                let updated_scanline_count =
                    scanline_count_per_polygon.get(polygon_index).unwrap() + 1;
                let slice: &mut [i32] = scanline_count_per_polygon.as_mut_slice();
                slice[polygon_index] = updated_scanline_count;
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
            collision_polygon_vertices.push(Vector2::new(bucket_start, 0.0));
            collision_polygon_vertices.push(Vector2::new(bucket_end, 0.0));
            on_screen_collision_polygon_vertices.set(polygon_index, &collision_polygon_vertices);
            break;
        }
    }
}

pub fn advance_polygons_by_one_frame(
    polygon_active_global: &mut PackedInt32Array,
    polygon_active_local: &mut PackedInt32Array,
    polygon_positions_y: &mut PackedFloat32Array,
    polygon_segments: &mut Array<PackedVector2Array>,
    polygon_1d_x_coords: &mut Array<PackedFloat32Array>,
    polygon_1d_y_coords: &mut Array<PackedFloat32Array>,
    screen_resolution: Vector2,
) {
    for idx in 0..MAX_POLYGONS {
        if polygon_active_global.get(idx).unwrap() == 0 {
            continue;
        }
        vertical_scroll_projected(
            idx,
            polygon_segments,
            polygon_1d_y_coords,
            polygon_positions_y,
            screen_resolution,
        );
        //vertical_scroll_one_pixel(idx, polygon_positions_y);
        apply_horizontal_projection(
            idx,
            polygon_segments,
            polygon_1d_x_coords,
            polygon_positions_y,
            screen_resolution,
        );
        if polygon_positions_y.get(idx).unwrap() > screen_resolution.y {
            clear_polygon(
                idx,
                polygon_active_global,
                polygon_active_local,
                polygon_positions_y,
                polygon_segments,
                polygon_1d_x_coords,
                polygon_1d_y_coords,
            );
        }
    }
}

pub fn clear_polygon(
    polygon_index: usize,
    polygon_active_global: &mut PackedInt32Array,
    polygon_active_local: &mut PackedInt32Array,
    polygon_positions_y: &mut PackedFloat32Array,
    polygon_segments: &mut Array<PackedVector2Array>,
    polygon_1d_x_coords: &mut Array<PackedFloat32Array>,
    polygon_1d_y_coords: &mut Array<PackedFloat32Array>,
) {
    polygon_active_global.insert(polygon_index, 0);
    polygon_active_local.insert(polygon_index, 0);
    polygon_positions_y.insert(polygon_index, 0.0);
    let mut cleared_segment_array: PackedVector2Array =
        polygon_segments.get(polygon_index).unwrap();
    cleared_segment_array.clear();
    polygon_segments.set(polygon_index, &cleared_segment_array);
    let mut cleared_x_array = polygon_1d_x_coords.get(polygon_index).unwrap();
    cleared_x_array.clear();
    polygon_1d_x_coords.set(polygon_index, &cleared_x_array);
    let mut cleared_y_array = polygon_1d_y_coords.get(polygon_index).unwrap();
    cleared_y_array.clear();
    polygon_1d_y_coords.set(polygon_index, &cleared_y_array);
}

fn vertical_scroll_one_pixel(polygon_index: usize, polygon_positions_y: &mut PackedFloat32Array) {
    let next_y = polygon_positions_y.get(polygon_index).unwrap() + 1.0;
    polygon_positions_y.insert(polygon_index, next_y);
}

fn vertical_scroll_projected(
    polygon_index: usize,
    polygon_segments: &mut Array<PackedVector2Array>,
    polygon_1d_y_coords: &mut Array<PackedFloat32Array>,
    polygon_positions_y: &mut PackedFloat32Array,
    screen_resolution: Vector2,
) {
    let half_h = screen_resolution.y * 0.5;
    let mut world_y_array = polygon_1d_y_coords.get(polygon_index).unwrap();
    let len = world_y_array.len();
    for i in 0..len {
        // let y_new = world_y_array.get(i).unwrap() + NOISE_SCROLL_VELOCITY_Y;
        // world_y_array.remove(i);
        // world_y_array.insert(i, y_new);
        let w = world_y_array.get(i).unwrap();
        let n = w * PARALLAX_PROJECTION_ASYMPTOTIC_DEPTH_SCALAR / (1.0 + w);
        let scr_y = (n + 1.0) * half_h;
        let scr_y_next = scr_y + 1.0;
        let m_next = 2.0 * scr_y_next / screen_resolution.y - 1.0;
        let w_next = m_next / (PARALLAX_PROJECTION_ASYMPTOTIC_DEPTH_SCALAR - m_next);
        world_y_array.remove(i);
        world_y_array.insert(i, w_next);
    }

    polygon_1d_y_coords.set(polygon_index, &world_y_array);
    let mut screen_y_values: Vec<f32> = Vec::with_capacity(len);
    let mut min_screen_y = f32::MAX;
    for i in 0..len {
        // let n = world_y_array.get(i).unwrap();
        // let scale = 1.0 / (PARALLAX_PROJECTION_ASYMPTOTIC_DEPTH_SCALAR - n);
        // let scr_y = n * scale * half_h + half_h;
        let w = world_y_array.get(i).unwrap();
        let n = w * PARALLAX_PROJECTION_ASYMPTOTIC_DEPTH_SCALAR / (1.0 + w);
        let scr_y = (n + 1.0) * half_h;
        screen_y_values.push(scr_y);
        if scr_y < min_screen_y {
            min_screen_y = scr_y;
        }
    }
    let mut segments = polygon_segments.get(polygon_index).unwrap();
    for i in 0..len {
        let x_val = segments.get(i).unwrap().x;
        let y_local = screen_y_values[i] - min_screen_y;
        segments.remove(i);
        segments.insert(i, Vector2::new(x_val, y_local));
    }
    polygon_segments.set(polygon_index, &segments);
    polygon_positions_y.insert(polygon_index, min_screen_y);
}

fn apply_horizontal_projection(
    polygon_index: usize,
    polygon_segments: &mut Array<PackedVector2Array>,
    polygon_1d_x_coords: &Array<PackedFloat32Array>,
    polygon_positions_y: &PackedFloat32Array,
    screen_resolution: Vector2,
) {
    let half_h = screen_resolution.y * 0.5;
    let half_w = screen_resolution.x * 0.5;
    let top_y = polygon_positions_y.get(polygon_index).unwrap();
    let mut segs = polygon_segments.get(polygon_index).unwrap();
    let world_x_array = polygon_1d_x_coords.get(polygon_index).unwrap();

    for v in 0..segs.len() {
        let local = segs.get(v).unwrap();
        let frag_y = local.y + top_y;
        let norm_y = (2.0 * frag_y - screen_resolution.y) / screen_resolution.y;
        let scale = 1.0 / (PARALLAX_PROJECTION_ASYMPTOTIC_DEPTH_SCALAR - norm_y);
        let scr_x = world_x_array.get(v).unwrap() * scale * half_h + half_w;
        segs.remove(v);
        segs.insert(v, Vector2::new(scr_x, local.y));
    }
    polygon_segments.set(polygon_index, &segs);
}
