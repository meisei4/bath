pub mod audio_analysis;
pub mod collision_mask;
pub mod midi;
mod nodes;
pub mod render;
pub mod resource_paths;
mod sound_render;

use crate::audio_analysis::godot::{detect_bpm_aubio_ogg, detect_bpm_aubio_wav};
use crate::collision_mask::isp::{shift_polygon_vertices_down_by_pixels, update_polygons_with_scanline_alpha_buckets};
use crate::midi::godot::{
    make_note_on_off_event_dict_seconds, make_note_on_off_event_dict_ticks, write_samples_to_wav,
};
use crate::midi::util::{inject_program_change, prepare_events, process_midi_events_with_timing, render_sample_frame};
use collision_mask::godot::{
    generate_concave_collision_polygons_pixel_perfect, generate_convex_collision_polygons_pixel_perfect,
};
use godot::builtin::{PackedByteArray, PackedVector2Array, Vector2};
use godot::classes::file_access::ModeFlags;
use godot::classes::{FileAccess, Node2D};
use godot::global::godot_print;
use godot::prelude::{
    gdextension, godot_api, Array, Base, Dictionary, ExtensionLibrary, GString, GodotClass, PackedInt32Array,
};
use midly::{MidiMessage, Smf, TrackEventKind};
use rustysynth::{SoundFont, Synthesizer, SynthesizerSettings};
use std::io::Cursor;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}

#[derive(GodotClass)]
#[class(init, base=Node2D)]
struct RustUtil {
    #[base]
    base: Base<Node2D>,
}

const TARGET_CHANNEL: u8 = 0;
const PROGRAM: u8 = 0; //"Accordion" figure out a better way to do this

#[godot_api]
impl RustUtil {
    #[func]
    fn process_scanline_no_accum(
        i_time: f32,
        screen_h: f32,
        vel_y: f32,
        scanline_alpha_buckets: PackedVector2Array,
        mut collision_polygons: Array<PackedVector2Array>,
        mut scanline_count_per_polygon: PackedInt32Array,
        mut total_rows_scrolled: i32,
    ) -> Dictionary {
        let rows_per_second = vel_y * screen_h;
        let rows_now: i32 = (rows_per_second * i_time).floor() as i32;
        let rows_to_shift = rows_now.saturating_sub(total_rows_scrolled);
        if rows_to_shift > 0 {
            shift_polygon_vertices_down_by_pixels(
                &mut collision_polygons,
                scanline_count_per_polygon.as_slice(),
                rows_to_shift,
            );
            update_polygons_with_scanline_alpha_buckets(
                &mut collision_polygons,
                &scanline_alpha_buckets,
                &mut scanline_count_per_polygon,
            );
            total_rows_scrolled = rows_now;
        }
        let mut out = Dictionary::new();
        let _ = out.insert("collision_polygons", collision_polygons);
        let _ = out.insert("scanline_count_per_polygon", scanline_count_per_polygon);
        let _ = out.insert("total_rows_scrolled", total_rows_scrolled);
        out
    }

