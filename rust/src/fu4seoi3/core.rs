use crate::fu4seoi3::draw::*;
use raylib::consts::CameraProjection::CAMERA_ORTHOGRAPHIC;
use raylib::math::glam::Mat4;
use raylib::prelude::*;
use std::f32::consts::{PI, TAU};
use std::fs;
use std::mem::size_of;
use std::ops::{Add, Sub};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn timestamp() -> String {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64();
    let secs = now as u64;
    let millis = ((now - secs as f64) * 1000.0) as u32;
    let hours = (secs / 3600) % 24;
    let mins = (secs / 60) % 60;
    let s = secs % 60;
    format!("[{:02}:{:02}:{:02}.{:03}]", hours, mins, s, millis)
}

pub const RES_SCALE: f32 = 1.5;
pub const DC_WIDTH_BASE: f32 = 640.0;
pub const DC_HEIGHT_BASE: f32 = 480.0;
pub const DC_WIDTH: i32 = (DC_WIDTH_BASE * RES_SCALE) as i32;
pub const DC_HEIGHT: i32 = (DC_HEIGHT_BASE * RES_SCALE) as i32;

pub const MODEL_SCALE: Vector3 = Vector3::new(1.0, 1.0, 1.0);
pub const MODEL_POS: Vector3 = Vector3::ZERO;
pub const MAIN_POS: Vector3 = Vector3::new(0.0, 0.0, 2.0);
pub const JUGEMU_POS_ISO: Vector3 = Vector3::new(1.0, 1.0, 1.0);
pub const Y_AXIS: Vector3 = Vector3::new(0.0, 1.0, 0.0);

pub const JUGEMU_DISTANCE_ORTHO: f32 = 6.5;
pub const JUGEMU_DISTANCE_PERSPECTIVE: f32 = 9.0;

pub const FOVY_PERSPECTIVE: f32 = 50.0;
pub const FOVY_ORTHOGRAPHIC: f32 = 9.0;

pub const BLEND_SCALAR: f32 = 5.0;
pub const PLACEMENT_ANIM_DUR_SECONDS: f32 = 0.25;
pub const HINT_SCALE: f32 = 0.66;
pub const HINT_SCALE_VEC: Vector3 = Vector3::new(HINT_SCALE, HINT_SCALE, HINT_SCALE);

//TODO: PLEASE ALSO PUT THESE INTO the field config for now
pub const ROOM_W: i32 = 9;
pub const ROOM_H: i32 = 3;
pub const ROOM_D: i32 = 9;

//TODO: PLEASE PUT THESE INTO the field config for the deformation stuff
pub const RADIAL_FIELD_SIZE: usize = 64;
pub const UMBRAL_MASK_OUTER_RADIUS: f32 = 0.40;
pub const UMBRAL_MASK_CENTER: Vector2 = Vector2::new(0.5, 0.5);

pub const JET_STRENGTH: f32 = 10.0;
pub const JET_SPREAD_ANGLE: f32 = 14.0;
pub const JET_MAX_DISTANCE: f32 = 18.0;
pub const FUNNEL_STRENGTH: f32 = 2.0;
pub const FUNNEL_REACH: f32 = 6.0;
pub const FUNNEL_CATCH_RADIUS: f32 = 4.0;
pub const FUNNEL_SINK_RADIUS: f32 = 0.7;
pub const FUNNEL_CURVE_POWER: f32 = 1.5;
pub const WALL_REDIRECT_STRENGTH: f32 = 0.8;
pub const WALL_REDIRECT_DISTANCE: f32 = 5.0;
pub const CHI_SAMPLE_HEIGHT: f32 = 0.2;
pub const CHI_ARROW_LENGTH: f32 = 0.4;

pub const ROTATION_FREQUENCY_HZ: f32 = 0.2;
pub const DEFORMATION_CYCLES_PER_ROTATION: f32 = 1.0;
pub const ROTATIONAL_SAMPLES_FOR_INV_PROJ: usize = 40;
pub const WAVE_CYCLES_SLOW: f32 = 7.0;
pub const WAVE_CYCLES_FAST: f32 = 255.0;
pub const WAVE_AMPLITUDE_X: f32 = 0.0;
pub const WAVE_AMPLITUDE_Y: f32 = 0.1;

pub fn near_plane_height_orthographic(view_config: &ViewConfig) -> f32 {
    2.0 * (view_config.fovy_perspective * 0.5).to_radians().tan()
}

pub fn angular_velocity(field_config: &FieldConfig) -> f32 {
    TAU * field_config.rotation_frequency_hz
}

macro_rules! parse_config_fields {
    ($content:expr, $config:expr, {
        $( $key:literal => $field:ident : $type:ty ),* $(,)?
    }) => {
        for line in $content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() != 2 {
                continue;
            }
            let key = parts[0];

            match key {
                $(
                    $key => {
                        if let Ok(value) = parts[1].parse::<$type>() {
                            $config.$field = value;
                        }
                    }
                )*
                _ => {}
            }
        }
    };
}

#[derive(Clone, Debug)]
pub struct ViewConfig {
    pub jugemu_distance_ortho: f32,
    pub jugemu_distance_perspective: f32,
    pub fovy_perspective: f32,
    pub fovy_orthographic: f32,
    pub blend_scalar: f32,
    pub placement_anim_dur_seconds: f32,
    pub hint_scale: f32,
}

impl Default for ViewConfig {
    fn default() -> Self {
        Self {
            jugemu_distance_ortho: JUGEMU_DISTANCE_ORTHO,
            jugemu_distance_perspective: JUGEMU_DISTANCE_PERSPECTIVE,
            fovy_perspective: FOVY_PERSPECTIVE,
            fovy_orthographic: FOVY_ORTHOGRAPHIC,
            blend_scalar: BLEND_SCALAR,
            placement_anim_dur_seconds: PLACEMENT_ANIM_DUR_SECONDS,
            hint_scale: HINT_SCALE,
        }
    }
}

impl ViewConfig {
    pub fn load_from_file(path: &str) -> Self {
        let mut cfg = ViewConfig::default();
        if let Ok(content) = fs::read_to_string(path) {
            parse_config_fields!(content, cfg, {
                "JUGEMU_DISTANCE_ORTHO" => jugemu_distance_ortho: f32,
                "JUGEMU_DISTANCE_PERSPECTIVE" => jugemu_distance_perspective: f32,
                "FOVY_PERSPECTIVE" => fovy_perspective: f32,
                "FOVY_ORTHOGRAPHIC" => fovy_orthographic: f32,
                "BLEND_SCALAR" => blend_scalar: f32,
                "PLACEMENT_ANIM_DUR_SECONDS" => placement_anim_dur_seconds: f32,
                "HINT_SCALE" => hint_scale: f32,
            });
        }
        cfg
    }

    pub fn log_delta(&self, old: &ViewConfig) {
        let mut changed = Vec::new();

        if self.jugemu_distance_ortho != old.jugemu_distance_ortho {
            changed.push(format!(
                "JUGEMU_DISTANCE_ORTHO: {} → {}",
                old.jugemu_distance_ortho, self.jugemu_distance_ortho
            ));
        }
        if self.jugemu_distance_perspective != old.jugemu_distance_perspective {
            changed.push(format!(
                "JUGEMU_DISTANCE_PERSPECTIVE: {} → {}",
                old.jugemu_distance_perspective, self.jugemu_distance_perspective
            ));
        }
        if self.fovy_perspective != old.fovy_perspective {
            changed.push(format!(
                "FOVY_PERSPECTIVE: {} → {}",
                old.fovy_perspective, self.fovy_perspective
            ));
        }
        if self.fovy_orthographic != old.fovy_orthographic {
            changed.push(format!(
                "FOVY_ORTHOGRAPHIC: {} → {}",
                old.fovy_orthographic, self.fovy_orthographic
            ));
        }
        if self.blend_scalar != old.blend_scalar {
            changed.push(format!("BLEND_SCALAR: {} → {}", old.blend_scalar, self.blend_scalar));
        }
        if self.placement_anim_dur_seconds != old.placement_anim_dur_seconds {
            changed.push(format!(
                "PLACEMENT_ANIM_DUR_SECONDS: {} → {}",
                old.placement_anim_dur_seconds, self.placement_anim_dur_seconds
            ));
        }
        if self.hint_scale != old.hint_scale {
            changed.push(format!("HINT_SCALE: {} → {}", old.hint_scale, self.hint_scale));
        }

        if !changed.is_empty() {
            println!("{} ViewConfig changed: {}", timestamp(), changed.join(", "));
        }
    }
}

