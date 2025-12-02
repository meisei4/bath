use raylib::consts::CameraProjection::CAMERA_ORTHOGRAPHIC;
use raylib::ffi;
use raylib::math::glam::{Mat4, Vec3};
use raylib::prelude::*;
use std::f32::consts::{PI, TAU};
use std::fs;
use std::mem::size_of;
use std::ops::{Add, Sub};
use std::time::SystemTime;

pub const DC_WIDTH_BASE: f32 = 640.0;
pub const DC_HEIGHT_BASE: f32 = 480.0;
pub const DC_WIDTH: i32 = (DC_WIDTH_BASE * RES_SCALE) as i32;
pub const DC_HEIGHT: i32 = (DC_HEIGHT_BASE * RES_SCALE) as i32;

pub const MODEL_SCALE: Vector3 = Vector3::new(1.0, 1.0, 1.0);
pub const MODEL_POS: Vector3 = Vector3::ZERO;

pub const MAIN_POS: Vector3 = Vector3::new(0.0, 0.0, 2.0);
pub const JUGEMU_POS_ISO: Vector3 = Vector3::new(1.0, 1.0, 1.0);
pub const JUGEMU_DISTANCE_ORTHO: f32 = 6.5;
pub const JUGEMU_DISTANCE_PERSPECTIVE: f32 = 9.0;

pub const FOVY_PERSPECTIVE: f32 = 50.0;
pub const FOVY_ORTHOGRAPHIC: f32 = 9.0;
pub fn NEAR_PLANE_HEIGHT_ORTHOGRAPHIC() -> f32 {
    2.0 * (FOVY_PERSPECTIVE * 0.5).to_radians().tan()
}

pub const Y_AXIS: Vector3 = Vector3::new(0.0, 1.0, 0.0);
pub const RES_SCALE: f32 = 1.5;
pub const BLEND_SCALAR: f32 = 5.0;
pub const PLACEMENT_ANIM_DUR_SECONDS: f32 = 0.15;
pub const HINT_SCALE: f32 = 0.66;
pub const HINT_SCALE_VEC: Vector3 = Vector3::new(HINT_SCALE, HINT_SCALE, HINT_SCALE);

pub const BAHAMA_BLUE: Color = Color::new(0, 102, 153, 255);
pub const SUNFLOWER: Color = Color::new(255, 204, 153, 255);
pub const PALE_CANARY: Color = Color::new(255, 255, 153, 255);
pub const ANAKIWA: Color = Color::new(153, 204, 255, 255);
pub const MARINER: Color = Color::new(51, 102, 204, 255);
pub const NEON_CARROT: Color = Color::new(255, 153, 51, 255);
pub const EGGPLANT: Color = Color::new(102, 68, 102, 255);
pub const HOPBUSH: Color = Color::new(204, 102, 153, 255);
pub const LILAC: Color = Color::new(204, 153, 204, 255);
pub const RED_DAMASK: Color = Color::new(221, 102, 68, 255);
pub const CHESTNUT_ROSE: Color = Color::new(204, 102, 102, 255);

pub const HUD_MARGIN: i32 = 12;
pub const HUD_LINE_HEIGHT: i32 = 22;
pub const FONT_SIZE: i32 = 20;
pub const HUD_CHAR_SPACING: f32 = 2.0;

pub struct HudLayout {
    pub font_size_main: i32,
    pub font_size_debug: i32,
    pub line_height_main: i32,
    pub line_height_debug: i32,
    pub margin: i32,
    pub left_label_x: i32,
    pub left_value_x: i32,
    pub right_label_x: i32,
    pub right_value_x: i32,
    pub right_value_max_width: i32,
    pub bottom_block_start_y: i32,
    pub perf_x: i32,
    pub perf_y: i32,
    pub debug_padding: i32,
}

pub const ROOM_W: i32 = 9;
pub const ROOM_H: i32 = 3;
pub const ROOM_D: i32 = 9;

pub const HALF: f32 = 0.5;
pub const GRID_SCALE: f32 = 4.0;
pub const GRID_CELL_SIZE: f32 = 1.0 / GRID_SCALE;
pub const GRID_ORIGIN_INDEX: Vector2 = Vector2::new(0.0, 0.0);
pub const GRID_ORIGIN_OFFSET_CELLS: Vector2 = Vector2::new(2.0, 2.0);
pub const GRID_ORIGIN_UV_OFFSET: Vector2 = Vector2::new(
    (GRID_ORIGIN_INDEX.x + GRID_ORIGIN_OFFSET_CELLS.x) * GRID_CELL_SIZE,
    (GRID_ORIGIN_INDEX.y + GRID_ORIGIN_OFFSET_CELLS.y) * GRID_CELL_SIZE,
);

pub const LIGHT_WAVE_SPATIAL_FREQ_X: f32 = 8.0;
pub const LIGHT_WAVE_SPATIAL_FREQ_Y: f32 = 8.0;
pub const LIGHT_WAVE_TEMPORAL_FREQ_X: f32 = 255.0 * PI / 10.0;
pub const LIGHT_WAVE_TEMPORAL_FREQ_Y: f32 = 7.0 * PI / 10.0;
pub const LIGHT_WAVE_AMPLITUDE_X: f32 = 0.0;
pub const LIGHT_WAVE_AMPLITUDE_Y: f32 = 0.1;
pub const UMBRAL_MASK_OUTER_RADIUS: f32 = 0.40;
pub const UMBRAL_MASK_FADE_BAND: f32 = 0.025;
pub const UMBRAL_MASK_CENTER: Vector2 = Vector2::new(HALF, HALF);
pub const RADIAL_FIELD_SIZE: usize = 64;
pub const ROTATION_FREQUENCY_HZ: f32 = 0.05;
pub const ANGULAR_VELOCITY: f32 = TAU * ROTATION_FREQUENCY_HZ;
pub const TIME_BETWEEN_SAMPLES: f32 = 0.5;
pub const ROTATIONAL_SAMPLES_FOR_INV_PROJ: usize = 40;