    #[func]
    fn process_scanline_closest_0(
        prev_time: f32,
        i_time: f32,
        screen_h: f32,
        vel_y: f32,
        depth: f32,
        global_coordinate_scale: f32,
        uniform_stretch_correction: f32,
        stretch_scalar_y: f32,
        parallax_near_scale: f32,
        scanline_alpha_buckets: PackedVector2Array,
        mut collision_polygons: Array<PackedVector2Array>,
        mut scroll_noise_accum: f32, // projected-noise accumulator (see below)
        mut scanline_count_per_polygon: PackedInt32Array,
        mut total_rows_scrolled: i32,
    ) -> Dictionary {
        let delta_time = i_time - prev_time;
        let a = 0.5 * ((depth + 1.0) / (depth - 1.0)).ln();
        let b = 1.5 * (depth * ((depth + 1.0) / (depth - 1.0)).ln() - 2.0);
        let m_pi_4: f32 = 0.7853981633974483;
        let rot_y: f32 = (-m_pi_4).cos();
        let s = uniform_stretch_correction * stretch_scalar_y * parallax_near_scale * rot_y;
        scroll_noise_accum += vel_y * global_coordinate_scale * s * delta_time;
        let mut current_row: i32 = total_rows_scrolled;
        loop {
            let y_norm = (2.0 * (current_row as f32 + 0.5) / screen_h) - 1.0;
            let scale_y = a + b * y_norm;
            let noise_units_per_pixel = scale_y * s;
            if scroll_noise_accum <= noise_units_per_pixel {
                break;
            }
            scroll_noise_accum -= noise_units_per_pixel;
            shift_polygon_vertices_down_by_pixels(&mut collision_polygons, scanline_count_per_polygon.as_slice(), 1);
            update_polygons_with_scanline_alpha_buckets(
                &mut collision_polygons,
                &scanline_alpha_buckets,
                &mut scanline_count_per_polygon,
            );
            current_row += 1;
            total_rows_scrolled += 1;
        }

        let mut out = Dictionary::new();
        let prev_time_c = i_time;
        let _ = out.insert("prev_time", prev_time_c);
        let _ = out.insert("scroll_accum", scroll_noise_accum);
        let _ = out.insert("collision_polygons", collision_polygons);
        let _ = out.insert("scanline_count_per_polygon", scanline_count_per_polygon);
        let _ = out.insert("total_rows_scrolled", total_rows_scrolled);
        out
    }

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
    fn process_scanline_close(
        delta_time: f32,
        screen_h: f32,
        vel_y: f32,
        depth: f32,
        global_coordinate_scale: f32,
        uniform_stretch_correction: f32,
        stretch_scalar_y: f32,
        parallax_near_scale: f32,
        scanline_alpha_buckets: PackedVector2Array,
        mut collision_polygons: Array<PackedVector2Array>,
        mut scroll_noise_accum: f32,
        mut scanline_count_per_polygon: PackedInt32Array,
        mut total_rows_scrolled: i32,
    ) -> Dictionary {
        let a = 0.5 * ((depth + 1.0) / (depth - 1.0)).ln();
        let b = 1.5 * (depth * ((depth + 1.0) / (depth - 1.0)).ln() - 2.0);
        let _stretch = uniform_stretch_correction * stretch_scalar_y * parallax_near_scale;
        scroll_noise_accum += vel_y * delta_time;
        let mut current_row: i32 = total_rows_scrolled;
        loop {
            let y_norm = (2.0 * (current_row as f32 + 0.5) / screen_h) - 1.0;
            let scale_y = a + b * y_norm;
            //let noise_units_per_pixel = (scale_y / global_coordinate_scale);
            const _ROT_Y_FACTOR: f32 = 0.70710678;
            //let noise_units_per_pixel = (scale_y / global_coordinate_scale) * _ROT_Y_FACTOR * stretch;
            let noise_units_per_pixel = scale_y / global_coordinate_scale;
            if scroll_noise_accum < noise_units_per_pixel {
                break;
            }
            scroll_noise_accum -= noise_units_per_pixel;
            shift_polygon_vertices_down_by_pixels(&mut collision_polygons, scanline_count_per_polygon.as_slice(), 1);
            update_polygons_with_scanline_alpha_buckets(
                &mut collision_polygons,
                &scanline_alpha_buckets,
                &mut scanline_count_per_polygon,
            );
            current_row += 1;
            total_rows_scrolled += 1;
        }
        let mut out = Dictionary::new();
        let _ = out.insert("scroll_accum", scroll_noise_accum);
        let _ = out.insert("scanline_count_per_polygon", scanline_count_per_polygon);
        let _ = out.insert("collision_polygons", collision_polygons);
        let _ = out.insert("total_rows_scrolled", total_rows_scrolled);
        out
    }

