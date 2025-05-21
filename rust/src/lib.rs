mod midi;
mod collision_mask;
mod audio_analysis;

use godot::builtin::{PackedByteArray, PackedVector2Array, Vector2};
use godot::classes::Node2D;
use godot::prelude::*;

use crate::midi::midi::parse_midi_events_into_note_on_off_event_buffer;
use audio_analysis::util::{band_pass_filter, detect_bpm, extract_onset_times};
use collision_mask::util::{
    generate_concave_collision_polygons_pixel_perfect,
    generate_convex_collision_polygons_pixel_perfect,
};


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
            generate_concave_collision_polygons_pixel_perfect(
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
            generate_convex_collision_polygons_pixel_perfect(
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
        detect_bpm(path)
        //TODO: this is not actually accurate just an alternative, look at offline vs realtime later
        //_detect_bpm_accurate(path)
    }

    #[func]
    pub fn isolate_melody(path: GString, center_hz: f32) -> PackedFloat32Array {
        let infile = path.to_string();
        let out_str = if infile.to_lowercase().ends_with(".wav") {
            format!("{}__isolated.wav", &infile[..infile.len() - 4])
        } else {
            format!("{}__isolated.wav", infile)
        };
        let out_g = GString::from(out_str.clone());
        band_pass_filter(path.clone(), center_hz, out_g.clone());
        godot_print!(
            "test_melody_extraction: wrote isolated stem to '{}'",
            out_str
        );
        let onsets = extract_onset_times(out_g.clone());
        godot_print!("test_melody_extraction: detected {} onsets", onsets.len());
        onsets
    }

    #[func]
    pub fn get_midi_event_buffer(&self, midi_path: GString) -> PackedVector4Array {
        let map = parse_midi_events_into_note_on_off_event_buffer(&midi_path.to_string());
        let mut buffer = PackedVector4Array::new();
        for (key, segments) in map {
            for (onset, release) in segments {
                buffer.push(Vector4::new(
                    onset as f32,
                    release as f32,
                    key.midi_note as f32,
                    key.instrument_id as f32,
                ));
            }
        }
        buffer
    }
}
