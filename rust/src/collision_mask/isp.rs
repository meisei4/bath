use godot::builtin::Vector2;
use godot::prelude::{real, Array, PackedInt32Array, PackedVector2Array};

pub const MAX_POLYGONS: usize = 8;

fn scanline_bucket_overlaps_polygon(
    bucket_start: real,
    bucket_end: real,
    polygon_top_right_vertex: real,
    polygon_top_left_vertex: real,
) -> bool {
    bucket_start <= polygon_top_right_vertex && bucket_end >= polygon_top_left_vertex
}

pub fn update_polygons_with_scanline_alpha_buckets(
    collision_polygons: &mut Array<PackedVector2Array>,
    scanline_alpha_buckets: &PackedVector2Array,
    scanline_count_per_polygon: &mut PackedInt32Array,
) {
    let counts = scanline_count_per_polygon.as_mut_slice();
    let buckets = scanline_alpha_buckets.as_slice();
    for bucket in buckets {
        let start = bucket.x;
        let end = bucket.y;
        let mut updated = false;
        for idx in 0..MAX_POLYGONS {
            if counts[idx] == 0 {
                continue;
            }
            let mut poly = collision_polygons.get(idx).unwrap();
            let left = poly.get(0).unwrap().x;
            let right = poly.get(1).unwrap().x;

            if scanline_bucket_overlaps_polygon(start, end, right, left) {
                poly.insert(0, Vector2::new(end, 0.0));
                poly.insert(0, Vector2::new(start, 0.0));

                collision_polygons.set(idx, &poly);
                counts[idx] += 1;
                updated = true;
                break;
            }
        }
        if updated {
            continue;
        }

        for idx in 0..MAX_POLYGONS {
            if counts[idx] != 0 {
                continue;
            }
            let mut poly = PackedVector2Array::new();
            poly.push(Vector2::new(start, 0.0));
            poly.push(Vector2::new(end, 0.0));
            collision_polygons.set(idx, &poly);
            counts[idx] = 1;
            break;
        }
    }
}

pub fn shift_polygon_vertices_down_by_pixels(
    collision_polygons: &mut Array<PackedVector2Array>,
    row_counts: &[i32],
    dy: i32,
) {
    if dy == 0 {
        return;
    }
    let delta = dy as f32;
    for slot in 0..MAX_POLYGONS {
        if row_counts[slot] == 0 {
            continue;
        }
        let mut collision_polygon_vertices = collision_polygons.get(slot).unwrap();
        let slice: &mut [Vector2] = collision_polygon_vertices.as_mut_slice();
        for vertex in slice.iter_mut() {
            vertex.y += delta;
        }
        collision_polygons.set(slot, &collision_polygon_vertices);
    }
}
