use crate::collision_mask::godot::generate_concave_collision_polygons_pixel_perfect;
use crate::collision_mask::isp::{shift_polygon_vertices_down_by_pixels, update_polygons_with_scanline_alpha_buckets};
use godot::builtin::{Array, Dictionary, PackedByteArray, PackedInt32Array, PackedVector2Array, Vector2};
use godot::classes::Node2D;
use godot::global::godot_print;
use godot::obj::Base;
use godot::prelude::{godot_api, GodotClass};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(GodotClass)]
#[class(init, base=Node2D)]
pub struct Collision {
    #[base]
    base: Base<Node2D>,
}

#[godot_api]
impl Collision {
    #[func]
    fn process_scanline_closest_1(
        mut prev_time: f32,
        i_time: f32,
        screen_h: f32,
        noise_vel: Vector2,
        depth: f32,
        global_scale: f32,
        u_stretch: f32,
        stretch_y: f32,
        near_scale: f32,
        scanline_alpha_buckets: PackedVector2Array,
        mut collision_polygons: Array<PackedVector2Array>,
        mut noise_accum: f32,
        mut scanline_count_per_polygon: PackedInt32Array,
        mut total_rows: i32,
    ) -> Dictionary {
        static FRAME_COUNTER: AtomicU64 = AtomicU64::new(0);
        static INIT_LOGGED: AtomicBool = AtomicBool::new(false);
        let frame = FRAME_COUNTER.fetch_add(1, Ordering::Relaxed) + 1;
        if !INIT_LOGGED.load(Ordering::Relaxed) {
            INIT_LOGGED.store(true, Ordering::Relaxed);
            godot_print!(
                "▶ Shader Params:\n\
             • iResolution = 256×384\n\
             • parallaxDepth = {}\n\
             • strideLength = 1.0\n\
             • globalCoordinateScale = {}\n\
             • noiseScrollVelocity = {:?}\n\
             • uniformStretchCorrection = {:.8}\n\
             • stretchScalarY = {}\n\
             • parallaxNearScale = {:.5}",
                depth,
                global_scale,
                noise_vel,
                u_stretch,
                stretch_y,
                near_scale,
            );
        }
        let delta_t = i_time - prev_time;
        if frame % 10 == 0 {
            godot_print!(
                "Frame {:>4} | time = {:>6.3}s (+{:>6.3}s) | noise_accum = {:>8.5}",
                frame,
                i_time,
                delta_t,
                noise_accum
            );
        }
        prev_time = i_time;
        let a = 0.5 * ((depth + 1.0) / (depth - 1.0)).ln();
        let b = 1.5 * (depth * ((depth + 1.0) / (depth - 1.0)).ln() - 2.0);
        let stretch = u_stretch * stretch_y * near_scale;
        let rot = {
            use std::f32::consts::FRAC_1_SQRT_2;
            let rx = FRAC_1_SQRT_2 * (noise_vel.x + noise_vel.y);
            let ry = FRAC_1_SQRT_2 * (-noise_vel.x + noise_vel.y);
            (rx.hypot(ry)) * global_scale * stretch
        };
        noise_accum += rot * delta_t;
        loop {
            let y_norm = (2.0 * (total_rows as f32 + 0.5) / screen_h) - 1.0;
            let scale_y = a + b * y_norm;
            let noise_px = scale_y * stretch;
            //let noise_px = stretch / scale_y;    //broke
            if noise_accum < noise_px {
                break;
            }
            noise_accum -= noise_px;
            if total_rows % 5 == 0 {
                godot_print!(
                    "[SNAP @ row {:>4}] y_norm={:+.4}, scale_y={:+.4}, noise_per_px={:.4}, rem_accum={:.4}",
                    total_rows,
                    y_norm,
                    scale_y,
                    noise_px,
                    noise_accum
                );
                let active_before: Vec<_> = scanline_count_per_polygon
                    .as_slice()
                    .iter()
                    .enumerate()
                    .filter(|&(_, &c)| c != 0)
                    .map(|(i, &c)| format!("[{}]={}", i, c))
                    .collect();
                godot_print!("  BEFORE: active_polygons = {}", active_before.join(", "));
            }
            shift_polygon_vertices_down_by_pixels(&mut collision_polygons, scanline_count_per_polygon.as_slice(), 1);
            update_polygons_with_scanline_alpha_buckets(
                &mut collision_polygons,
                &scanline_alpha_buckets,
                &mut scanline_count_per_polygon,
            );
            if total_rows % 5 == 0 {
                let active_after: Vec<_> = scanline_count_per_polygon
                    .as_slice()
                    .iter()
                    .enumerate()
                    .filter(|&(_, &c)| c != 0)
                    .map(|(i, &c)| format!("[{}]={}", i, c))
                    .collect();
                godot_print!("  AFTER : active_polygons = {}", active_after.join(", "));
            }
            total_rows += 1;
        }
        let mut out = Dictionary::new();
        let _ = out.insert("prev_time", prev_time);
        let _ = out.insert("scroll_accum", noise_accum);
        let _ = out.insert("total_rows_scrolled", total_rows);
        let _ = out.insert("scanline_count_per_polygon", scanline_count_per_polygon);
        let _ = out.insert("collision_polygons", collision_polygons);
        out
    }