#[derive(Clone, Debug)]
pub struct FieldConfig {
    pub jet_strength: f32,
    pub jet_spread_angle: f32,
    pub jet_max_distance: f32,
    pub funnel_strength: f32,
    pub funnel_reach: f32,
    pub funnel_catch_radius: f32,
    pub funnel_sink_radius: f32,
    pub funnel_curve_power: f32,
    pub wall_redirect_strength: f32,
    pub wall_redirect_distance: f32,

    pub chi_sample_height: f32,
    pub chi_arrow_length: f32,

    pub rotation_frequency_hz: f32,
    pub deformation_cycles_per_rotation: f32,
    pub rotational_samples_for_inv_proj: usize,
    pub wave_cycles_slow: f32,
    pub wave_cycles_fast: f32,
    pub wave_amplitude_x: f32,
    pub wave_amplitude_y: f32,
}

impl Default for FieldConfig {
    fn default() -> Self {
        Self {
            jet_strength: JET_STRENGTH,
            jet_spread_angle: JET_SPREAD_ANGLE,
            jet_max_distance: JET_MAX_DISTANCE,
            funnel_strength: FUNNEL_STRENGTH,
            funnel_reach: FUNNEL_REACH,
            funnel_catch_radius: FUNNEL_CATCH_RADIUS,
            funnel_sink_radius: FUNNEL_SINK_RADIUS,
            funnel_curve_power: FUNNEL_CURVE_POWER,
            wall_redirect_strength: WALL_REDIRECT_STRENGTH,
            wall_redirect_distance: WALL_REDIRECT_DISTANCE,
            chi_sample_height: CHI_SAMPLE_HEIGHT,
            chi_arrow_length: CHI_ARROW_LENGTH,

            rotation_frequency_hz: ROTATION_FREQUENCY_HZ,
            deformation_cycles_per_rotation: DEFORMATION_CYCLES_PER_ROTATION,
            wave_cycles_slow: WAVE_CYCLES_SLOW,
            wave_cycles_fast: WAVE_CYCLES_FAST,
            wave_amplitude_x: WAVE_AMPLITUDE_X,
            wave_amplitude_y: WAVE_AMPLITUDE_Y,
            rotational_samples_for_inv_proj: ROTATIONAL_SAMPLES_FOR_INV_PROJ,
        }
    }
}

impl FieldConfig {
    pub fn load_from_file(path: &str) -> Self {
        let mut config = FieldConfig::default();
        if let Ok(content) = fs::read_to_string(path) {
            parse_config_fields!(content, config, {
                "JET_STRENGTH" => jet_strength: f32,
                "JET_SPREAD_ANGLE" => jet_spread_angle: f32,
                "JET_MAX_DISTANCE" => jet_max_distance: f32,
                "FUNNEL_STRENGTH" => funnel_strength: f32,
                "FUNNEL_REACH" => funnel_reach: f32,
                "FUNNEL_CATCH_RADIUS" => funnel_catch_radius: f32,
                "FUNNEL_SINK_RADIUS" => funnel_sink_radius: f32,
                "FUNNEL_CURVE_POWER" => funnel_curve_power: f32,
                "WALL_REDIRECT_STRENGTH" => wall_redirect_strength: f32,
                "WALL_REDIRECT_DISTANCE" => wall_redirect_distance: f32,
                "CHI_SAMPLE_HEIGHT" => chi_sample_height: f32,
                "CHI_ARROW_LENGTH" => chi_arrow_length: f32,

                "ROTATION_FREQUENCY_HZ" => rotation_frequency_hz: f32,
                "DEFORMATION_CYCLES_PER_ROTATION" => deformation_cycles_per_rotation: f32,
                "ROTATIONAL_SAMPLES_FOR_INV_PROJ" => rotational_samples_for_inv_proj: usize,
                "WAVE_CYCLES_SLOW" => wave_cycles_slow: f32,
                "WAVE_CYCLES_FAST" => wave_cycles_fast: f32,
                "WAVE_AMPLITUDE_X" => wave_amplitude_x: f32,
                "WAVE_AMPLITUDE_Y" => wave_amplitude_y: f32,
            });
        }
        if config.rotational_samples_for_inv_proj == 0 {
            eprintln!(
                "{} WARNING: ROTATIONAL_SAMPLES_FOR_INV_PROJ cannot be 0, using default",
                timestamp()
            );
            config.rotational_samples_for_inv_proj = ROTATIONAL_SAMPLES_FOR_INV_PROJ;
        }
        if config.rotation_frequency_hz <= 0.0 {
            eprintln!(
                "{} WARNING: ROTATION_FREQUENCY_HZ must be > 0, using default",
                timestamp()
            );
            config.rotation_frequency_hz = ROTATION_FREQUENCY_HZ;
        }
        config
    }

    //TODO: I still dont like how this returns boolean for needs regeneration but i cant think of cleaner way yet and its still intermediate design
    pub fn log_delta(&self, old: &FieldConfig) -> bool {
        let mut changed = Vec::new();
        let mut needs_regeneration = false;

        if self.jet_strength != old.jet_strength {
            changed.push(format!("JET_STRENGTH: {} → {}", old.jet_strength, self.jet_strength));
        }
        if self.jet_spread_angle != old.jet_spread_angle {
            changed.push(format!(
                "JET_SPREAD_ANGLE: {} → {}",
                old.jet_spread_angle, self.jet_spread_angle
            ));
        }
        if self.jet_max_distance != old.jet_max_distance {
            changed.push(format!(
                "JET_MAX_DISTANCE: {} → {}",
                old.jet_max_distance, self.jet_max_distance
            ));
        }
        if self.funnel_strength != old.funnel_strength {
            changed.push(format!(
                "FUNNEL_STRENGTH: {} → {}",
                old.funnel_strength, self.funnel_strength
            ));
        }
        if self.funnel_reach != old.funnel_reach {
            changed.push(format!("FUNNEL_REACH: {} → {}", old.funnel_reach, self.funnel_reach));
        }
        if self.funnel_catch_radius != old.funnel_catch_radius {
            changed.push(format!(
                "FUNNEL_CATCH_RADIUS: {} → {}",
                old.funnel_catch_radius, self.funnel_catch_radius
            ));
        }
        if self.funnel_sink_radius != old.funnel_sink_radius {
            changed.push(format!(
                "FUNNEL_SINK_RADIUS: {} → {}",
                old.funnel_sink_radius, self.funnel_sink_radius
            ));
        }
        if self.funnel_curve_power != old.funnel_curve_power {
            changed.push(format!(
                "FUNNEL_CURVE_POWER: {} → {}",
                old.funnel_curve_power, self.funnel_curve_power
            ));
        }
        if self.wall_redirect_strength != old.wall_redirect_strength {
            changed.push(format!(
                "WALL_REDIRECT_STRENGTH: {} → {}",
                old.wall_redirect_strength, self.wall_redirect_strength
            ));
        }
        if self.wall_redirect_distance != old.wall_redirect_distance {
            changed.push(format!(
                "WALL_REDIRECT_DISTANCE: {} → {}",
                old.wall_redirect_distance, self.wall_redirect_distance
            ));
        }
        if self.chi_sample_height != old.chi_sample_height {
            changed.push(format!(
                "CHI_SAMPLE_HEIGHT: {} → {}",
                old.chi_sample_height, self.chi_sample_height
            ));
        }
        if self.chi_arrow_length != old.chi_arrow_length {
            changed.push(format!(
                "CHI_ARROW_LENGTH: {} → {}",
                old.chi_arrow_length, self.chi_arrow_length
            ));
        }

        if self.rotation_frequency_hz != old.rotation_frequency_hz {
            changed.push(format!(
                "ROTATION_FREQUENCY_HZ: {} → {} [REGEN]",
                old.rotation_frequency_hz, self.rotation_frequency_hz
            ));
            needs_regeneration = true;
        }
        if self.deformation_cycles_per_rotation != old.deformation_cycles_per_rotation {
            changed.push(format!(
                "DEFORMATION_CYCLES_PER_ROTATION: {} → {} [REGEN]",
                old.deformation_cycles_per_rotation, self.deformation_cycles_per_rotation
            ));
            needs_regeneration = true;
        }
        if self.wave_cycles_slow != old.wave_cycles_slow {
            changed.push(format!(
                "WAVE_CYCLES_SLOW: {} → {} [REGEN]",
                old.wave_cycles_slow, self.wave_cycles_slow
            ));
            needs_regeneration = true;
        }
        if self.wave_cycles_fast != old.wave_cycles_fast {
            changed.push(format!(
                "WAVE_CYCLES_FAST: {} → {} [REGEN]",
                old.wave_cycles_fast, self.wave_cycles_fast
            ));
            needs_regeneration = true;
        }
        if self.wave_amplitude_x != old.wave_amplitude_x {
            changed.push(format!(
                "WAVE_AMPLITUDE_X: {} → {} [REGEN]",
                old.wave_amplitude_x, self.wave_amplitude_x
            ));
            needs_regeneration = true;
        }
        if self.wave_amplitude_y != old.wave_amplitude_y {
            changed.push(format!(
                "WAVE_AMPLITUDE_Y: {} → {} [REGEN]",
                old.wave_amplitude_y, self.wave_amplitude_y
            ));
            needs_regeneration = true;
        }
        if self.rotational_samples_for_inv_proj != old.rotational_samples_for_inv_proj {
            changed.push(format!(
                "ROTATIONAL_SAMPLES_FOR_INV_PROJ: {} → {} [REGEN]",
                old.rotational_samples_for_inv_proj, self.rotational_samples_for_inv_proj
            ));
            needs_regeneration = true;
        }

        if !changed.is_empty() {
            println!("{} FieldConfig changed: {}", timestamp(), changed.join(", "));
        }

        needs_regeneration
    }
}

