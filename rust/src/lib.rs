use crate::audio_analysis_util::{band_pass_filter, extract_onset_times};
use godot::builtin::{PackedByteArray, PackedVector2Array, Vector2};
use godot::classes::Node2D;
use godot::prelude::*;
mod audio_analysis_util;
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
    #[func]
    pub fn compute_concave_collision_polygons(
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

        let concave_polygons: Vec<Vec<Vector2>> =
            collision_mask_util::generate_concave_collision_polygons_pixel_perfect(
                &pixel_data,
                (width, height),
                tile_size,
            );

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
            collision_mask_util::generate_convex_collision_polygons_pixel_perfect(
                &pixel_data,
                (width, height),
                tile_size,
            );

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
    pub fn detect_bpm(&self, path: GString) -> f32 {
        audio_analysis_util::detect_bpm(path)
        //audio_analysis_util::_detect_bpm_accurate(path)
    }

    #[func]
    pub fn isolate_melody(path: GString, center_hz: f32) -> PackedFloat32Array {
        // 1) compute output filename
        let infile = path.to_string();
        let out_str = if infile.to_lowercase().ends_with(".wav") {
            format!("{}__isolated.wav", &infile[..infile.len() - 4])
        } else {
            format!("{}__isolated.wav", infile)
        };
        let out_g = GString::from(out_str.clone());

        // 2) isolate the synth band
        band_pass_filter(path.clone(), center_hz, out_g.clone());
        godot_print!(
            "test_melody_extraction: wrote isolated stem to '{}'",
            out_str
        );

        // 3) detect onsets in that isolated stem
        let onsets = extract_onset_times(out_g.clone());
        godot_print!("test_melody_extraction: detected {} onsets", onsets.len());

        // 4) return the array of onset times
        onsets
    }
}
