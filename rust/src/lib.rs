// src/lib.rs

use godot::prelude::*;
use godot::classes::Node2D;
use godot::builtin::{PackedByteArray, PackedVector2Array, Vector2};

mod collision_mask_util;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}

#[derive(GodotClass)]
#[class(init, base=Node2D)]
struct RustUtil {
    #[base]
    base: Base<Node2D>,
}

#[godot_api]
impl RustUtil {
    /// Entry‐point: compute collision polygons for an image mask.
    #[func]
    pub fn compute_collision_polygons(
        &self,
        raw_pixel_mask: PackedByteArray,
        image_width_pixels: i32,
        image_height_pixels: i32,
        tile_edge_length: i32,
    ) -> Array<PackedVector2Array> {
        let pixel_data: Vec<u8> = raw_pixel_mask.to_vec();
        let width: usize = image_width_pixels as usize;
        let height: usize = image_height_pixels as usize;
        let tile_size: usize = tile_edge_length as usize;

        let (columns, rows) = collision_mask_util::compute_tile_grid_size(width, height, tile_size);

        let hulls: Vec<Vec<Vector2>> = collision_mask_util::generate_collision_polygons(
            &pixel_data,
            (width, height),
            (columns, rows),
            tile_size,
        );

        let mut godot_polygons_array: Array<PackedVector2Array> = Array::new();
        for hull in hulls {
            let mut packed_hull: PackedVector2Array = PackedVector2Array::new();
            for point in hull {
                packed_hull.push(point);
            }
            godot_polygons_array.push(&packed_hull);
        }

        godot_polygons_array
    }
}
