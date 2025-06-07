use godot::builtin::Vector2;
use godot::prelude::{Array, PackedFloat32Array, PackedInt32Array, PackedVector2Array};

pub const MAX_POLYGONS: usize = 24;

const PARALLAX_PROJECTION_ASYMPTOTIC_DEPTH_SCALAR: f32 = 6.0;
const NOISE_SCROLL_VELOCITY_Y: f32 = 0.05;

pub fn compute_quantized_vertical_pixel_coord(i_time: f32, i_resolution: Vector2) -> i32 {
    let base_normalized_y = -1.0;
    let projected_base_y =
        base_normalized_y / (PARALLAX_PROJECTION_ASYMPTOTIC_DEPTH_SCALAR - base_normalized_y);
    let projected_scrolled_y = projected_base_y + i_time * NOISE_SCROLL_VELOCITY_Y;
    let screen_y = (projected_scrolled_y * i_resolution.y + i_resolution.y) * 0.5;
    screen_y.floor() as i32
}

fn push_1d_coord_pair(
    polygon_1d_x_coords: &mut Array<PackedFloat32Array>,
    polygon_1d_y_coords: &mut Array<PackedFloat32Array>,
    polygon_slot: usize,
    bucket_x_start: f32,
    bucket_x_end: f32,
    fragment_y_spawn: f32,
    screen_resolution: Vector2,
) {
    while polygon_1d_x_coords.len() <= polygon_slot {
        polygon_1d_x_coords.push(&PackedFloat32Array::new());
        polygon_1d_y_coords.push(&PackedFloat32Array::new());
    }
    let mut world_x_coord_array = polygon_1d_x_coords.get(polygon_slot).unwrap();
    let mut world_y_coord_array = polygon_1d_y_coords.get(polygon_slot).unwrap();
    let normalized_spawn_y = (2.0 * fragment_y_spawn - screen_resolution.y) / screen_resolution.y;
    let normalized_left_x = (2.0 * bucket_x_start - screen_resolution.x) / screen_resolution.y;
    let normalized_right_x = (2.0 * bucket_x_end - screen_resolution.x) / screen_resolution.y;
    let depth_scalar = PARALLAX_PROJECTION_ASYMPTOTIC_DEPTH_SCALAR - normalized_spawn_y;
    world_x_coord_array.insert(0, normalized_right_x * depth_scalar);
    world_x_coord_array.insert(0, normalized_left_x * depth_scalar);
    world_y_coord_array.insert(0, normalized_spawn_y);
    world_y_coord_array.insert(0, normalized_spawn_y);
    polygon_1d_x_coords.set(polygon_slot, &world_x_coord_array);
    polygon_1d_y_coords.set(polygon_slot, &world_y_coord_array);
}

pub fn update_polygons_with_alpha_buckets(
    polygon_active_global: &mut PackedInt32Array,
    polygon_active_local: &mut PackedInt32Array,
    polygon_positions_y: &mut PackedFloat32Array,
    polygon_segments: &mut Array<PackedVector2Array>,
    polygon_1d_x_coords: &mut Array<PackedFloat32Array>,
    polygon_1d_y_coords: &mut Array<PackedFloat32Array>,
    alpha_buckets: &PackedVector2Array,
    screen_resolution: Vector2,
) {
    let bucket_pair_count = alpha_buckets.len() / 2;
    for bucket_pair_index in 0..bucket_pair_count {
        let bucket_x_start = alpha_buckets.get(bucket_pair_index * 2).unwrap().x;
        let bucket_x_end = alpha_buckets.get(bucket_pair_index * 2 + 1).unwrap().x;
        let mut polygon_found = false;
        for polygon_index in 0..MAX_POLYGONS {
            if polygon_active_global.get(polygon_index).unwrap() == 0 {
                continue;
            }
            let mut local_segment_array = polygon_segments.get(polygon_index).unwrap();
            if local_segment_array.len() < 2 {
                continue;
            }
            let current_top_left_x = local_segment_array.get(0).unwrap().x;
            let current_top_right_x = local_segment_array.get(1).unwrap().x;
            if bucket_x_start <= current_top_right_x && bucket_x_end >= current_top_left_x {
                let new_local_y = -polygon_positions_y.get(polygon_index).unwrap();
                local_segment_array.insert(0, Vector2::new(bucket_x_end, new_local_y));
                local_segment_array.insert(0, Vector2::new(bucket_x_start, new_local_y));
                polygon_segments.set(polygon_index, &local_segment_array);
                let updated_local_row_count = polygon_active_local.get(polygon_index).unwrap() + 1;
                polygon_active_local.insert(polygon_index, updated_local_row_count);
                push_1d_coord_pair(
                    polygon_1d_x_coords,
                    polygon_1d_y_coords,
                    polygon_index,
                    bucket_x_start,
                    bucket_x_end,
                    new_local_y - polygon_positions_y.get(polygon_index).unwrap(),
                    screen_resolution,
                );
                polygon_found = true;
                break;
            }
        }
        if polygon_found {
            continue;
        }
        for polygon_index in 0..MAX_POLYGONS {
            if polygon_active_global.get(polygon_index).unwrap() != 0 {
                continue;
            }
            polygon_active_global.insert(polygon_index, 1);
            polygon_active_local.insert(polygon_index, 1);
            polygon_positions_y.insert(polygon_index, 0.0);
            let mut new_segment_array = PackedVector2Array::new();
            new_segment_array.push(Vector2::new(bucket_x_start, 0.0));
            new_segment_array.push(Vector2::new(bucket_x_end, 0.0));
            polygon_segments.set(polygon_index, &new_segment_array);
            push_1d_coord_pair(
                polygon_1d_x_coords,
                polygon_1d_y_coords,
                polygon_index,
                bucket_x_start,
                bucket_x_end,
                0.0,
                screen_resolution,
            );
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
        vertical_scroll_one_pixel(idx, polygon_positions_y);
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
        let y_new = world_y_array.get(i).unwrap() + NOISE_SCROLL_VELOCITY_Y;
        world_y_array.remove(i);
        world_y_array.insert(i, y_new);
    }
    polygon_1d_y_coords.set(polygon_index, &world_y_array);
    let mut screen_y_values: Vec<f32> = Vec::with_capacity(len);
    let mut min_screen_y = f32::MAX;
    for i in 0..len {
        let n = world_y_array.get(i).unwrap();
        let scale = 1.0 / (PARALLAX_PROJECTION_ASYMPTOTIC_DEPTH_SCALAR - n);
        //TODO: this is the fucked place i think
        let scr_y = n * scale * half_h + half_h;
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