pub struct ConfigWatcher<T> {
    path: String,
    last_modified: Option<SystemTime>,
    loader: fn(&str) -> T,
}

impl<T> ConfigWatcher<T> {
    pub fn new(path: &str, loader: fn(&str) -> T) -> Self {
        Self {
            path: path.to_string(),
            last_modified: None,
            loader,
        }
    }

    pub fn check_reload(&mut self) -> Option<T> {
        if let Ok(metadata) = fs::metadata(&self.path) {
            if let Ok(modified) = metadata.modified() {
                if self.last_modified.is_none() || Some(modified) != self.last_modified {
                    self.last_modified = Some(modified);
                    return Some((self.loader)(&self.path));
                }
            }
        }
        None
    }
}

pub struct ViewState {
    pub ndc_space: bool,
    pub aspect_correct: bool,
    pub paused: bool,
    pub color_mode: bool,
    pub texture_mode: bool,
    pub jugemu_mode: bool,
    pub ortho_mode: bool,
    pub jugemu_ortho_mode: bool,
    pub target_mesh_index: usize,
    pub space_blend: f32,
    pub aspect_blend: f32,
    pub ortho_blend: f32,
    pub jugemu_zoom: JugemuState,
}

impl ViewState {
    pub fn new() -> Self {
        Self {
            ndc_space: false,
            aspect_correct: true,
            paused: false,
            color_mode: true,
            texture_mode: false,
            jugemu_mode: true,
            ortho_mode: false,
            jugemu_ortho_mode: true,
            target_mesh_index: 1,
            space_blend: 0.0,
            aspect_blend: 1.0,
            ortho_blend: 0.0,
            jugemu_zoom: JugemuState::default(),
        }
    }
}

pub struct JugemuState {
    pub fovy_ortho: f32,
    pub fovy_perspective: f32,
    pub distance_ortho: f32,
    pub distance_perspective: f32,
}

impl Default for JugemuState {
    fn default() -> Self {
        Self {
            fovy_ortho: 9.0,
            fovy_perspective: 50.0,
            distance_ortho: 6.5,
            distance_perspective: 9.0,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum FieldDisrupter {
    Base,
    DoorPrimary,
    Window,
    BackWall,
}

pub struct FieldSample {
    pub position: Vector3,
    pub direction: Vector2,
    pub magnitude: f32,
    pub dominant: FieldDisrupter,
}

pub struct HoverState {
    pub indices: Option<(i32, i32, i32)>,
    pub center: Option<Vector3>,
    pub placed_cell_index: Option<usize>,
}

impl HoverState {
    pub fn is_occupied(&self) -> bool {
        self.placed_cell_index.is_some()
    }
}

pub fn compute_hover_state(
    handle: &RaylibHandle,
    camera: &Camera3D,
    room: &Room,
    placed_cells: &[PlacedCell],
) -> HoverState {
    let mouse = handle.get_mouse_position();
    let ray = handle.get_screen_to_world_ray(mouse, *camera);
    if let Some((ix, iy, iz)) = room.select_floor_cell(ray) {
        let center = room.cell_center(ix, iy, iz);
        let placed_cell_index = placed_cells.iter().position(|c| c.ix == ix && c.iy == iy && c.iz == iz);
        HoverState {
            indices: Some((ix, iy, iz)),
            center: Some(center),
            placed_cell_index,
        }
    } else {
        HoverState {
            indices: None,
            center: None,
            placed_cell_index: None,
        }
    }
}

#[derive(Clone)]
pub struct PlacedCell {
    pub ix: i32,
    pub iy: i32,
    pub iz: i32,
    pub mesh_index: usize,
    pub placed_time: f32,
    pub settled: bool,
    pub texture_enabled: bool,
    pub color_enabled: bool,
}

impl PlacedCell {
    pub fn age_at(&self, now: f32) -> f32 {
        now - self.placed_time
    }

    pub fn scale_at(&self, now: f32, view_cfg: &ViewConfig) -> f32 {
        let t = (self.age_at(now) / view_cfg.placement_anim_dur_seconds).clamp(0.0, 1.0);
        lerp(view_cfg.hint_scale, 1.0, t)
    }

    pub fn is_filled(&self) -> bool {
        self.settled && (self.color_enabled || self.texture_enabled)
    }
}

pub enum EditStack {
    PlaceCell {
        cell: PlacedCell, // TODO: this is confusing name placing a cell with a PlacedCell? figure this out better
        time: f32,
    },
    RemoveCell {
        cell: PlacedCell,
        time: f32,
    },
    ToggleTexture {
        ix: i32,
        iy: i32,
        iz: i32,
        time: f32,
    },
    ToggleColor {
        ix: i32,
        iy: i32,
        iz: i32,
        time: f32,
    },
}

fn find_cell_mut<'a>(placed_cells: &'a mut [PlacedCell], ix: i32, iy: i32, iz: i32) -> Option<&'a mut PlacedCell> {
    placed_cells.iter_mut().find(|c| c.ix == ix && c.iy == iy && c.iz == iz)
}

fn remove_cell_at(placed_cells: &mut Vec<PlacedCell>, ix: i32, iy: i32, iz: i32) {
    if let Some(idx) = placed_cells.iter().position(|c| c.ix == ix && c.iy == iy && c.iz == iz) {
        placed_cells.remove(idx);
    }
}
pub fn redo(edit: &EditStack, placed_cells: &mut Vec<PlacedCell>) {
    match edit {
        EditStack::PlaceCell { cell, .. } => {
            if placed_cells
                .iter()
                .all(|c| c.ix != cell.ix || c.iy != cell.iy || c.iz != cell.iz)
            {
                placed_cells.push(cell.clone());
            }
        },
        EditStack::RemoveCell { cell, .. } => {
            remove_cell_at(placed_cells, cell.ix, cell.iy, cell.iz);
        },
        EditStack::ToggleTexture { ix, iy, iz, .. } => {
            if let Some(c) = find_cell_mut(placed_cells, *ix, *iy, *iz) {
                c.texture_enabled = !c.texture_enabled;
            }
        },
        EditStack::ToggleColor { ix, iy, iz, .. } => {
            if let Some(c) = find_cell_mut(placed_cells, *ix, *iy, *iz) {
                c.color_enabled = !c.color_enabled;
            }
        },
    }
}

pub fn undo(edit: &EditStack, placed_cells: &mut Vec<PlacedCell>) {
    match edit {
        EditStack::PlaceCell { cell, .. } => {
            remove_cell_at(placed_cells, cell.ix, cell.iy, cell.iz);
        },
        EditStack::RemoveCell { cell, .. } => {
            if placed_cells
                .iter()
                .all(|c| c.ix != cell.ix || c.iy != cell.iy || c.iz != cell.iz)
            {
                placed_cells.push(cell.clone());
            }
        },
        EditStack::ToggleTexture { ix, iy, iz, .. } => {
            if let Some(c) = find_cell_mut(placed_cells, *ix, *iy, *iz) {
                c.texture_enabled = !c.texture_enabled;
            }
        },
        EditStack::ToggleColor { ix, iy, iz, .. } => {
            if let Some(c) = find_cell_mut(placed_cells, *ix, *iy, *iz) {
                c.color_enabled = !c.color_enabled;
            }
        },
    }
}

fn edit_time(edit: &EditStack) -> f32 {
    match edit {
        EditStack::PlaceCell { time, .. } => *time,
        EditStack::RemoveCell { time, .. } => *time,
        EditStack::ToggleTexture { time, .. } => *time,
        EditStack::ToggleColor { time, .. } => *time,
    }
}

fn edit_name(edit: &EditStack) -> &'static str {
    match edit {
        EditStack::PlaceCell { .. } => "PlaceCell",
        EditStack::RemoveCell { .. } => "RemoveCell",
        EditStack::ToggleTexture { .. } => "ToggleTexture",
        EditStack::ToggleColor { .. } => "ToggleColor",
    }
}

