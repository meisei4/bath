use godot::prelude::*;
use godot::classes::{Sprite2D, ISprite2D, Node2D};
use godot::builtin::{PackedByteArray, PackedVector2Array, Vector2, Vector2i};
use std::collections::VecDeque;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}

#[derive(GodotClass)]
#[class(init, base=Sprite2D)]
struct Player {
    #[base]
    base: Base<Sprite2D>,

    #[export]
    #[init(val = 200.0)]
    speed: f64,

    #[export]
    #[init(val = 180.0)]
    angular_speed: f64,
}

#[godot_api]
impl ISprite2D for Player {
    fn process(&mut self, delta: f64) {
        let dx = (self.speed  as f32) * (delta as f32);
        let dr = (self.angular_speed as f32) * (delta as f32);
        let old_pos = self.base().get_position();
        let new_pos = Vector2::new(old_pos.x + dx, old_pos.y);
        self.base_mut().set_position(new_pos);
        let old_rot = self.base().get_rotation_degrees();
        let new_rot = old_rot + dr;
        self.base_mut().set_rotation_degrees(new_rot);
    }
}

#[derive(GodotClass)]
#[class(init, base=Node2D)]
struct TileUtilities {
    #[base]
    base: Base<Node2D>,
}

// #[gdextension]
// unsafe impl ExtensionLibrary for TileUtilities {}

#[godot_api]
impl TileUtilities {
    #[func]
    pub fn scan_any_solid_pixel_in_tile(
        &self,
        pixel_mask_array: PackedByteArray,
        image_width: i64,
        image_height: i64,
        tile_x: i64,
        tile_y: i64,
        tile_size: i64,
    ) -> bool {
        let start_x = tile_x * tile_size;
        let end_x = ((tile_x + 1) * tile_size).min(image_width);
        let start_y = tile_y * tile_size;
        let end_y = ((tile_y + 1) * tile_size).min(image_height);

        for y in start_y..end_y {
            for x in start_x..end_x {
                let idx = (y * image_width + x) as usize;
                if pixel_mask_array.get(idx) == Some(1) {
                    return true;
                }
            }
        }
        false
    }

    #[func]
    pub fn get_unvisited_solid_neighbors(
        &self,
        cell: Vector2i,
        tile_columns: i32,
        tile_rows: i32,
        tile_solidness_array: PackedByteArray,
        visited_array: PackedByteArray,
    ) -> PackedVector2Array {
        let directions = [
            Vector2i::new(1, 0),
            Vector2i::new(-1, 0),
            Vector2i::new(0, 1),
            Vector2i::new(0, -1),
        ];

        let mut result = PackedVector2Array::new();
        for dir in directions {
            let nx = cell.x + dir.x;
            let ny = cell.y + dir.y;
            if nx >= 0 && ny >= 0 && nx < tile_columns && ny < tile_rows {
                let index = (ny * tile_columns + nx) as usize;
                if tile_solidness_array.get(index) == Some(1) && visited_array.get(index) == Some(0) {
                    //visited_array[index] = 1;
                    result.push(Vector2::new(nx as f32, ny as f32));
                }
            }
        }
        result
    }

    /// Replaces GDScript `_update_tile_solidness_array`
    ///
    /// # GDScript signature
    /// `_update_tile_solidness_array(pixel_mask_array, width, height, tile_columns, tile_rows, tile_size)`
    #[func]
    pub fn update_tile_solidness_array(
        &self,
        pixel_mask_array: PackedByteArray,
        image_width: i64,
        image_height: i64,
        tile_column_count: i32,
        tile_row_count: i32,
        tile_size: i32,
    ) -> PackedByteArray {
        let width = image_width as usize;
        let height = image_height as usize;
        let cols = tile_column_count as usize;
        let rows = tile_row_count as usize;
        let ts = tile_size as usize;

        // Prepare the output array of 0/1 per tile
        let mut tile_solidness_array = PackedByteArray::new();
        tile_solidness_array.resize(cols * rows);

        for tile_y in 0..rows {
            for tile_x in 0..cols {
                // compute pixel bounds of this tile
                let start_x = tile_x * ts;
                let end_x = ((tile_x + 1) * ts).min(width);
                let start_y = tile_y * ts;
                let end_y = ((tile_y + 1) * ts).min(height);

                // scan for any solid pixel
                let mut found_solid = false;
                'pixel_scan: for py in start_y..end_y {
                    for px in start_x..end_x {
                        let idx = py * width + px;
                        if pixel_mask_array.get(idx) == Some(1) {
                            found_solid = true;
                            break 'pixel_scan;
                        }
                    }
                }

                let idx = tile_y * cols + tile_x;
                tile_solidness_array[idx] = if found_solid { 1 } else { 0 };
            }
        }

        tile_solidness_array
    }

    /// Replaces GDScript `_find_all_connected_regions_in_tile_array_packed`
    ///
    /// # GDScript signature
    /// `_find_all_connected_regions_in_tile_array_packed(tile_solidness_array, tile_columns, tile_rows) -> Array[PackedVector2Array]`
    #[func]
    pub fn find_all_connected_regions_in_tile_array_packed(
        &self,
        tile_solidness_array: PackedByteArray,
        tile_column_count: i32,
        tile_row_count: i32,
    ) -> Array<PackedVector2Array> {
        let cols = tile_column_count as usize;
        let rows = tile_row_count as usize;

        // track visited tiles
        let mut visited = vec![0u8; cols * rows];
        let mut regions = Array::<PackedVector2Array>::new();
        // 4-neighbour offsets
        let neighbour_deltas = [
            ( 1isize,  0isize),
            (-1,        0),
            ( 0,        1),
            ( 0,       -1),
        ];

        for tile_y in 0..rows {
            for tile_x in 0..cols {
                let linear_idx = tile_y * cols + tile_x;
                // start a new region if solid & unvisited
                if tile_solidness_array.get(linear_idx) == Some(1) && visited[linear_idx] == 0 {
                    visited[linear_idx] = 1;

                    // BFS/DFS queue
                    let mut queue = VecDeque::new();
                    queue.push_back((tile_x, tile_y));

                    // collect this region’s tiles
                    let mut packed_tile_list = PackedVector2Array::new();

                    while let Some((cx, cy)) = queue.pop_back() {
                        // append this tile
                        packed_tile_list.push(Vector2::new(cx as f32, cy as f32));

                        // enqueue its unvisited, solid neighbours
                        for (dx, dy) in &neighbour_deltas {
                            let nx = cx as isize + dx;
                            let ny = cy as isize + dy;
                            if nx >= 0 && ny >= 0 && (nx as usize) < cols && (ny as usize) < rows {
                                let nidx = (ny as usize) * cols + (nx as usize);
                                if tile_solidness_array.get(nidx) == Some(1) && visited[nidx] == 0 {
                                    visited[nidx] = 1;
                                    queue.push_back((nx as usize, ny as usize));
                                }
                            }
                        }
                    }
                    //push a reference of the packed_tile_list rustyyyyy???
                    regions.push(&packed_tile_list);
                }
            }
        }
        regions
    }
}