pub const CHI_FIELD_SAMPLE_HEIGHT: f32 = 0.25;
pub const CHI_ARROW_LENGTH: f32 = 0.25;

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
    pub fn age(&self, i_time: f32) -> f32 {
        i_time - self.placed_time
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
        }
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

pub fn compute_hover_state(handle: &RaylibHandle, jugemu: &Camera3D, placed_cells: &[PlacedCell]) -> HoverState {
    if let Some((ix, iy, iz)) = get_hovered_room_floor_cell(handle, jugemu) {
        let center = cell_center(ix, iy, iz);
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

fn get_hovered_room_floor_cell(handle: &RaylibHandle, camera: &Camera3D) -> Option<(i32, i32, i32)> {
    let mouse = handle.get_mouse_position();
    let ray = handle.get_screen_to_world_ray(mouse, *camera);

    if ray.direction.y.abs() < 1e-5 {
        return None;
    }

    let floor_y = -(ROOM_H as f32) / 2.0;
    let t = (floor_y - ray.position.y) / ray.direction.y;
    if t <= 0.0 {
        return None;
    }

    let hit = Vector3::new(
        ray.position.x + ray.direction.x * t,
        floor_y,
        ray.position.z + ray.direction.z * t,
    );

    let origin = room_origin();
    let local_x = hit.x - origin.x;
    let local_z = hit.z - origin.z;
    if local_x < 0.0 || local_z < 0.0 || local_x >= ROOM_W as f32 || local_z >= ROOM_D as f32 {
        return None;
    }

    let cell_x = local_x.floor() as i32;
    let cell_z = local_z.floor() as i32;
    let cell_y = 0;

    Some((cell_x, cell_y, cell_z))
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

pub struct DynamicGeometryOps {
    pub vertices_touched: usize,
    pub bytes_modified: usize,
    pub operations_performed: usize,
}

impl DynamicGeometryOps {
    pub fn calculate(view_state: &ViewState, placed_cells: &[PlacedCell], ghost_vertex_count: usize) -> Self {
        let active_ghosts = if view_state.paused {
            0
        } else {
            placed_cells.iter().filter(|c| c.mesh_index == 0).count()
                + if view_state.target_mesh_index == 0 { 1 } else { 0 }
        };

        let vertices_touched = active_ghosts * ghost_vertex_count;
        let bytes_modified = vertices_touched * size_of::<Vector3>();

        let operations_performed = vertices_touched * 3;

        DynamicGeometryOps {
            vertices_touched,
            bytes_modified,
            operations_performed,
        }
    }
}

pub struct ColorGuard {
    cached_colors_ptr: *mut std::ffi::c_uchar,
    restore_target: *mut ffi::Mesh,
}

impl ColorGuard {
    pub fn hide(mesh: &mut WeakMesh) -> Self {
        let mesh_ptr = mesh.as_mut() as *mut ffi::Mesh;
        let colors_ptr = unsafe { (*mesh_ptr).colors };
        unsafe {
            (*mesh_ptr).colors = std::ptr::null_mut();
        }
        Self {
            cached_colors_ptr: colors_ptr,
            restore_target: mesh_ptr,
        }
    }
}

impl Drop for ColorGuard {
    fn drop(&mut self) {
        unsafe {
            (*self.restore_target).colors = self.cached_colors_ptr;
        }
    }
}

pub struct TextureGuard {
    cached_texture_id: std::ffi::c_uint,
    restore_target: *mut Model,
}

impl TextureGuard {
    pub fn hide(model: &mut Model) -> Self {
        use raylib::consts::MaterialMapIndex::MATERIAL_MAP_ALBEDO;
        let cached_id = model.materials_mut()[0].maps_mut()[MATERIAL_MAP_ALBEDO as usize]
            .texture
            .id;
        model.materials_mut()[0].maps_mut()[MATERIAL_MAP_ALBEDO as usize]
            .texture
            .id = 0;
        Self {
            cached_texture_id: cached_id,
            restore_target: model as *mut Model,
        }
    }

    pub fn set_texture(model: &mut Model, texture_id: u32) -> Self {
        use raylib::consts::MaterialMapIndex::MATERIAL_MAP_ALBEDO;
        let cached_id = model.materials_mut()[0].maps_mut()[MATERIAL_MAP_ALBEDO as usize]
            .texture
            .id;
        model.materials_mut()[0].maps_mut()[MATERIAL_MAP_ALBEDO as usize]
            .texture
            .id = texture_id;
        Self {
            cached_texture_id: cached_id,
            restore_target: model as *mut Model,
        }
    }
}

impl Drop for TextureGuard {
    fn drop(&mut self) {
        use raylib::consts::MaterialMapIndex::MATERIAL_MAP_ALBEDO;
        unsafe {
            (*self.restore_target).materials_mut()[0].maps_mut()[MATERIAL_MAP_ALBEDO as usize]
                .texture
                .id = self.cached_texture_id;
        }
    }
}

#[derive(Clone)]
pub struct Door {
    pub p1: Vector3,
    pub p2: Vector3,
    pub is_primary: bool,
}

impl Door {
    pub fn center(&self) -> Vector3 {
        Vector3::new(
            (self.p1.x + self.p2.x) * 0.5,
            (self.p1.y + self.p2.y) * 0.5,
            (self.p1.z + self.p2.z) * 0.5,
        )
    }

    pub fn normal(&self) -> Vector2 {
        let dx = self.p2.x - self.p1.x;
        let dz = self.p2.z - self.p1.z;
        Vector2::new(dz, -dx).normalize_or_zero()
    }

    pub fn width(&self) -> f32 {
        let dx = self.p2.x - self.p1.x;
        let dz = self.p2.z - self.p1.z;
        (dx * dx + dz * dz).sqrt()
    }

    pub fn tangent(&self) -> Vector2 {
        let dx = self.p2.x - self.p1.x;
        let dz = self.p2.z - self.p1.z;
        Vector2::new(dx, dz).normalize_or_zero()
    }
}

#[derive(Clone)]
pub struct Window {
    pub p1: Vector3,
    pub p2: Vector3,
}

impl Window {
    pub fn center(&self) -> Vector3 {
        Vector3::new(
            (self.p1.x + self.p2.x) * 0.5,
            (self.p1.y + self.p2.y) * 0.5,
            (self.p1.z + self.p2.z) * 0.5,
        )
    }

    pub fn width(&self) -> f32 {
        let dx = self.p2.x - self.p1.x;
        let dz = self.p2.z - self.p1.z;
        (dx * dx + dz * dz).sqrt()
    }

    pub fn normal(&self) -> Vector2 {
        let dx = self.p2.x - self.p1.x;
        let dz = self.p2.z - self.p1.z;
        Vector2::new(dz, -dx).normalize_or_zero()
    }

    pub fn tangent(&self) -> Vector2 {
        let dx = self.p2.x - self.p1.x;
        let dz = self.p2.z - self.p1.z;
        Vector2::new(dx, dz).normalize_or_zero()
    }
}

pub struct FieldSample {
    pub position: Vector3,
    pub direction: Vector2,
    pub magnitude: f32,
}

#[derive(Clone, Debug, Default)]
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
}

impl FieldConfig {
    pub fn load_from_file(path: &str) -> Self {
        let mut config = FieldConfig::default();

        if let Ok(content) = fs::read_to_string(path) {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }

                let parts: Vec<&str> = line.split(' ').map(|s| s.trim()).collect();
                if parts.len() != 2 {
                    continue;
                }

                let key = parts[0];
                let value: f32 = parts[1].parse().unwrap_or(0.0);

                match key {
                    "JET_STRENGTH" => config.jet_strength = value,
                    "JET_SPREAD_ANGLE" => config.jet_spread_angle = value,
                    "JET_MAX_DISTANCE" => config.jet_max_distance = value,
                    "FUNNEL_STRENGTH" => config.funnel_strength = value,
                    "FUNNEL_REACH" => config.funnel_reach = value,
                    "FUNNEL_CATCH_RADIUS" => config.funnel_catch_radius = value,
                    "FUNNEL_SINK_RADIUS" => config.funnel_sink_radius = value,
                    "FUNNEL_CURVE_POWER" => config.funnel_curve_power = value,
                    "WALL_REDIRECT_STRENGTH" => config.wall_redirect_strength = value,
                    "WALL_REDIRECT_DISTANCE" => config.wall_redirect_distance = value,
                    _ => {},
                }
            }
        }

        config
    }
}