fn edit_coords(edit: &EditStack) -> (i32, i32, i32) {
    match edit {
        EditStack::PlaceCell { cell, .. } => (cell.ix, cell.iy, cell.iz),
        EditStack::RemoveCell { cell, .. } => (cell.ix, cell.iy, cell.iz),
        EditStack::ToggleTexture { ix, iy, iz, .. } => (*ix, *iy, *iz),
        EditStack::ToggleColor { ix, iy, iz, .. } => (*ix, *iy, *iz),
    }
}

pub fn log_edit_stack(edit_stack: &[EditStack], edit_cursor: usize) {
    println!("\nEDIT STACK");
    println!("cursor at index {}", edit_cursor);
    println!("APPLIED (0..{}):", edit_cursor);
    for (i, edit) in edit_stack.iter().enumerate().take(edit_cursor) {
        let t = edit_time(edit);
        let name = edit_name(edit);
        let (ix, iy, iz) = edit_coords(edit);
        println!("  [{:02}] t={:7.3}  {:13} @ ({:2},{:2},{:2})", i, t, name, ix, iy, iz);
    }

    println!("REDO   ({}..{}):", edit_cursor, edit_stack.len());
    for (i, edit) in edit_stack.iter().enumerate().skip(edit_cursor) {
        let t = edit_time(edit);
        let name = edit_name(edit);
        let (ix, iy, iz) = edit_coords(edit);
        println!("  [{:02}] t={:7.3}  {:13} @ ({:2},{:2},{:2})", i, t, name, ix, iy, iz);
    }
}

#[derive(Clone, Copy)]
pub enum OpeningKind {
    Door { primary: bool },
    Window,
}

#[derive(Clone, Copy)]
pub struct Opening {
    pub p0: Vector3,
    pub p1: Vector3,
    pub h0: f32,
    pub kind: OpeningKind,
    pub model_index: Option<usize>,
}

impl Opening {
    pub fn center(&self) -> Vector3 {
        Vector3::new(
            (self.p0.x + self.p1.x) * 0.5,
            (self.p0.y + self.p1.y) * 0.5,
            (self.p0.z + self.p1.z) * 0.5,
        )
    }

    pub fn normal(&self) -> Vector2 {
        let dx = self.p1.x - self.p0.x;
        let dz = self.p1.z - self.p0.z;
        Vector2::new(dz, -dx).normalize_or_zero()
    }

    pub fn tangent(&self) -> Vector2 {
        let dx = self.p1.x - self.p0.x;
        let dz = self.p1.z - self.p0.z;
        Vector2::new(dx, dz).normalize_or_zero()
    }

    pub fn width(&self) -> f32 {
        let dx = self.p1.x - self.p0.x;
        let dz = self.p1.z - self.p0.z;
        (dx * dx + dz * dz).sqrt()
    }

    pub fn position(&self, room: &Room) -> Vector3 {
        let mid = self.center();
        match self.kind {
            //TODO: consolidate the h0 and the concepts of model "heart" (i.e. model coords 0,0,0 has meaning beyond just logic for placement in world space)
            // (hikite makes sense for fusuma but deriving for others should help solidify the positioning on the walls i i think
            OpeningKind::Door { .. } => Vector3::new(mid.x, self.p0.y + self.h0, mid.z),
            OpeningKind::Window => Vector3::new(mid.x, self.p0.y + room.h as f32 * 0.5, mid.z),
        }
    }

    pub fn rotation_into_room(&self, room: &Room) -> f32 {
        let center = self.center();
        let north_z = room.origin.z + room.d as f32;
        let south_z = room.origin.z;
        let east_x = room.origin.x + room.w as f32;
        let west_x = room.origin.x;

        let north_dist = (center.z - north_z).abs();
        let south_dist = (center.z - south_z).abs();
        let east_dist = (center.x - east_x).abs();
        let west_dist = (center.x - west_x).abs();

        let mut wall = 1;
        let mut d_min = north_dist;

        if south_dist < d_min {
            d_min = south_dist;
            wall = 2;
        }
        if east_dist < d_min {
            d_min = east_dist;
            wall = 3;
        }
        if west_dist < d_min {
            d_min = west_dist;
            wall = 4;
        }

        match wall {
            1 => 180.0, // north: +Z -> -Z
            2 => 0.0,   // south: +Z -> +Z
            3 => -90.0, // east:  +Z -> -X
            4 => 90.0,  // west:  +Z -> +X
            _ => 0.0,
        }
    }
}

pub struct Room {
    pub w: i32,
    pub h: i32,
    pub d: i32,
    pub origin: Vector3,
    pub openings: Vec<Opening>,
    pub field_samples: Vec<FieldSample>,
    pub config: FieldConfig,
}

impl Default for Room {
    fn default() -> Self {
        let origin = Vector3::new(-(ROOM_W as f32) / 2.0, -(ROOM_H as f32) / 2.0, -(ROOM_D as f32) / 2.0);
        let center_x = origin.x + ROOM_W as f32 * 0.5;
        let north_z = origin.z + ROOM_D as f32;
        let primary_door = Opening {
            p0: Vector3::new(center_x - 1.0, origin.y, north_z),
            p1: Vector3::new(center_x + 1.0, origin.y, north_z),
            h0: 0.0,
            kind: OpeningKind::Door { primary: true },
            model_index: None,
        };
        let west_x = origin.x;
        let center_z = origin.z + ROOM_D as f32 * 0.5;
        let window = Opening {
            p0: Vector3::new(west_x, origin.y, center_z - 1.5),
            p1: Vector3::new(west_x, origin.y, center_z + 1.5),
            h0: 0.0,
            kind: OpeningKind::Window,
            model_index: None,
        };
        let mut room = Room {
            w: ROOM_W,
            h: ROOM_H,
            d: ROOM_D,
            origin,
            openings: vec![primary_door, window],
            field_samples: Vec::new(),
            config: FieldConfig::default(),
        };
        room.generate_chi_field();
        room
    }
}

impl Room {
    pub fn for_each_cell(&self, mut f: impl FnMut(i32, i32, i32, Vector3)) {
        for iy in 0..self.h {
            for iz in 0..self.d {
                for ix in 0..self.w {
                    let center = self.cell_center(ix, iy, iz);
                    f(ix, iy, iz, center);
                }
            }
        }
    }

    pub fn primary_door(&self) -> &Opening {
        self.openings
            .iter()
            .find(|o| matches!(o.kind, OpeningKind::Door { primary: true }))
            .expect("Room must have a primary door")
    }

    pub fn primary_window(&self) -> Option<&Opening> {
        self.openings.iter().find(|o| matches!(o.kind, OpeningKind::Window))
    }

    #[inline]
    pub fn cell_center(&self, ix: i32, iy: i32, iz: i32) -> Vector3 {
        Vector3::new(
            self.origin.x + ix as f32 + 0.5,
            self.origin.y + iy as f32 + 0.5,
            self.origin.z + iz as f32 + 0.5,
        )
    }

