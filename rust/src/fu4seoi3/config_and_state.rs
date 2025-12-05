use crate::fu4seoi3::core::*;
use raylib::prelude::*;
use std::f32::consts::TAU;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

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
    pub jugemu_state: JugemuState,
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
            jugemu_state: JugemuState::default(),
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

pub fn update_view_from_input(handle: &RaylibHandle, view_state: &mut ViewState, jugemu: &mut Camera3D) {
    handle_view_toggles(handle, view_state);
    handle_jugemu_projection_toggle(handle, view_state, jugemu);
    handle_mesh_selection(handle, view_state);
}

pub fn handle_view_toggles(handle: &RaylibHandle, view_state: &mut ViewState) {
    if handle.is_key_pressed(KeyboardKey::KEY_N) {
        view_state.ndc_space = !view_state.ndc_space;
    }
    if handle.is_key_pressed(KeyboardKey::KEY_Q) {
        view_state.aspect_correct = !view_state.aspect_correct;
    }
    if handle.is_key_pressed(KeyboardKey::KEY_SPACE) {
        view_state.paused = !view_state.paused;
    }
    if handle.is_key_pressed(KeyboardKey::KEY_C) {
        view_state.color_mode = !view_state.color_mode;
    }
    if handle.is_key_pressed(KeyboardKey::KEY_T) {
        view_state.texture_mode = !view_state.texture_mode;
    }
    if handle.is_key_pressed(KeyboardKey::KEY_J) {
        view_state.jugemu_mode = !view_state.jugemu_mode;
    }
    if handle.is_key_pressed(KeyboardKey::KEY_O) {
        view_state.ortho_mode = !view_state.ortho_mode;
    }
}

pub fn handle_jugemu_projection_toggle(handle: &RaylibHandle, view_state: &mut ViewState, jugemu: &mut Camera3D) {
    if handle.is_key_pressed(KeyboardKey::KEY_P) {
        if view_state.jugemu_ortho_mode {
            view_state.jugemu_state.fovy_ortho = jugemu.fovy;
            view_state.jugemu_state.distance_ortho = camera_distance(jugemu);
            jugemu.fovy = view_state.jugemu_state.fovy_perspective;
            let dir = jugemu.position.normalize();
            jugemu.position = dir * view_state.jugemu_state.distance_perspective;
        } else {
            view_state.jugemu_state.fovy_perspective = jugemu.fovy;
            view_state.jugemu_state.distance_perspective = camera_distance(jugemu);
            jugemu.fovy = view_state.jugemu_state.fovy_ortho;
            let dir = jugemu.position.normalize();
            jugemu.position = dir * view_state.jugemu_state.distance_ortho;
        }
        view_state.jugemu_ortho_mode = !view_state.jugemu_ortho_mode;
    }
}

pub fn camera_distance(cam: &Camera3D) -> f32 {
    let dx = cam.position.x - cam.target.x;
    let dy = cam.position.y - cam.target.y;
    let dz = cam.position.z - cam.target.z;
    (dx * dx + dy * dy + dz * dz).sqrt()
}

pub fn handle_mesh_selection(handle: &RaylibHandle, view_state: &mut ViewState) {
    if handle.is_key_pressed(KeyboardKey::KEY_ONE) {
        view_state.target_mesh_index = 0;
    }
    if handle.is_key_pressed(KeyboardKey::KEY_TWO) {
        view_state.target_mesh_index = 1;
    }
    if handle.is_key_pressed(KeyboardKey::KEY_THREE) {
        view_state.target_mesh_index = 2;
    }
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
    if let Some(cell_index) = placed_cells.iter().position(|c| c.ix == ix && c.iy == iy && c.iz == iz) {
        placed_cells.remove(cell_index);
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

pub fn timestamp() -> String {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64();
    let secs = now as u64;
    let millis = ((now - secs as f64) * 1000.0) as u32;
    let hours = (secs / 3600) % 24;
    let mins = (secs / 60) % 60;
    let s = secs % 60;
    format!("[{:02}:{:02}:{:02}.{:03}]", hours, mins, s, millis)
}