pub struct ConfigWatcher {
    path: String,
    last_modified: Option<SystemTime>,
}

impl ConfigWatcher {
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
            last_modified: None,
        }
    }

    pub fn check_reload(&mut self) -> Option<FieldConfig> {
        if let Ok(metadata) = fs::metadata(&self.path) {
            if let Ok(modified) = metadata.modified() {
                if self.last_modified.is_none() || Some(modified) != self.last_modified {
                    self.last_modified = Some(modified);
                    return Some(FieldConfig::load_from_file(&self.path));
                }
            }
        }
        None
    }
}

pub struct Room {
    pub room_w: i32,
    pub room_d: i32,
    pub doors: Vec<Door>,
    pub windows: Vec<Window>,
    pub field_samples: Vec<FieldSample>,
    pub config: FieldConfig,
}

impl Default for Room {
    fn default() -> Self {
        let origin = room_origin();
        let north_z = origin.z + ROOM_D as f32;
        let center_x = origin.x + ROOM_W as f32 * 0.5;
        let door = Door {
            p1: Vector3::new(center_x - 1.0, origin.y, north_z),
            p2: Vector3::new(center_x + 1.0, origin.y, north_z),
            is_primary: true,
        };
        let west_x = origin.x;
        let center_z = origin.z + ROOM_D as f32 * 0.5;
        let window = Window {
            p1: Vector3::new(west_x, origin.y, center_z - 1.5),
            p2: Vector3::new(west_x, origin.y, center_z + 1.5),
        };
        let mut room = Room {
            room_w: ROOM_W,
            room_d: ROOM_D,
            doors: vec![door],
            windows: vec![window],
            field_samples: Vec::new(),
            config: FieldConfig::default(),
        };

        room.generate_field();
        room
    }
}

fn blend_directions(current: Vector2, desired: Vector2, weight: f32) -> Vector2 {
    let w = weight.clamp(0.0, 1.0);
    let c = current.normalize_or_zero();
    let d = desired.normalize_or_zero();
    c.lerp(d, w).normalize_or_zero()
}