    pub fn top_right_front_corner(&self, ix: i32, iy: i32, iz: i32, camera: &Camera3D) -> Vector3 {
        let center = self.cell_center(ix, iy, iz);
        let half = 0.5_f32;
        let offsets = [
            Vector3::new(-half, -half, -half),
            Vector3::new(-half, -half, half),
            Vector3::new(-half, half, -half),
            Vector3::new(-half, half, half),
            Vector3::new(half, -half, -half),
            Vector3::new(half, -half, half),
            Vector3::new(half, half, -half),
            Vector3::new(half, half, half),
        ];
        let (depth, right, up) = basis_vector(&camera);
        let cam_pos = camera.position;
        fn to_camera_space(p: Vector3, cam_pos: Vector3, right: Vector3, up: Vector3, depth: Vector3) -> Vector3 {
            let v = p.sub(cam_pos);
            Vector3::new(v.dot(right), v.dot(up), v.dot(depth))
        }
        let mut best_world = center.add(offsets[0]);
        let mut best_cam = to_camera_space(best_world, cam_pos, right, up, depth);
        let eps = 1e-4_f32;
        for &offset in offsets.iter().skip(1) {
            let world = center.add(offset);
            let cam = to_camera_space(world, cam_pos, right, up, depth);
            let better = cam.x > best_cam.x + eps
                || ((cam.x - best_cam.x).abs() <= eps && cam.y > best_cam.y + eps)
                || ((cam.x - best_cam.x).abs() <= eps && (cam.y - best_cam.y).abs() <= eps && cam.z < best_cam.z - eps);
            if better {
                best_cam = cam;
                best_world = world;
            }
        }
        best_world
    }

    pub fn select_floor_cell(&self, ray: Ray) -> Option<(i32, i32, i32)> {
        if ray.direction.y.abs() < 1e-5 {
            return None;
        }
        let floor_y = self.origin.y;
        let t = (floor_y - ray.position.y) / ray.direction.y;
        if t <= 0.0 {
            return None;
        }
        let hit = Vector3::new(
            ray.position.x + ray.direction.x * t,
            floor_y,
            ray.position.z + ray.direction.z * t,
        );
        let local_x = hit.x - self.origin.x;
        let local_z = hit.z - self.origin.z;
        if local_x < 0.0 || local_z < 0.0 || local_x >= self.w as f32 || local_z >= self.d as f32 {
            return None;
        }
        let ix = local_x.floor() as i32;
        let iz = local_z.floor() as i32;
        let iy = 0;
        Some((ix, iy, iz))
    }

    pub fn reload_config(&mut self, config: FieldConfig) {
        self.config = config;
        self.generate_chi_field();
    }

    fn rectangular_jet_from_opening(&self, point: Vector3, opening: &Opening) -> (Vector2, f32) {
        if !matches!(opening.kind, OpeningKind::Door { .. }) {
            return (Vector2::ZERO, 0.0);
        }

        let center = opening.center();
        let p2d = Vector2::new(point.x, point.z);
        let c2d = Vector2::new(center.x, center.z);

        let to_point = p2d - c2d;
        let jet_normal = opening.normal();
        let jet_tangent = opening.tangent();

        let forward_dist = to_point.dot(jet_normal);
        let lateral_offset = to_point.dot(jet_tangent);

        if forward_dist <= 0.0 {
            return (Vector2::ZERO, 0.0);
        }
        if forward_dist > self.config.jet_max_distance {
            return (Vector2::ZERO, 0.0);
        }

        let half_width = opening.width() * 0.5;
        let spread_rad = self.config.jet_spread_angle.to_radians();
        let spread_amount = forward_dist * spread_rad.tan();
        let jet_half_width = half_width + spread_amount;

        if lateral_offset.abs() > jet_half_width {
            return (Vector2::ZERO, 0.0);
        }

        let dir = jet_normal;
        let edge = 1.0 - (lateral_offset.abs() / jet_half_width).powf(2.0);
        let dist = 1.0 - (forward_dist / self.config.jet_max_distance);
        let mag = self.config.jet_strength * edge * dist;

        (dir, mag)
    }

    fn converging_duct_to_opening(&self, point: Vector3, dir: Vector2, mag: f32, opening: &Opening) -> (Vector2, f32) {
        if !matches!(opening.kind, OpeningKind::Window) {
            return (dir, mag);
        }

        let center = opening.center();
        let p2d = Vector2::new(point.x, point.z);
        let c2d = Vector2::new(center.x, center.z);
        let normal = opening.normal();
        let tangent = opening.tangent();

        let to_point = p2d - c2d;

        let dist_from_window = to_point.dot(normal);
        if dist_from_window <= 0.0 {
            return (dir, mag);
        }
        if dist_from_window > self.config.funnel_reach {
            return (dir, mag);
        }

        let lateral = to_point.dot(tangent);
        let nd = dist_from_window / self.config.funnel_reach;
        let width_interp = nd.powf(self.config.funnel_curve_power);
        let funnel_radius = self.config.funnel_sink_radius
            + (self.config.funnel_catch_radius - self.config.funnel_sink_radius) * width_interp;

        if lateral.abs() > funnel_radius {
            return (dir, mag);
        }

        let target_lateral = if funnel_radius > 0.0 {
            lateral * (self.config.funnel_sink_radius / funnel_radius)
        } else {
            0.0
        };

        let target_point = c2d + tangent * target_lateral;
        let desired_dir = (target_point - p2d).normalize_or_zero();
        let proximity = 1.0 - nd;
        let lateral_factor = 1.0 - (lateral.abs() / funnel_radius).powf(2.0);
        let weight = self.config.funnel_strength * proximity * lateral_factor;
        let new_dir = blend_directions(dir, desired_dir, weight);
        let new_mag = if mag == 0.0 {
            weight * self.config.funnel_strength
        } else {
            mag
        };

        (new_dir, new_mag)
    }

    fn apply_back_wall_redirect(&self, point: Vector3, dir: Vector2, mag: f32) -> (Vector2, f32) {
        let origin = self.origin;
        let back_wall_z = origin.z;
        let dist = (point.z - back_wall_z).abs();
        let max_dist = self.config.wall_redirect_distance;
        if max_dist <= 0.0 || dist >= max_dist {
            return (dir, mag);
        }

        let base = self.config.wall_redirect_strength.clamp(0.0, 1.0);
        let falloff = 1.0 - (dist / max_dist);
        let weight = base * falloff;
        let center_x = origin.x + self.w as f32 * 0.5;
        let lateral = point.x - center_x;
        let desired_dir = Vector2::new(if lateral >= 0.0 { 1.0 } else { -1.0 }, 0.0);
        let new_dir = blend_directions(dir, desired_dir, weight);
        (new_dir, mag)
    }

    fn compute_energy_at_point(
        &self,
        point: Vector3,
        door: &Opening,
        maybe_window: Option<&Opening>,
    ) -> (Vector2, f32) {
        let (mut dir, mut mag) = self.rectangular_jet_from_opening(point, door);

        if let Some(win) = maybe_window {
            let (d, m) = self.converging_duct_to_opening(point, dir, mag, win);
            dir = d;
            mag = m;
        }

        let (d, m) = self.apply_back_wall_redirect(point, dir, mag);
        dir = d;
        mag = m;

        (dir.normalize_or_zero(), mag)
    }
    pub fn generate_chi_field(&mut self) {
        self.field_samples.clear();
        let door = *self.primary_door();
        let window = self.primary_window().copied();
        let base_y = self.origin.y + self.config.chi_sample_height;

        for iy in 0..self.h {
            for iz in 0..self.d {
                for ix in 0..self.w {
                    if iy != 0 {
                        continue;
                    }
                    let center = self.cell_center(ix, iy, iz);
                    let center_pos = Vector3::new(center.x, base_y, center.z);
                    let (dir, mag) = self.compute_energy_at_point(center_pos, &door, window.as_ref());
                    let dominant = self.classify_dominant_disrupter(center_pos, &door, window.as_ref());
                    self.field_samples.push(FieldSample {
                        position: center_pos,
                        direction: dir,
                        magnitude: mag,
                        dominant,
                    });
                    for &(dx, dz) in &[(-0.5, -0.5), (0.5, -0.5), (-0.5, 0.5), (0.5, 0.5)] {
                        let pos = Vector3::new(center_pos.x + dx, base_y, center_pos.z + dz);
                        let (d2, m2) = self.compute_energy_at_point(pos, &door, window.as_ref());
                        let dominant2 = self.classify_dominant_disrupter(pos, &door, window.as_ref());

                        self.field_samples.push(FieldSample {
                            position: pos,
                            direction: d2,
                            magnitude: m2,
                            dominant: dominant2,
                        });
                    }
                }
            }
        }
    }