    #[func]
    fn process_scanline1(
        &self,
        delta_time: f32,
        screen_h: f32,
        vel_y: f32,
        depth: f32,
        global_coordinate_scale: f32,
        scanline_alpha_buckets: PackedVector2Array,
        mut collision_polygons: Array<PackedVector2Array>,
        mut scroll_accum: f32, // now noise-space accumulator
        mut scanline_count_per_polygon: PackedInt32Array,
    ) -> Dictionary {
        let a = 0.5 * ((depth + 1.0) / (depth - 1.0)).ln();
        let b = 1.5 * (depth * ((depth + 1.0) / (depth - 1.0)).ln() - 2.0);
        scroll_accum += vel_y * delta_time;
        let mut current_row: i32 = 0;
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let total_millis = now.as_millis();
        let hours = (total_millis / 1000 / 60 / 60) % 24;
        let minutes = (total_millis / 1000 / 60) % 60;
        let seconds = (total_millis / 1000) % 60;
        let millis = total_millis % 1000;
        godot_print!(
            "INFO [SYSTEM_TIME: {:02}:{:02}:{:02}.{:03}] scroll_noise_accum: {:.5}",
            hours,
            minutes,
            seconds,
            millis,
            scroll_accum
        );
        loop {
            let y_norm = (2.0 * (current_row as f32 + 0.5) / screen_h) - 1.0;
            let scale_y = a + b * y_norm;
            let noise_units_per_pixel = scale_y / global_coordinate_scale;

            if scroll_accum < noise_units_per_pixel {
                break;
            }
            scroll_accum -= noise_units_per_pixel;
            godot_print!(
                "row {} | y_norm: {:.5} | scale_y: {:.5} | npp: {:.5} | remaining: {:.5}",
                current_row,
                y_norm,
                scale_y,
                noise_units_per_pixel,
                scroll_accum
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
            current_row += 1;
        }
        let mut out = Dictionary::new();
        let _ = out.insert("scroll_accum", scroll_accum); // still returned under old name
        let _ = out.insert("scanline_count_per_polygon", scanline_count_per_polygon);
        let _ = out.insert("collision_polygons", collision_polygons);
        out
    }

    #[func]
    fn process_scanline_og(
        &self,
        delta_time: f32,
        screen_h: f32,
        vel_y: f32,
        depth: f32,
        scanline_alpha_buckets: PackedVector2Array,
        mut collision_polygons: Array<PackedVector2Array>,
        mut scroll_accum: f32,
        mut scanline_count_per_polygon: PackedInt32Array,
    ) -> Dictionary {
        let a = 0.5 * ((depth + 1.0) / (depth - 1.0)).ln();
        let b = 1.5 * (depth * ((depth + 1.0) / (depth - 1.0)).ln() - 2.0);
        let scale_y_top = a + b * -1.0;
        let speed_px_per_sec = vel_y * screen_h / (2.0 * scale_y_top);
        scroll_accum += speed_px_per_sec * delta_time;
        let rows = scroll_accum.floor() as i32;
        scroll_accum -= rows as f32;

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let total_millis = now.as_millis();
        let hours = (total_millis / 1000 / 60 / 60) % 24;
        let minutes = (total_millis / 1000 / 60) % 60;
        let seconds = (total_millis / 1000) % 60;
        let millis = total_millis % 1000;
        godot_print!(
            "INFO [SYSTEM_TIME: {:02}:{:02}:{:02}.{:03}] rows: {}, speed_px_per_sec: {}, delta_time: {:.3}, scroll_accum: {:.3}",
            hours,
            minutes,
            seconds,
            millis,
            rows,
            speed_px_per_sec,
            delta_time,
            scroll_accum
        );
        for row in 0..rows {
            godot_print!("row: {}", row);
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
        }

        let mut out = Dictionary::new();
        let _ = out.insert("scroll_accum", scroll_accum);
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

    #[func]
    pub fn compute_convex_collision_polygons(
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
        let mut godot_polygons_array: Array<PackedVector2Array> = Array::new();
        let convex_polygons: Vec<Vec<Vector2>> =
            generate_convex_collision_polygons_pixel_perfect(&pixel_data, (width, height), tile_size);

        for convex_polygon in convex_polygons {
            let mut packed_polygon: PackedVector2Array = PackedVector2Array::new();
            for point in convex_polygon {
                packed_polygon.push(point);
            }
            godot_polygons_array.push(&packed_polygon);
        }
        godot_polygons_array
    }

    #[func]
    pub fn detect_bpm_wav(&self, wav_file_path: GString) -> f32 {
        let wav_path = wav_file_path.to_string();
        let wav_file = FileAccess::open(&wav_path, ModeFlags::READ).unwrap();
        let wav_bytes = wav_file.get_buffer(wav_file.get_length() as i64).to_vec();
        detect_bpm_aubio_wav(&wav_bytes)
    }

    #[func]
    pub fn detect_bpm_ogg(&self, ogg_file_path: GString) -> f32 {
        let ogg_path = ogg_file_path.to_string();
        let ogg_file = FileAccess::open(&ogg_path, ModeFlags::READ).unwrap();
        let ogg_bytes = ogg_file.get_buffer(ogg_file.get_length() as i64).to_vec();
        detect_bpm_aubio_ogg(&ogg_bytes)
    }

    #[func]
    pub fn get_midi_note_on_off_event_buffer_ticks(&self, midi_file_path: GString) -> Dictionary {
        make_note_on_off_event_dict_ticks(&midi_file_path)
    }

    #[func]
    pub fn get_midi_note_on_off_event_buffer_seconds(&self, midi_file_path: GString) -> Dictionary {
        make_note_on_off_event_dict_seconds(&midi_file_path)
    }

    #[func]
    pub fn render_midi_to_sound_bytes_constant_time(
        &self,
        sample_rate: i32,
        midi_file_path: GString,
        sf2_file_path: GString,
    ) -> PackedByteArray {
        let sf2_path = sf2_file_path.to_string();
        let sf2_file = FileAccess::open(&sf2_path, ModeFlags::READ).unwrap();
        let sf2_bytes = sf2_file.get_buffer(sf2_file.get_length() as i64).to_vec();
        let mut sf2_cursor = Cursor::new(sf2_bytes);
        let soundfont = Arc::new(SoundFont::new(&mut sf2_cursor).unwrap());
        let mut synth = Synthesizer::new(&soundfont, &SynthesizerSettings::new(sample_rate)).unwrap();
        let midi_path = midi_file_path.to_string();
        let file = FileAccess::open(&midi_path, ModeFlags::READ).unwrap();
        let midi_file_bytes = file.get_buffer(file.get_length() as i64).to_vec();
        let smf = Smf::parse(&midi_file_bytes).unwrap();
        //TODO: make this more about the accordion, "program" is such a shitty name for an instrument in midi
        // i am not a fan of who ever made that naming decision, they better not be japanese
        let mut events = prepare_events(&smf);
        events = inject_program_change(events, TARGET_CHANNEL, PROGRAM);
        let mut audio = Vec::new();
        let mut active_notes = std::collections::HashSet::new();
        let mut time_cursor = 0.0;
        let step_secs = 1.0 / sample_rate as f64;
        process_midi_events_with_timing(events, &smf, |event_time, event, ch| {
            while time_cursor < event_time {
                audio.push(render_sample_frame(&mut synth));
                time_cursor += step_secs;
            }
            if let Some(channel) = ch {
                match event {
                    TrackEventKind::Midi { message, .. } => match message {
                        MidiMessage::NoteOn { key, vel } => {
                            let note = key.as_int() as i32;
                            let velocity = vel.as_int() as i32;
                            if velocity > 0 {
                                synth.note_on(channel as i32, note, velocity);
                                active_notes.insert((channel, note));
                            } else {
                                synth.note_off(channel as i32, note);
                                active_notes.remove(&(channel, note));
                            }
                        },
                        MidiMessage::NoteOff { key, .. } => {
                            let note = key.as_int() as i32;
                            synth.note_off(channel as i32, note);
                            active_notes.remove(&(channel, note));
                        },
                        _ => {},
                    },
                    _ => {},
                }
            }
        });
        while !active_notes.is_empty() {
            audio.push(render_sample_frame(&mut synth));
            time_cursor += step_secs;
        }
        write_samples_to_wav(sample_rate, audio)
        //TODO: look into vorbis later if needed, the rust support is very ugly with C libraries wrapped up and dragged in
    }
}