impl Room {
    pub fn reload_config(&mut self, config: FieldConfig) {
        self.config = config;
        println!(
            "JET_STRENGTH = {}\n\
         JET_SPREAD_ANGLE = {}\n\
         JET_MAX_DISTANCE = {}\n\
         FUNNEL_STRENGTH = {}\n\
         FUNNEL_REACH = {}\n\
         FUNNEL_CATCH_RADIUS = {}\n\
         FUNNEL_SINK_RADIUS = {}\n\
         FUNNEL_CURVE_POWER = {}\n\
         WALL_REDIRECT_STRENGTH = {}\n\
         WALL_REDIRECT_DISTANCE = {}",
            self.config.jet_strength,
            self.config.jet_spread_angle,
            self.config.jet_max_distance,
            self.config.funnel_strength,
            self.config.funnel_reach,
            self.config.funnel_catch_radius,
            self.config.funnel_sink_radius,
            self.config.funnel_curve_power,
            self.config.wall_redirect_strength,
            self.config.wall_redirect_distance,
        );
        self.generate_field();
        self.log_debug_samples();
    }

    fn rectangular_jet_from_door(&self, point: Vector3, door: &Door) -> (Vector2, f32) {
        let door_center_2d = Vector2::new(door.center().x, door.center().z);
        let point_2d = Vector2::new(point.x, point.z);
        let to_point = point_2d - door_center_2d;

        let jet_normal = door.normal();
        let jet_tangent = door.tangent();

        let forward_dist = to_point.dot(jet_normal);
        let lateral_offset = to_point.dot(jet_tangent);

        if forward_dist <= 0.0 {
            return (Vector2::ZERO, 0.0);
        }

        if forward_dist > self.config.jet_max_distance {
            return (Vector2::ZERO, 0.0);
        }

        let spread_radians = self.config.jet_spread_angle.to_radians();
        let half_door_width = door.width() * 0.5;
        let spread_amount = forward_dist * spread_radians.tan();
        let jet_half_width = half_door_width + spread_amount;

        if lateral_offset.abs() > jet_half_width {
            return (Vector2::ZERO, 0.0);
        }

        let dir = jet_normal;
        let edge_falloff = 1.0 - (lateral_offset.abs() / jet_half_width).powf(2.0);
        let distance_falloff = 1.0 - (forward_dist / self.config.jet_max_distance);
        let mag = self.config.jet_strength * edge_falloff * distance_falloff;
        (dir, mag)
    }

    fn converging_duct_to_window(&self, point: Vector3, dir: Vector2, mag: f32, window: &Window) -> (Vector2, f32) {
        let window_center_2d = Vector2::new(window.center().x, window.center().z);
        let point_2d = Vector2::new(point.x, point.z);
        let window_normal = window.normal();
        let window_tangent = window.tangent();
        let to_point = point_2d - window_center_2d;
        let distance_from_window = to_point.dot(window_normal);
        let lateral_position = to_point.dot(window_tangent);
        if distance_from_window <= 0.0 {
            return (dir, mag);
        }
        if distance_from_window > self.config.funnel_reach {
            return (dir, mag);
        }
        let normalized_dist = distance_from_window / self.config.funnel_reach;
        let width_interp = normalized_dist.powf(self.config.funnel_curve_power);
        let funnel_radius_at_point = self.config.funnel_sink_radius
            + (self.config.funnel_catch_radius - self.config.funnel_sink_radius) * width_interp;

        if lateral_position.abs() > funnel_radius_at_point {
            return (dir, mag);
        }

        let target_lateral = if funnel_radius_at_point > 0.0 {
            lateral_position * (self.config.funnel_sink_radius / funnel_radius_at_point)
        } else {
            0.0
        };

        let target_point = window_center_2d + window_tangent * target_lateral;
        let to_target = target_point - point_2d;
        let desired_dir = to_target.normalize_or_zero();
        let proximity_factor = 1.0 - normalized_dist;
        let lateral_factor = 1.0 - (lateral_position.abs() / funnel_radius_at_point).powf(2.0);
        let weight = self.config.funnel_strength * proximity_factor * lateral_factor;
        let new_mag = if mag == 0.0 {
            weight * self.config.funnel_strength
        } else {
            mag
        };
        let new_dir = blend_directions(dir, desired_dir, weight);
        (new_dir, new_mag)
    }

    fn apply_back_wall_redirect(&self, point: Vector3, dir: Vector2, mag: f32) -> (Vector2, f32) {
        let origin = room_origin();
        let back_wall_z = origin.z;
        let dist = (point.z - back_wall_z).abs();
        let max_dist = self.config.wall_redirect_distance;
        if max_dist <= 0.0 || dist >= max_dist {
            return (dir, mag);
        }

        let base_strength = self.config.wall_redirect_strength.clamp(0.0, 1.0);
        let falloff = 1.0 - (dist / max_dist);
        let weight = base_strength * falloff;

        let center_x = origin.x + self.room_w as f32 * 0.5;
        let lateral = point.x - center_x;
        let sign = if lateral >= 0.0 { 1.0 } else { -1.0 };

        let desired_dir = Vector2::new(sign, 0.0);
        let new_dir = blend_directions(dir, desired_dir, weight);

        (new_dir, mag)
    }