    fn source_influences_at_point(
        &self,
        point: Vector3,
        door: &Opening,
        maybe_window: Option<&Opening>,
    ) -> (f32, f32, f32) {
        let (_dir_door, mag_door) = self.rectangular_jet_from_opening(point, door);
        let mut door_inf = mag_door.max(0.0);
        let mut window_inf = 0.0;

        if let Some(win) = maybe_window {
            let center = win.center();
            let p2d = Vector2::new(point.x, point.z);
            let c2d = Vector2::new(center.x, center.z);
            let normal = win.normal();
            let tangent = win.tangent();
            let to_point = p2d - c2d;

            let dist_from_window = to_point.dot(normal);
            if dist_from_window > 0.0 && dist_from_window <= self.config.funnel_reach {
                let lateral = to_point.dot(tangent);

                let nd = dist_from_window / self.config.funnel_reach;
                let width_interp = nd.powf(self.config.funnel_curve_power);
                let funnel_radius = self.config.funnel_sink_radius
                    + (self.config.funnel_catch_radius - self.config.funnel_sink_radius) * width_interp;

                if funnel_radius > 0.0 && lateral.abs() <= funnel_radius {
                    let proximity = 1.0 - nd;
                    let lateral_factor = 1.0 - (lateral.abs() / funnel_radius).powf(2.0);
                    window_inf = (proximity * lateral_factor * self.config.funnel_strength).max(0.0);
                }
            }
        }

        let mut wall_inf = 0.0;
        let back_wall_z = self.origin.z;
        let dist = (point.z - back_wall_z).abs();

        if self.config.wall_redirect_distance > 0.0 && dist < self.config.wall_redirect_distance {
            let falloff = 1.0 - (dist / self.config.wall_redirect_distance);
            wall_inf = (falloff * self.config.wall_redirect_strength).max(0.0);
        }

        (door_inf, window_inf, wall_inf)
    }

    fn classify_dominant_disrupter(
        &self,
        point: Vector3,
        door: &Opening,
        maybe_window: Option<&Opening>,
    ) -> FieldDisrupter {
        let (door_inf, window_inf, wall_inf) = self.source_influences_at_point(point, door, maybe_window);

        let max_inf = door_inf.max(window_inf.max(wall_inf));
        let eps = 1e-4;

        if max_inf < eps {
            return FieldDisrupter::Base;
        }

        if door_inf >= window_inf && door_inf >= wall_inf {
            FieldDisrupter::DoorPrimary
        } else if window_inf >= door_inf && window_inf >= wall_inf {
            FieldDisrupter::Window
        } else {
            FieldDisrupter::BackWall
        }
    }
}

pub fn update_spatial_frame(
    main: &Camera3D,
    aspect: f32,
    near: f32,
    far: f32,
    spatial_frame: &mut WeakMesh,
    space_factor: f32,
    aspect_factor: f32,
    ortho_factor: f32,
    view_config: &ViewConfig,
) {
    let (depth, right, up) = basis_vector(&main);
    let half_h_near = lerp(
        near * (view_config.fovy_perspective * 0.5).to_radians().tan(),
        0.5 * near_plane_height_orthographic(view_config),
        ortho_factor,
    );
    let half_w_near = lerp(half_h_near, half_h_near * aspect, aspect_factor);
    let half_h_far = lerp(
        far * (view_config.fovy_perspective * 0.5).to_radians().tan(),
        0.5 * near_plane_height_orthographic(view_config),
        ortho_factor,
    );
    let half_w_far = lerp(half_h_far, half_h_far * aspect, aspect_factor);
    let half_depth_ndc = lerp(half_h_near, 0.5 * (far - near), lerp(aspect_factor, 0.0, ortho_factor));
    let half_depth = lerp(0.5 * (far - near), half_depth_ndc, space_factor);
    let far_half_w = lerp(half_w_far, half_w_near, space_factor);
    let far_half_h = lerp(half_h_far, half_h_near, space_factor);
    let center_near = main.position.add(depth * near);

    let src_vertices = spatial_frame.vertices().to_vec();
    let mut out_vertices = src_vertices.clone();

    for [a, b, c] in spatial_frame.triangles() {
        for &i in [a, b, c].iter() {
            let offset = src_vertices[i].sub(center_near);

            let x_sign = if offset.dot(right) >= 0.0 { 1.0 } else { -1.0 };
            let y_sign = if offset.dot(up) >= 0.0 { 1.0 } else { -1.0 };
            let far_mask = if offset.dot(depth) > half_depth { 1.0 } else { 0.0 };
            let final_half_w = half_w_near + far_mask * (far_half_w - half_w_near);
            let final_half_h = half_h_near + far_mask * (far_half_h - half_h_near);
            let center = center_near.add(depth * (far_mask * 2.0 * half_depth));

            out_vertices[i] = center
                .add(right * (x_sign * final_half_w))
                .add(up * (y_sign * final_half_h));
        }
    }
    spatial_frame.vertices_mut().copy_from_slice(&out_vertices);
}

#[derive(Copy, Clone)]
pub struct MeshMetrics {
    pub vertex_count: usize,
    pub triangle_count: usize,
    pub normal_count: usize,
    pub texcoord_count: usize,
    pub color_count: usize,
    pub index_count: usize,
    pub total_bytes: usize,
}

impl MeshMetrics {
    pub fn measure(mesh: &WeakMesh) -> Self {
        let vertex_count = mesh.vertices().len();
        let triangle_count = mesh.triangles().count();
        let normal_count = mesh.normals().map(|n| n.len()).unwrap_or(0);
        let texcoord_count = mesh.texcoords().map(|t| t.len()).unwrap_or(0);
        let color_count = mesh.colors().map(|c| c.len()).unwrap_or(0);
        let index_count = mesh.indices().map(|i| i.len()).unwrap_or(0);

        let mut total_bytes = 0;
        total_bytes += vertex_count * size_of::<Vector3>();
        total_bytes += normal_count * size_of::<Vector3>();
        total_bytes += texcoord_count * size_of::<Vector2>();
        total_bytes += color_count * size_of::<Color>();
        total_bytes += index_count * size_of::<u16>();

        MeshMetrics {
            vertex_count,
            triangle_count,
            normal_count,
            texcoord_count,
            color_count,
            index_count,
            total_bytes,
        }
    }
}

pub fn gpu_vertex_stride_bytes(metrics: &MeshMetrics) -> usize {
    let mut stride = size_of::<Vector3>();
    if metrics.normal_count > 0 {
        stride += size_of::<Vector3>();
    }
    if metrics.texcoord_count > 0 {
        stride += size_of::<Vector2>();
    }
    if metrics.color_count > 0 {
        stride += size_of::<Color>();
    }
    stride
}

pub struct FrameDynamicMetrics {
    pub vertex_positions_written: usize,
    pub vertex_normals_written: usize,
    pub vertex_colors_written: usize,
}

impl FrameDynamicMetrics {
    pub fn new() -> Self {
        FrameDynamicMetrics {
            vertex_positions_written: 0,
            vertex_normals_written: 0,
            vertex_colors_written: 0,
        }
    }

    pub fn reset(&mut self) {
        self.vertex_positions_written = 0;
        self.vertex_normals_written = 0;
        self.vertex_colors_written = 0;
    }

    pub fn total_bytes_written(&self) -> usize {
        self.vertex_positions_written * size_of::<Vector3>()
            + self.vertex_normals_written * size_of::<Vector3>()
            + self.vertex_colors_written * size_of::<Color>()
    }
}

pub struct AnimationMetrics {
    pub sample_count: usize,
    pub verts_per_sample: usize,
    pub total_bytes: usize,
}

impl AnimationMetrics {
    pub fn measure(mesh_samples: &[Vec<Vector3>]) -> Option<Self> {
        if mesh_samples.is_empty() {
            return None;
        }

        let sample_count = mesh_samples.len();
        let verts_per_sample = mesh_samples[0].len();
        let total_bytes = sample_count * verts_per_sample * size_of::<Vector3>();

        Some(AnimationMetrics {
            sample_count,
            verts_per_sample,
            total_bytes,
        })
    }
}