    #[func]
    fn process_scanline_logged(
        mut prev_time: f32,
        i_time: f32,
        screen_h: f32,
        noise_vel: Vector2,
        depth: f32,
        global_scale: f32,
        u_stretch: f32,
        stretch_y: f32,
        near_scale: f32,
        scanline_alpha_buckets: PackedVector2Array,
        mut collision_polygons: Array<PackedVector2Array>,
        mut noise_accum: f32,
        mut scanline_count_per_polygon: PackedInt32Array,
        mut total_rows: i32,
    ) -> Dictionary {
        let delta_t = i_time - prev_time;
        prev_time = i_time;
        let a = 0.5 * ((depth + 1.0) / (depth - 1.0)).ln();
        let b = 1.5 * (depth * ((depth + 1.0) / (depth - 1.0)).ln() - 2.0);
        let stretch = u_stretch * stretch_y * near_scale;
        let v_noise = {
            use std::f32::consts::FRAC_1_SQRT_2; // 1/√2
            let rot_x = FRAC_1_SQRT_2 * noise_vel.x + FRAC_1_SQRT_2 * noise_vel.y;
            let rot_y = -FRAC_1_SQRT_2 * noise_vel.x + FRAC_1_SQRT_2 * noise_vel.y;
            (rot_x.hypot(rot_y)) * global_scale * stretch
        };
        noise_accum += v_noise * delta_t;
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let total_millis = now.as_millis();
        let hours = (total_millis / 1000 / 60 / 60) % 24;
        let minutes = (total_millis / 1000 / 60) % 60;
        let seconds = (total_millis / 1000) % 60;
        let millis = total_millis % 1000;
        godot_print!(
            "INFO [SYSTEM_TIME: {:02}:{:02}:{:02}.{:03}] noise_accum: {:.5}",
            hours,
            minutes,
            seconds,
            millis,
            noise_accum
        );
        loop {
            let y_norm = (2.0 * (total_rows as f32 + 0.5) / screen_h) - 1.0;
            let scale_y = a + b * y_norm;
            let noise_per_px = scale_y * stretch;
            if noise_accum < noise_per_px {
                break;
            }
            noise_accum -= noise_per_px;
            godot_print!(
                "y_norm: {:.5} | scale_y: {:.5} | noise_per_px: {:.5} | noise_accum: {:.5}",
                y_norm,
                scale_y,
                noise_per_px,
                noise_accum
            );
            godot_print!("BEFORE");
            godot_print!("scanline_alpha_buckets:");
            for (i, vec) in scanline_alpha_buckets.as_slice().iter().enumerate() {
                godot_print!("  [{}] {:?}", i, vec);
            }
            godot_print!("collision_polygons:");
            for (i, poly) in collision_polygons.iter_shared().enumerate() {
                godot_print!("  [{}] {:?}", i, poly);
            }
            godot_print!("scanline_count_per_polygon:");
            for (i, count) in scanline_count_per_polygon.as_slice().iter().enumerate() {
                godot_print!("  [{}] {}", i, count);
            }
            shift_polygon_vertices_down_by_pixels(&mut collision_polygons, scanline_count_per_polygon.as_slice(), 1);
            update_polygons_with_scanline_alpha_buckets(
                &mut collision_polygons,
                &scanline_alpha_buckets,
                &mut scanline_count_per_polygon,
            );
            godot_print!("AFTER");
            godot_print!("scanline_alpha_buckets:");
            for (i, vec) in scanline_alpha_buckets.as_slice().iter().enumerate() {
                godot_print!("  [{}] {:?}", i, vec);
            }
            godot_print!("collision_polygons:");
            for (i, poly) in collision_polygons.iter_shared().enumerate() {
                godot_print!("  [{}] {:?}", i, poly);
            }
            godot_print!("scanline_count_per_polygon:");
            for (i, count) in scanline_count_per_polygon.as_slice().iter().enumerate() {
                godot_print!("  [{}] {}", i, count);
            }
            total_rows += 1;
        }
        let mut out = Dictionary::new();
        let _ = out.insert("prev_time", prev_time);
        let _ = out.insert("scroll_accum", noise_accum);
        let _ = out.insert("total_rows_scrolled", total_rows);
        let _ = out.insert("scanline_count_per_polygon", scanline_count_per_polygon);
        let _ = out.insert("collision_polygons", collision_polygons);
        out
    }

    #[func]
    pub fn compute_concave_collision_polygons(
        &self,
        raw_pixel_mask: PackedByteArray,
        image_width_pixels: i32,
        image_height_pixels: i32,
        tile_edge_length: i32,
    ) -> Array<PackedVector2Array> {
        // TODO: because godot complains about unsigned int r8 format, we just convert it here
        //  this is really gross to me and i think i could perhaps learn enough to argue for supporting
        //  R8_UINT in godot's RenderDevice.DataFormat <-> ImageFormat mapping in the source code.
        //  see: https://github.com/godotengine/godot/blob/6c9765d87e142e786f0190783f41a0250a835c99/servers/rendering/renderer_rd/storage_rd/texture_storage.cpp#L2281C1-L2664C1
        let pixel_data: Vec<u8> = raw_pixel_mask
            .to_vec()
            .into_iter()
            .map(|b| if b != 0 { 1 } else { 0 })
            .collect();
        let width: usize = image_width_pixels as usize;
        let height: usize = image_height_pixels as usize;
        let tile_size: usize = tile_edge_length as usize;
        let mut godot_polygons_array: Array<PackedVector2Array> = Array::new();
        let concave_polygons: Vec<Vec<Vector2>> =
            generate_concave_collision_polygons_pixel_perfect(&pixel_data, (width, height), tile_size);
        for concave_polygon in concave_polygons {
            let mut packed_polygon: PackedVector2Array = PackedVector2Array::new();
            for point in concave_polygon {
                packed_polygon.push(point);
            }
            godot_polygons_array.push(&packed_polygon);
        }
        godot_polygons_array
    }
}