    pub fn generate_field(&mut self) {
        self.field_samples.clear();
        let origin = room_origin();
        let primary_door = self.doors.iter().find(|d| d.is_primary).expect("Need primary door");
        let primary_window = self.windows.first();

        for iz in 0..self.room_d {
            for ix in 0..self.room_w {
                let center_x = origin.x + ix as f32 + 0.5;
                let center_z = origin.z + iz as f32 + 0.5;
                let center_pos = Vector3::new(center_x, origin.y + CHI_FIELD_SAMPLE_HEIGHT, center_z);
                let (center_dir, center_mag) = self.compute_energy_at_point(center_pos, primary_door, primary_window);

                self.field_samples.push(FieldSample {
                    position: center_pos,
                    direction: center_dir,
                    magnitude: center_mag,
                });

                for &(dx, dz) in &[(-0.5, -0.5), (0.5, -0.5), (-0.5, 0.5), (0.5, 0.5)] {
                    let corner_x = origin.x + ix as f32 + 0.5 + dx;
                    let corner_z = origin.z + iz as f32 + 0.5 + dz;
                    let corner_pos = Vector3::new(corner_x, origin.y + CHI_FIELD_SAMPLE_HEIGHT, corner_z);
                    let (corner_dir, corner_mag) =
                        self.compute_energy_at_point(corner_pos, primary_door, primary_window);

                    self.field_samples.push(FieldSample {
                        position: corner_pos,
                        direction: corner_dir,
                        magnitude: corner_mag,
                    });
                }
            }
        }
    }

    fn compute_energy_at_point(&self, point: Vector3, door: &Door, window: Option<&Window>) -> (Vector2, f32) {
        let (mut dir, mut mag) = self.rectangular_jet_from_door(point, door);

        if let Some(win) = window {
            let (d, m) = self.converging_duct_to_window(point, dir, mag, win);
            dir = d;
            mag = m;
        }

        let (d, m) = self.apply_back_wall_redirect(point, dir, mag);
        dir = d;
        mag = m;

        let dir = dir.normalize_or_zero();
        (dir, mag)
    }

    pub fn log_debug_samples(&self) {
        let origin = room_origin();
        let primary_door = self.doors.iter().find(|d| d.is_primary);
        if primary_door.is_none() {
            return;
        }
        let primary_door = primary_door.unwrap();
        let primary_window = self.windows.first();

        let center_x = origin.x + self.room_w as f32 * 0.5;
        let center_z = origin.z + self.room_d as f32 * 0.5;

        let center_pos = Vector3::new(center_x, origin.y + CHI_FIELD_SAMPLE_HEIGHT, center_z);

        let near_door_pos = {
            let c = primary_door.center();
            Vector3::new(c.x, origin.y + CHI_FIELD_SAMPLE_HEIGHT, c.z - 1.0)
        };

        let near_window_pos = if let Some(win) = primary_window {
            let c = win.center();
            Vector3::new(c.x + 1.0, origin.y + CHI_FIELD_SAMPLE_HEIGHT, c.z)
        } else {
            center_pos
        };

        let near_back_wall_pos = Vector3::new(center_x, origin.y + CHI_FIELD_SAMPLE_HEIGHT, origin.z + 0.5);

        let quarter_w = self.room_w as f32 * 0.25;
        let quarter_d = self.room_d as f32 * 0.25;

        let nw_pos = Vector3::new(
            center_x - quarter_w,
            origin.y + CHI_FIELD_SAMPLE_HEIGHT,
            center_z + quarter_d,
        );

        let ne_pos = Vector3::new(
            center_x + quarter_w,
            origin.y + CHI_FIELD_SAMPLE_HEIGHT,
            center_z + quarter_d,
        );

        let sw_pos = Vector3::new(
            center_x - quarter_w,
            origin.y + CHI_FIELD_SAMPLE_HEIGHT,
            center_z - quarter_d,
        );

        let se_pos = Vector3::new(
            center_x + quarter_w,
            origin.y + CHI_FIELD_SAMPLE_HEIGHT,
            center_z - quarter_d,
        );

        let probes = [
            ("MID", center_pos),
            ("DOOR", near_door_pos),
            ("WINDOW", near_window_pos),
            ("BACKWALL", near_back_wall_pos),
            ("NW", nw_pos),
            ("NE", ne_pos),
            ("SW", sw_pos),
            ("SE", se_pos),
        ];

        println!("--- chi debug samples ---");
        for (name, pos) in probes {
            let (dir, mag) = self.compute_energy_at_point(pos, primary_door, primary_window);
            let angle_deg = dir.y.atan2(dir.x).to_degrees();
            println!(
                "[chi] {:>12}: pos=({:5.2}, {:5.2}) dir=({:5.2}, {:5.2}) angle={:6.1}° mag={:4.2}",
                name, pos.x, pos.z, dir.x, dir.y, angle_deg, mag,
            );
        }
    }
}

pub fn room_origin() -> Vector3 {
    Vector3::new(-(ROOM_W as f32) / 2.0, -(ROOM_H as f32) / 2.0, -(ROOM_D as f32) / 2.0)
}

pub fn cell_center(ix: i32, iy: i32, iz: i32) -> Vector3 {
    let origin = room_origin();
    Vector3::new(
        origin.x + ix as f32 + 0.5,
        origin.y + iy as f32 + 0.5,
        origin.z + iz as f32 + 0.5,
    )
}