pub struct MeshDescriptor {
    pub name: &'static str,
    pub world: Model,
    pub ndc: Model,
    pub texture: Texture2D,
    pub metrics_world: MeshMetrics,
    pub metrics_ndc: MeshMetrics,
    pub combined_bytes: usize,
    pub z_shift_anisotropic: f32,
    pub z_shift_isotropic: f32,
}

pub fn world_to_ndc_space(
    camera: &Camera3D,
    aspect: f32,
    near: f32,
    far: f32,
    world: &Model,
    ndc: &mut Model,
    rotation: f32,
    ortho_factor: f32,
    aspect_factor: f32,
    frame_metrics: &mut FrameDynamicMetrics,
    view_config: &ViewConfig,
) {
    let (depth, right, up) = basis_vector(camera);

    let half_h_near = lerp(
        near * (view_config.fovy_perspective * 0.5).to_radians().tan(),
        0.5 * near_plane_height_orthographic(view_config),
        ortho_factor,
    );
    let half_w_near = lerp(half_h_near, half_h_near * aspect, aspect_factor);
    let center_near = camera.position + depth * near;
    let half_depth_ndc = lerp(half_h_near, 0.5 * (far - near), lerp(aspect_factor, 0.0, ortho_factor));
    let center_ndc = center_near + depth * half_depth_ndc;
    let world_mesh = &world.meshes()[0];
    let ndc_mesh = &mut ndc.meshes_mut()[0];
    let src_vertices = world_mesh.vertices();
    let dst_vertices = ndc_mesh.vertices_mut();

    for [a, b, c] in world_mesh.triangles() {
        for i in [a, b, c] {
            let wv = translate_rotate_scale(0, src_vertices[i], MODEL_POS, MODEL_SCALE, rotation);
            let depth_signed = (wv - camera.position).dot(depth);
            let clip_coord = intersect(camera, near, wv, ortho_factor);
            let rel = clip_coord - center_near;
            let x_ndc = rel.dot(right) / half_w_near;
            let y_ndc = rel.dot(up) / half_h_near;
            let persp_z = (far + near - 2.0 * far * near / depth_signed) / (far - near);
            let ortho_z = 2.0 * (depth_signed - near) / (far - near) - 1.0;
            let z_ndc = lerp(persp_z, ortho_z, ortho_factor);
            let xw = right * (x_ndc * half_w_near);
            let yw = up * (y_ndc * half_h_near);
            let zw = depth * (z_ndc * half_depth_ndc);
            let final_pos = center_ndc + xw + yw + zw;
            dst_vertices[i] = translate_rotate_scale(1, final_pos, MODEL_POS, MODEL_SCALE, rotation);
            frame_metrics.vertex_positions_written += 1;
        }
    }
}

pub fn blend_world_and_ndc_vertices(
    world_model: &Model,
    ndc_model: &mut Model,
    blend: f32,
    frame_metrics: &mut FrameDynamicMetrics,
) {
    let src = &world_model.meshes()[0];
    let dst = &mut ndc_model.meshes_mut()[0];
    let src_v = src.vertices();
    let dst_v = dst.vertices_mut();
    for [a, b, c] in src.triangles() {
        for i in [a, b, c] {
            dst_v[i].x = lerp(src_v[i].x, dst_v[i].x, blend);
            dst_v[i].y = lerp(src_v[i].y, dst_v[i].y, blend);
            dst_v[i].z = lerp(src_v[i].z, dst_v[i].z, blend);
            frame_metrics.vertex_positions_written += 1;
        }
    }
}

pub fn collect_deformed_vertex_samples(base: &[Vector3], field_config: &FieldConfig) -> Vec<Vec<Vector3>> {
    let mut samples = Vec::with_capacity(field_config.rotational_samples_for_inv_proj);

    for i in 0..field_config.rotational_samples_for_inv_proj {
        let angle = -(i as f32) * TAU / (field_config.rotational_samples_for_inv_proj as f32);
        let rotation_progress = (i as f32) / (field_config.rotational_samples_for_inv_proj as f32);
        let wave_time = rotation_progress
            * (1.0 / field_config.rotation_frequency_hz)
            * field_config.deformation_cycles_per_rotation;

        let mut frame = base.to_vec();
        rotate_vertices_in_plane_slice(&mut frame, angle);
        let radial = generate_silhouette_radial_field(wave_time, field_config);
        deform_vertices_with_radial_field(&mut frame, &radial);
        rotate_vertices_in_plane_slice(&mut frame, -angle);
        samples.push(frame);
    }

    samples
}

pub fn generate_silhouette_radial_field(i_time: f32, field_config: &FieldConfig) -> Vec<f32> {
    let mut rf = Vec::with_capacity(RADIAL_FIELD_SIZE);

    for i in 0..RADIAL_FIELD_SIZE {
        let ang = (i as f32) * TAU / (RADIAL_FIELD_SIZE as f32);
        rf.push(sample_silhouette_radial_field(ang, i_time, field_config));
    }

    let max_r = rf.iter().cloned().fold(1e-6, f32::max);
    for r in &mut rf {
        *r /= max_r;
    }
    rf
}

#[inline]
pub fn sample_silhouette_radial_field(ang: f32, time: f32, field_config: &FieldConfig) -> f32 {
    let dir = Vector2::new(ang.cos(), ang.sin());
    let phase = field_config.wave_amplitude_x.hypot(field_config.wave_amplitude_y) + 2.0;
    let mut low = 0.0_f32;
    let mut high = UMBRAL_MASK_OUTER_RADIUS + phase;

    for _ in 0..8 {
        // TODO: can this be put in a config? it seems magic number evne if algo based
        let mut p = UMBRAL_MASK_CENTER + dir * high;
        if grid_phase_magnitude(&mut p, time, field_config) >= UMBRAL_MASK_OUTER_RADIUS {
            break;
        }
        high *= 1.5;
    }

    for _ in 0..20 {
        //TODO: can this be put in a config? it seems magic number evne if algo based
        let mid = 0.5 * (low + high);

        let mut p = UMBRAL_MASK_CENTER + dir * mid;
        if grid_phase_magnitude(&mut p, time, field_config) >= UMBRAL_MASK_OUTER_RADIUS {
            high = mid;
        } else {
            low = mid;
        }
    }

    high
}

pub fn deform_vertices_with_radial_field(vertices: &mut [Vector3], radial_field: &[f32]) {
    if vertices.is_empty() {
        return;
    }

    for v in vertices {
        let rad = interpolate_between_radial_field_elements(v.x, v.y, radial_field);
        v.x *= rad;
        v.y *= rad;
    }
}

pub fn interpolate_between_radial_field_elements(sample_x: f32, sample_y: f32, radial_field: &[f32]) -> f32 {
    let ang = sample_y.atan2(sample_x).rem_euclid(TAU);
    let idx = ang / TAU * RADIAL_FIELD_SIZE as f32;
    let i0 = idx.floor() as usize % RADIAL_FIELD_SIZE;
    let i1 = (i0 + 1) % RADIAL_FIELD_SIZE;
    let t = idx.fract();
    radial_field[i0] * (1.0 - t) + radial_field[i1] * t
}

#[inline]
pub fn grid_phase_magnitude(grid_coord: &mut Vector2, time: f32, field_config: &FieldConfig) -> f32 {
    let mut phase = spatial_phase(*grid_coord);
    phase += temporal_phase(time, field_config);
    *grid_coord += add_phase(phase, field_config);
    grid_coord.distance(UMBRAL_MASK_CENTER)
}

#[inline]
pub fn spatial_phase(grid: Vector2) -> Vector2 {
    const LIGHT_WAVE_SPATIAL_FREQ_X: f32 = 8.0;
    const LIGHT_WAVE_SPATIAL_FREQ_Y: f32 = 8.0;
    Vector2::new(grid.y * LIGHT_WAVE_SPATIAL_FREQ_X, grid.x * LIGHT_WAVE_SPATIAL_FREQ_Y)
}

#[inline]
pub fn temporal_phase(time: f32, field_config: &FieldConfig) -> Vector2 {
    let rotation_period = 1.0 / field_config.rotation_frequency_hz;
    let freq_x = (field_config.wave_cycles_fast * TAU) / rotation_period;
    let freq_y = (field_config.wave_cycles_slow * TAU) / rotation_period;

    Vector2::new(time * freq_x, time * freq_y)
}

#[inline]
pub fn add_phase(p: Vector2, field_config: &FieldConfig) -> Vector2 {
    Vector2::new(
        field_config.wave_amplitude_x * p.x.cos(),
        field_config.wave_amplitude_y * p.y.sin(),
    )
}

pub fn interpolate_between_deformed_vertices(
    model: &mut Model,
    mesh_rotation: f32,
    samples: &[Vec<Vector3>],
    frame_metrics: &mut FrameDynamicMetrics,
    field_config: &FieldConfig,
) {
    let normalized_rotation = (-mesh_rotation).rem_euclid(TAU);
    let rotation_progress = normalized_rotation / TAU;
    let f = rotation_progress * samples.len() as f32;
    let i0 = f.floor() as usize % samples.len();
    let i1 = (i0 + 1) % samples.len();
    let w = f.fract();
    let dst = model.meshes_mut()[0].vertices_mut();
    for ((v, a), b) in dst.iter_mut().zip(samples[i0].iter()).zip(samples[i1].iter()) {
        v.x = a.x * (1.0 - w) + b.x * w;
        v.y = a.y * (1.0 - w) + b.y * w;
        v.z = a.z * (1.0 - w) + b.z * w;
    }

    frame_metrics.vertex_positions_written += dst.len();
}

pub fn calculate_average_ndc_z_shift(world_model: &Model, ndc_model: &Model) -> f32 {
    let wv = world_model.meshes()[0].vertices();
    let nv = ndc_model.meshes()[0].vertices();
    let mut sum = 0.0;
    let mut count = 0usize;

    for (a, b) in wv.iter().zip(nv.iter()) {
        sum += b.z - a.z;
        count += 1;
    }

    if count > 0 {
        sum / count as f32
    } else {
        0.0
    }
}

pub fn update_blend(blend: &mut f32, dt: f32, target_on: bool, view_config: &ViewConfig) {
    if dt <= 0.0 {
        return;
    }

    let dir = if target_on { 1.0 } else { -1.0 };
    *blend = (*blend + dir * view_config.blend_scalar * dt).clamp(0.0, 1.0);
}

fn blend_directions(current: Vector2, desired: Vector2, weight: f32) -> Vector2 {
    let w = weight.clamp(0.0, 1.0);
    let c = current.normalize_or_zero();
    let d = desired.normalize_or_zero();
    c.lerp(d, w).normalize_or_zero()
}

pub fn basis_vector(main: &Camera3D) -> (Vector3, Vector3, Vector3) {
    let depth = main.target.sub(main.position).normalize();
    let right = depth.cross(main.up).normalize();
    let up = right.cross(depth).normalize();
    (depth, right, up)
}

#[inline]
pub fn observed_line_of_sight(observer: &Camera3D) -> Vector3 {
    Vector3::new(
        observer.target.x - observer.position.x,
        observer.target.y - observer.position.y,
        observer.target.z - observer.position.z,
    )
    .normalize_or_zero()
}

#[inline]
pub fn triangle_normal(a: Vector3, b: Vector3, c: Vector3) -> Vector3 {
    (b - a).cross(c - a).normalize_or_zero()
}

#[inline]
pub fn rotate_point_about_axis(c: Vector3, axis: (Vector3, Vector3), theta: f32) -> Vector3 {
    let (a, b) = axis;
    let ab = b - a;
    let ab_dir = ab.normalize_or_zero();
    let ac = c - a;
    let ac_proj = ab_dir.dot(ac) * ab_dir;
    let ac_x = ac - ac_proj;
    let ac_y = ab_dir.cross(ac_x);
    let rotated = ac_x * theta.cos() + ac_y * theta.sin() + ac_proj;
    a + rotated
}

#[inline]
pub fn rotate_vertices_in_plane_slice(vertices: &mut [Vector3], rot: f32) {
    let (s, c) = rot.sin_cos();
    for v in vertices {
        let x0 = v.x;
        let z0 = v.z;
        v.x = x0 * c + z0 * s;
        v.z = -x0 * s + z0 * c;
    }
}

fn translate_rotate_scale(inverse: i32, coord: Vector3, pos: Vector3, scale: Vector3, rotation: f32) -> Vector3 {
    let matrix = Mat4::from_scale(scale) * Mat4::from_rotation_y(rotation) * Mat4::from_translation(pos);
    let m = if inverse != 0 { matrix.inverse() } else { matrix };
    m.transform_point3(coord)
}

fn intersect(camera: &Camera3D, near: f32, world_coord: Vector3, ortho_factor: f32) -> Vector3 {
    let view_dir = camera.target.sub(camera.position).normalize();
    let cam_to_point = world_coord - camera.position;
    let depth_along_view = cam_to_point.dot(view_dir);
    let center_near = camera.position + view_dir * near;
    if depth_along_view <= 0.0 {
        return center_near;
    }

    let scale = near / depth_along_view;
    let persp = camera.position + cam_to_point * scale;
    let ortho = world_coord + view_dir * (center_near - world_coord).dot(view_dir);

    Vector3::new(
        persp.x + (ortho.x - persp.x) * ortho_factor,
        persp.y + (ortho.y - persp.y) * ortho_factor,
        persp.z + (ortho.z - persp.z) * ortho_factor,
    )
}

pub fn format_time_clock(seconds: f32) -> String {
    let clamped = if seconds.is_sign_negative() { 0.0 } else { seconds };
    let total_ms = (clamped * 1000.0).round() as u64;

    let minutes = total_ms / 60_000;
    let secs = (total_ms / 1000) % 60;
    let millis = total_ms % 1000;
    format!("{:02}:{:02}:{:03}", minutes, secs, millis)
}

pub fn format_bytes(bytes: usize) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = 1024.0 * 1024.0;

    let b = bytes as f64;

    if b < KB {
        format!("{bytes} B")
    } else if b < MB {
        let kb = b / KB;
        if kb < 10.0 {
            format!("{:.2} kB", kb)
        } else if kb < 100.0 {
            format!("{:.1} kB", kb)
        } else {
            format!("{:.0} kB", kb)
        }
    } else {
        let mb = b / MB;
        if mb < 10.0 {
            format!("{:.2} MB", mb)
        } else if mb < 100.0 {
            format!("{:.1} MB", mb)
        } else {
            format!("{:.0} MB", mb)
        }
    }
}

pub fn orbit_space(handle: &mut RaylibHandle, camera: &mut Camera3D) {
    let dt = handle.get_frame_time();
    let offset = camera.position - camera.target;
    let mut radius = offset.length();
    let mut azimuth = offset.z.atan2(offset.x);
    let horiz = (offset.x * offset.x + offset.z * offset.z).sqrt();
    let mut elevation = offset.y.atan2(horiz);

    if handle.is_key_down(KeyboardKey::KEY_LEFT) {
        azimuth += 1.0 * dt;
    }
    if handle.is_key_down(KeyboardKey::KEY_RIGHT) {
        azimuth -= 1.0 * dt;
    }
    if handle.is_key_down(KeyboardKey::KEY_UP) {
        elevation += 1.0 * dt;
    }
    if handle.is_key_down(KeyboardKey::KEY_DOWN) {
        elevation -= 1.0 * dt;
    }

    if handle.is_key_down(KeyboardKey::KEY_W) {
        radius -= 1.0 * dt;
    }
    if handle.is_key_down(KeyboardKey::KEY_S) {
        radius += 1.0 * dt;
    }

    radius = radius.clamp(0.25, 10.0);

    if camera.projection == CAMERA_ORTHOGRAPHIC {
        if handle.is_key_down(KeyboardKey::KEY_SEMICOLON) {
            camera.fovy += 6.0 * dt;
        }
        if handle.is_key_down(KeyboardKey::KEY_MINUS) {
            camera.fovy -= 6.0 * dt;
        }
        camera.fovy = camera.fovy.clamp(1.0, 30.0);
    } else {
        if handle.is_key_down(KeyboardKey::KEY_SEMICOLON) {
            camera.fovy += 35.0 * dt;
        }
        if handle.is_key_down(KeyboardKey::KEY_MINUS) {
            camera.fovy -= 35.0 * dt;
        }
        camera.fovy = camera.fovy.clamp(1.0, 130.0);
    }

    elevation = elevation.clamp(-PI / 2.0 + 0.1, PI / 2.0 - 0.1);
    camera.position.x = camera.target.x + radius * elevation.cos() * azimuth.cos();
    camera.position.y = camera.target.y + radius * elevation.sin();
    camera.position.z = camera.target.z + radius * elevation.cos() * azimuth.sin();
}