pub fn cell_top_right_front_corner(ix: i32, iy: i32, iz: i32, camera: &Camera3D) -> Vector3 {
    let center = cell_center(ix, iy, iz);
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

    let (depth, right, up) = basis_vector(camera);
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

pub fn update_spatial_frame(
    main: &mut Camera3D,
    aspect: f32,
    near: f32,
    far: f32,
    spatial_frame: &mut WeakMesh,
    space_factor: f32,
    aspect_factor: f32,
    ortho_factor: f32,
) {
    let (depth, right, up) = basis_vector(&main);
    let half_h_near = lerp(
        near * (FOVY_PERSPECTIVE * 0.5).to_radians().tan(),
        0.5 * NEAR_PLANE_HEIGHT_ORTHOGRAPHIC(),
        ortho_factor,
    );
    let half_w_near = lerp(half_h_near, half_h_near * aspect, aspect_factor);
    let half_h_far = lerp(
        far * (FOVY_PERSPECTIVE * 0.5).to_radians().tan(),
        0.5 * NEAR_PLANE_HEIGHT_ORTHOGRAPHIC(),
        ortho_factor,
    );
    let half_w_far = lerp(half_h_far, half_h_far * aspect, aspect_factor);
    let half_depth_ndc = lerp(half_h_near, 0.5 * (far - near), lerp(aspect_factor, 0.0, ortho_factor));
    let half_depth = lerp(0.5 * (far - near), half_depth_ndc, space_factor);
    let far_half_w = lerp(half_w_far, half_w_near, space_factor);
    let far_half_h = lerp(half_h_far, half_h_near, space_factor);
    let center_near = main.position.add(depth * near);

    let spatial_frame_triangles = spatial_frame.triangles().collect::<Vec<[usize; 3]>>();
    let spatial_frame_vertices_snapshot = spatial_frame.vertices().to_vec();
    let spatial_frame_vertices = spatial_frame.vertices_mut();

    for [a, b, c] in spatial_frame_triangles {
        for &i in [a, b, c].iter() {
            let offset = spatial_frame_vertices_snapshot[i].sub(center_near);
            let x_sign = if offset.dot(right) >= 0.0 { 1.0 } else { -1.0 };
            let y_sign = if offset.dot(up) >= 0.0 { 1.0 } else { -1.0 };
            let far_mask = if offset.dot(depth) > half_depth { 1.0 } else { 0.0 };
            let final_half_w = half_w_near + far_mask * (far_half_w - half_w_near);
            let final_half_h = half_h_near + far_mask * (far_half_h - half_h_near);
            let center = center_near.add(depth * (far_mask * 2.0 * half_depth));
            spatial_frame_vertices[i] = center
                .add(right * (x_sign * final_half_w))
                .add(up * (y_sign * final_half_h));
        }
    }
}

pub fn world_to_ndc_space(
    main: &mut Camera3D,
    aspect: f32,
    near: f32,
    far: f32,
    world: &Model,
    ndc: &mut Model,
    rotation: f32,
    ortho_factor: f32,
    aspect_factor: f32,
    frame_dynamic_metrics: &mut FrameDynamicMetrics,
) {
    let (depth, right, up) = basis_vector(&main);
    let half_h_near = lerp(
        near * (FOVY_PERSPECTIVE * 0.5).to_radians().tan(),
        0.5 * NEAR_PLANE_HEIGHT_ORTHOGRAPHIC(),
        ortho_factor,
    );
    let half_w_near = lerp(half_h_near, half_h_near * aspect, aspect_factor);
    let half_depth_ndc = lerp(half_h_near, 0.5 * (far - near), lerp(aspect_factor, 0.0, ortho_factor));
    let center_near_plane = main.position.add(depth * near);
    let center_ndc_cube = center_near_plane.add(depth * half_depth_ndc);

    let world_mesh = &world.meshes()[0];
    let ndc_mesh = &mut ndc.meshes_mut()[0];
    let world_vertices = world_mesh.vertices();
    let ndc_vertices = ndc_mesh.vertices_mut();

    for [a, b, c] in world_mesh.triangles() {
        for &i in [a, b, c].iter() {
            let world_vertex = translate_rotate_scale(0, world_vertices[i], MODEL_POS, MODEL_SCALE, rotation);
            let signed_depth = world_vertex.sub(main.position).dot(depth);

            let intersection_coord = intersect(main, near, world_vertex, ortho_factor);
            let clip_plane_vector = intersection_coord.sub(center_near_plane);
            let x_ndc = clip_plane_vector.dot(right) / half_w_near;
            let y_ndc = clip_plane_vector.dot(up) / half_h_near;
            let z_ndc = lerp(
                (far + near - 2.0 * far * near / signed_depth) / (far - near),
                2.0 * (signed_depth - near) / (far - near) - 1.0,
                ortho_factor,
            );
            let scaled_right = right * (x_ndc * half_w_near);
            let scaled_up = up * (y_ndc * half_h_near);
            let scaled_depth = depth * (z_ndc * half_depth_ndc);
            let offset = scaled_right.add(scaled_up).add(scaled_depth);
            let scaled_ndc_coord = center_ndc_cube.add(offset);
            ndc_vertices[i] = translate_rotate_scale(1, scaled_ndc_coord, MODEL_POS, MODEL_SCALE, rotation);
            frame_dynamic_metrics.vertex_positions_written += 1;
        }
    }
}

pub fn blend_world_and_ndc_vertices(
    world_model: &Model,
    ndc_model: &mut Model,
    s_blend: f32,
    frame_dynamic_metrics: &mut FrameDynamicMetrics,
) {
    let world_mesh = &world_model.meshes()[0];
    let ndc_mesh = &mut ndc_model.meshes_mut()[0];
    let world_vertices = world_mesh.vertices();
    let ndc_vertices = ndc_mesh.vertices_mut();

    for [a, b, c] in world_mesh.triangles() {
        for &i in [a, b, c].iter() {
            ndc_vertices[i].x = lerp(world_vertices[i].x, ndc_vertices[i].x, s_blend);
            ndc_vertices[i].y = lerp(world_vertices[i].y, ndc_vertices[i].y, s_blend);
            ndc_vertices[i].z = lerp(world_vertices[i].z, ndc_vertices[i].z, s_blend);
            frame_dynamic_metrics.vertex_positions_written += 1;
        }
    }
}

pub fn collect_deformed_vertex_samples(base_vertices: &[Vector3]) -> Vec<Vec<Vector3>> {
    let vertices = base_vertices;
    let mut mesh_samples = Vec::with_capacity(ROTATIONAL_SAMPLES_FOR_INV_PROJ);
    for i in 0..ROTATIONAL_SAMPLES_FOR_INV_PROJ {
        let sample_time = i as f32 * TIME_BETWEEN_SAMPLES;
        let sample_rotation = -ANGULAR_VELOCITY * sample_time;
        let mut mesh_sample = vertices.to_vec();
        rotate_vertices_in_plane_slice(&mut mesh_sample, sample_rotation);
        let radial_field = generate_silhouette_radial_field(sample_time);
        deform_vertices_with_radial_field(&mut mesh_sample, &radial_field);
        rotate_vertices_in_plane_slice(&mut mesh_sample, -sample_rotation);
        mesh_samples.push(mesh_sample);
    }
    mesh_samples
}

pub fn generate_silhouette_radial_field(i_time: f32) -> Vec<f32> {
    let mut radial_field = Vec::with_capacity(RADIAL_FIELD_SIZE);
    for i in 0..RADIAL_FIELD_SIZE {
        let radial_field_angle = (i as f32) * TAU / (RADIAL_FIELD_SIZE as f32);
        radial_field.push(deformed_silhouette_radius_at_angle(radial_field_angle, i_time));
    }
    let max_radius = radial_field.iter().cloned().fold(1e-6, f32::max);
    for radius in &mut radial_field {
        *radius /= max_radius;
    }
    radial_field
}

pub fn deform_vertices_with_radial_field(vertices: &mut [Vector3], radial_field: &[f32]) {
    if vertices.is_empty() {
        return;
    }
    for vertex in vertices.iter_mut() {
        let interpolated_radial_magnitude = interpolate_between_radial_field_elements(vertex.x, vertex.y, radial_field);
        vertex.x *= interpolated_radial_magnitude;
        vertex.y *= interpolated_radial_magnitude;
    }
}

pub fn interpolate_between_deformed_vertices(
    model: &mut Model,
    i_time: f32,
    vertex_samples: &[Vec<Vector3>],
    frame_dynamic_metrics: &mut FrameDynamicMetrics,
) {
    let target_mesh = &mut model.meshes_mut()[0];
    let duration = vertex_samples.len() as f32 * TIME_BETWEEN_SAMPLES;
    let time = i_time % duration;
    let frame = time / TIME_BETWEEN_SAMPLES;
    let current_frame = frame.floor() as usize % vertex_samples.len();
    let next_frame = (current_frame + 1) % vertex_samples.len();
    let weight = frame.fract();
    let vertices = target_mesh.vertices_mut();
    let vertex_count = vertices.len();
    for ((dst_vertex, src_vertex), next_vertex) in vertices
        .iter_mut()
        .zip(vertex_samples[current_frame].iter())
        .zip(vertex_samples[next_frame].iter())
    {
        dst_vertex.x = src_vertex.x * (1.0 - weight) + next_vertex.x * weight;
        dst_vertex.y = src_vertex.y * (1.0 - weight) + next_vertex.y * weight;
        dst_vertex.z = src_vertex.z * (1.0 - weight) + next_vertex.z * weight;
    }
    frame_dynamic_metrics.vertex_positions_written += vertex_count;
}

pub fn interpolate_between_radial_field_elements(sample_x: f32, sample_y: f32, radial_field: &[f32]) -> f32 {
    let radial_disk_angle = sample_y.atan2(sample_x).rem_euclid(TAU);
    let radial_index = radial_disk_angle / TAU * RADIAL_FIELD_SIZE as f32;
    let lower_index = radial_index.floor() as usize % RADIAL_FIELD_SIZE;
    let upper_index = (lower_index + 1) % RADIAL_FIELD_SIZE;
    let interpolation_toward_upper = radial_index.fract();
    radial_field[lower_index] * (1.0 - interpolation_toward_upper)
        + radial_field[upper_index] * interpolation_toward_upper
}

#[inline]
pub fn deformed_silhouette_radius_at_angle(radial_field_angle: f32, i_time: f32) -> f32 {
    let direction_vector = Vector2::new(radial_field_angle.cos(), radial_field_angle.sin());
    let phase = LIGHT_WAVE_AMPLITUDE_X.hypot(LIGHT_WAVE_AMPLITUDE_Y) + 2.0;
    let mut lower_phase_radius = 0.0_f32;
    let mut upper_phase_radius = UMBRAL_MASK_OUTER_RADIUS + phase;
    for _ in 0..8 {
        let current_radius = grid_phase_magnitude(
            &mut (UMBRAL_MASK_CENTER + direction_vector * upper_phase_radius),
            i_time,
        );
        if current_radius >= UMBRAL_MASK_OUTER_RADIUS {
            break;
        }
        upper_phase_radius *= 1.5;
    }
    for _ in 0..20 {
        let mid_phase_radius = 0.5 * (lower_phase_radius + upper_phase_radius);
        let current_radius =
            grid_phase_magnitude(&mut (UMBRAL_MASK_CENTER + direction_vector * mid_phase_radius), i_time);
        if current_radius >= UMBRAL_MASK_OUTER_RADIUS {
            upper_phase_radius = mid_phase_radius;
        } else {
            lower_phase_radius = mid_phase_radius;
        }
    }
    upper_phase_radius
}

#[inline]
pub fn grid_phase_magnitude(grid_coord: &mut Vector2, i_time: f32) -> f32 {
    let mut grid_phase = spatial_phase(*grid_coord);
    grid_phase += temporal_phase(i_time);
    *grid_coord += add_phase(grid_phase);
    grid_coord.distance(UMBRAL_MASK_CENTER)
}

#[inline]
pub fn rotate_vertices_in_plane_slice(vertices: &mut [Vector3], rotation: f32) {
    let (rotation_sin, rotation_cos) = rotation.sin_cos();
    for vertex in vertices {
        let (x0, z0) = (vertex.x, vertex.z);
        vertex.x = x0 * rotation_cos + z0 * rotation_sin;
        vertex.z = -x0 * rotation_sin + z0 * rotation_cos;
    }
}

pub fn update_blend(blend: &mut f32, dt: f32, target_on: bool) {
    if dt > 0.0 {
        let dir = if target_on { 1.0 } else { -1.0 };
        *blend = (*blend + dir * BLEND_SCALAR * dt).clamp(0.0, 1.0);
    }
}

#[inline]
pub fn spatial_phase(grid_coords: Vector2) -> Vector2 {
    Vector2::new(
        grid_coords.y * LIGHT_WAVE_SPATIAL_FREQ_X,
        grid_coords.x * LIGHT_WAVE_SPATIAL_FREQ_Y,
    )
}

#[inline]
pub fn temporal_phase(time: f32) -> Vector2 {
    Vector2::new(time * LIGHT_WAVE_TEMPORAL_FREQ_X, time * LIGHT_WAVE_TEMPORAL_FREQ_Y)
}

#[inline]
pub fn add_phase(phase: Vector2) -> Vector2 {
    Vector2::new(
        LIGHT_WAVE_AMPLITUDE_X * phase.x.cos(),
        LIGHT_WAVE_AMPLITUDE_Y * phase.y.sin(),
    )
}

pub fn calculate_average_ndc_z_shift(world_model: &Model, ndc_model: &Model) -> f32 {
    let world_vertices = world_model.meshes()[0].vertices();
    let ndc_vertices = ndc_model.meshes()[0].vertices();

    let mut total_z_shift = 0.0;
    let mut count = 0;

    for (world_v, ndc_v) in world_vertices.iter().zip(ndc_vertices.iter()) {
        total_z_shift += ndc_v.z - world_v.z;
        count += 1;
    }

    if count > 0 {
        total_z_shift / count as f32
    } else {
        0.0
    }
}

pub fn basis_vector(main: &Camera3D) -> (Vector3, Vector3, Vector3) {
    let depth = main.target.sub(main.position).normalize();
    let right = depth.cross(main.up).normalize();
    let up = right.cross(depth).normalize();
    (depth, right, up)
}

#[inline]
pub fn observed_line_of_sight(observer: &Camera3D) -> Vec3 {
    Vec3::new(
        observer.target.x - observer.position.x,
        observer.target.y - observer.position.y,
        observer.target.z - observer.position.z,
    )
    .normalize_or_zero()
}

#[inline]
pub fn triangle_normal(a: Vec3, b: Vec3, c: Vec3) -> Vec3 {
    (b - a).cross(c - a).normalize_or_zero()
}

#[inline]
pub fn rotate_point_about_axis(c: Vec3, axis: (Vec3, Vec3), theta: f32) -> Vec3 {
    let (a, b) = axis;
    let ab = b - a;
    let ab_axis_dir = ab.normalize_or_zero();
    let ac = c - a;
    let ac_z_component = ab_axis_dir.dot(ac) * ab_axis_dir;
    let ac_x_component = ac - ac_z_component;
    let ac_y_component = ab_axis_dir.cross(ac_x_component);
    let origin = a;
    let rotated_x_component = ac_x_component * theta.cos();
    let rotated_y_component = ac_y_component * theta.sin();
    let rotated_c = rotated_x_component + rotated_y_component + ac_z_component;
    origin + rotated_c
}

fn translate_rotate_scale(inverse: i32, coord: Vector3, pos: Vector3, scale: Vector3, rotation: f32) -> Vector3 {
    let matrix = Mat4::from_scale(scale) * Mat4::from_rotation_y(rotation) * Mat4::from_translation(pos);
    let result = if inverse != 0 { matrix.inverse() } else { matrix };
    result.transform_point3(coord)
}

fn intersect(main: &mut Camera3D, near: f32, world_coord: Vector3, ortho_factor: f32) -> Vector3 {
    let view_dir = main.target.sub(main.position).normalize();
    let main_camera_to_point = world_coord.sub(main.position);
    let depth_along_view = main_camera_to_point.dot(view_dir);
    let center_near_plane = main.position.add(view_dir * near);
    if depth_along_view <= 0.0 {
        return center_near_plane;
    }
    let scale_to_near = near / depth_along_view;
    let result_perspective = main.position.add(main_camera_to_point * scale_to_near);
    let result_ortho = world_coord.add(view_dir * (center_near_plane.sub(world_coord).dot(view_dir)));
    Vector3::new(
        result_perspective.x + (result_ortho.x - result_perspective.x) * ortho_factor,
        result_perspective.y + (result_ortho.y - result_perspective.y) * ortho_factor,
        result_perspective.z + (result_ortho.z - result_perspective.z) * ortho_factor,
    )
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

    let offset = Vector3::new(
        camera.position.x - camera.target.x,
        camera.position.y - camera.target.y,
        camera.position.z - camera.target.z,
    );

    let mut radius = (offset.x * offset.x + offset.y * offset.y + offset.z * offset.z).sqrt();
    let mut azimuth = offset.z.atan2(offset.x);
    let horizontal_radius = (offset.x * offset.x + offset.z * offset.z).sqrt();
    let mut elevation = offset.y.atan2(horizontal_radius);

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
