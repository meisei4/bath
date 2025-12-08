use crate::fu4seoi3::config_and_state::*;
use raylib::consts::CameraProjection::CAMERA_ORTHOGRAPHIC;
use raylib::math::glam::Mat4;
use raylib::prelude::*;
use std::f32::consts::{FRAC_PI_2, PI, TAU};
use std::mem::size_of;
use std::ops::{Add, Sub};

pub struct Field {
    pub boundary: FieldBoundary,
    pub entities: Vec<FieldEntity>,
    pub config: FieldConfig,
    operators: Vec<Box<dyn FieldOperator>>,
    pub samples: Vec<FieldSample>,
}

impl Field {
    pub fn new(boundary: FieldBoundary, entities: Vec<FieldEntity>, config: FieldConfig) -> Self {
        Field {
            boundary,
            entities,
            config,
            operators: Vec::new(),
            samples: Vec::new(),
        }
    }

    #[inline]
    pub fn cell_center(&self, ix: i32, iy: i32, iz: i32) -> Vector3 {
        Vector3::new(
            self.boundary.origin.x + ix as f32 + 0.5,
            self.boundary.origin.y + iy as f32 + 0.5,
            self.boundary.origin.z + iz as f32 + 0.5,
        )
    }

    pub fn sample_at(&self, point: Vector3) -> FieldSample {
        let mut accumulator = FieldOperationAccumulator::default();
        for operator in &self.operators {
            operator.apply(point, &mut accumulator, self);
        }
        FieldSample::from_accumulator(point, accumulator)
    }

    pub fn regenerate_samples(&mut self, sample_height: f32) {
        let (w, h, d) = (self.boundary.w as i32, self.boundary.h as i32, self.boundary.d as i32);
        let mut samples = Vec::new();
        let base_y = self.boundary.origin.y + sample_height;

        for iy in 0..h {
            if iy != 0 {
                continue;
            }
            for iz in 0..d {
                for ix in 0..w {
                    let center = self.cell_center(ix, iy, iz);
                    let center_pos = Vector3::new(center.x, base_y, center.z);
                    samples.push(self.sample_at(center_pos));

                    for &(dx, dz) in &[(-0.5, -0.5), (0.5, -0.5), (-0.5, 0.5), (0.5, 0.5)] {
                        let pos = Vector3::new(center_pos.x + dx, base_y, center_pos.z + dz);
                        samples.push(self.sample_at(pos));
                    }
                }
            }
        }

        self.samples = samples;
    }
}

#[derive(Default)]
struct FieldOperationAccumulator {
    direction: Vector2,
    magnitude: f32,
    door_component: f32,
    window_component: f32,
    wall_component: f32,
}

pub struct FieldSample {
    pub position: Vector3,
    pub direction: Vector2,
    pub magnitude: f32,
    pub dominant_field_operator: FieldOperatorKind,
    pub door_component: f32,
    pub window_component: f32,
    pub wall_component: f32,
}

impl FieldSample {
    fn from_accumulator(position: Vector3, accumulator: FieldOperationAccumulator) -> Self {
        let dominant = if accumulator.window_component < 0.05 && accumulator.wall_component < 0.05 {
            FieldOperatorKind::Emit
        } else if accumulator.window_component >= accumulator.wall_component {
            FieldOperatorKind::Absorb
        } else {
            FieldOperatorKind::Scatter
        };

        FieldSample {
            position,
            direction: accumulator.direction.normalize_or_zero(),
            magnitude: accumulator.magnitude,
            dominant_field_operator: dominant,
            door_component: accumulator.door_component,
            window_component: accumulator.window_component,
            wall_component: accumulator.wall_component,
        }
    }
}

pub struct FieldBoundary {
    pub origin: Vector3,
    pub w: f32,
    pub h: f32,
    pub d: f32,
}

#[derive(Clone, Copy)]
pub enum FieldEntityKind {
    Door { primary: bool },
    Window,
    BackWall,
}

pub struct FieldEntity {
    pub p0: Vector3, //TODO: how to consolidate this anchoring stuff with any kind of entity? seems fine? like bones? idk
    pub p1: Vector3,
    pub h0: f32,
    pub kind: FieldEntityKind,
    pub model_index: Option<usize>,
}

impl FieldEntity {
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
            FieldEntityKind::Door { .. } => Vector3::new(mid.x, self.p0.y + self.h0, mid.z),
            FieldEntityKind::Window => Vector3::new(mid.x, self.p0.y + room.h as f32 * 0.5, mid.z),
            FieldEntityKind::BackWall => Vector3::new(mid.x, mid.y, mid.z),
        }
    }

    pub fn rotation_into_room(&self, room: &Room) -> Option<f32> {
        if matches!(self.kind, FieldEntityKind::BackWall) {
            return None;
        }

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
        let mut _d_min = north_dist;

        if south_dist < _d_min {
            _d_min = south_dist;
            wall = 2;
        }
        if east_dist < _d_min {
            _d_min = east_dist;
            wall = 3;
        }
        if west_dist < _d_min {
            _d_min = west_dist;
            wall = 4;
        }

        let rotation = match wall {
            1 => 180.0, // north: +Z -> -Z
            2 => 0.0,   // south: +Z -> +Z
            3 => -90.0, // east:  +Z -> -X
            4 => 90.0,  // west:  +Z -> +X
            _ => 0.0,
        };

        Some(rotation)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum FieldOperatorKind {
    Emit,
    Absorb,
    Scatter,
}

impl FieldOperatorKind {
    pub fn color(self) -> Color {
        match self {
            FieldOperatorKind::Emit => ANAKIWA,
            FieldOperatorKind::Absorb => PALE_CANARY,
            FieldOperatorKind::Scatter => CHESTNUT_ROSE,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            FieldOperatorKind::Emit => "emit",
            FieldOperatorKind::Absorb => "absorb",
            FieldOperatorKind::Scatter => "scatter",
        }
    }
}

trait FieldOperator {
    fn apply(&self, point: Vector3, accumulator: &mut FieldOperationAccumulator, field: &Field);
}

pub struct Emit {
    pub field_entity_index: usize,
}

impl FieldOperator for Emit {
    fn apply(&self, point: Vector3, accumulator: &mut FieldOperationAccumulator, field: &Field) {
        let field_entities = &field.entities;
        if self.field_entity_index >= field_entities.len() {
            return;
        }

        let field_entity = &field_entities[self.field_entity_index];
        if !matches!(field_entity.kind, FieldEntityKind::Door { .. }) {
            return;
        }

        let cfg = &field.config;
        let center = field_entity.center();
        let p2d = Vector2::new(point.x, point.z);
        let c2d = Vector2::new(center.x, center.z);
        let to_point = p2d - c2d;
        let jet_normal = field_entity.normal();
        let jet_tangent = field_entity.tangent();
        let forward_dist = to_point.dot(jet_normal);
        let lateral_offset = to_point.dot(jet_tangent);
        if forward_dist <= 0.0 || forward_dist > cfg.jet_max_distance {
            return;
        }

        let half_width = field_entity.width() * 0.5;
        let spread_rad = cfg.jet_spread_angle.to_radians();
        let spread_amount = forward_dist * spread_rad.tan();
        let jet_half_width = half_width + spread_amount;
        if lateral_offset.abs() > jet_half_width {
            return;
        }

        let dir = jet_normal;
        let edge = 1.0 - (lateral_offset.abs() / jet_half_width).powf(2.0);
        let dist = 1.0 - (forward_dist / cfg.jet_max_distance);
        let mag = cfg.jet_strength * edge * dist;
        if mag <= 0.0 {
            return;
        }

        accumulator.direction = dir;
        accumulator.magnitude = mag;
        accumulator.door_component = 1.0;
    }
}

pub struct Absorb {
    pub field_entity_index: usize,
}

impl FieldOperator for Absorb {
    fn apply(&self, point: Vector3, accumulator: &mut FieldOperationAccumulator, field: &Field) {
        let field_entities = &field.entities;
        if self.field_entity_index >= field_entities.len() {
            return;
        }

        let field_entity = &field_entities[self.field_entity_index];
        if !matches!(field_entity.kind, FieldEntityKind::Window) {
            return;
        }

        let cfg = &field.config;
        let dir_before = accumulator.direction;
        let mag_before = accumulator.magnitude;
        let center = field_entity.center();
        let p2d = Vector2::new(point.x, point.z);
        let c2d = Vector2::new(center.x, center.z);
        let normal = field_entity.normal();
        let tangent = field_entity.tangent();
        let to_point = p2d - c2d;
        let dist_from_window = to_point.dot(normal);
        if dist_from_window <= 0.0 || dist_from_window > cfg.funnel_reach {
            return;
        }

        let lateral = to_point.dot(tangent);
        let nd = dist_from_window / cfg.funnel_reach;
        let width_interp = nd.powf(cfg.funnel_curve_power);
        let funnel_radius = cfg.funnel_sink_radius + (cfg.funnel_catch_radius - cfg.funnel_sink_radius) * width_interp;
        if lateral.abs() > funnel_radius {
            return;
        }

        let target_lateral = if funnel_radius > 0.0 {
            lateral * (cfg.funnel_sink_radius / funnel_radius)
        } else {
            0.0
        };

        let target_point = c2d + tangent * target_lateral;
        let desired_dir = (target_point - p2d).normalize_or_zero();
        let proximity = 1.0 - nd;
        let lateral_factor = 1.0 - (lateral.abs() / funnel_radius).powf(2.0);
        let weight = cfg.funnel_strength * proximity * lateral_factor;
        let dir_after = blend_directions(dir_before, desired_dir, weight);
        let mag_after = if mag_before == 0.0 {
            weight * cfg.funnel_strength
        } else {
            mag_before
        };

        let window_strength_weight = weight.clamp(0.0, 1.0);
        let mut window_angle_weight = 0.0;
        if dir_before.length_squared() > 0.0 && dir_after.length_squared() > 0.0 {
            let angle = angle_between(dir_before, dir_after);
            window_angle_weight = (angle / FRAC_PI_2).clamp(0.0, 1.0);
        }

        let greater_weight = window_angle_weight.max(window_strength_weight);
        if greater_weight > accumulator.window_component {
            accumulator.window_component = greater_weight;
        }

        accumulator.direction = dir_after;
        accumulator.magnitude = mag_after;
    }
}
pub struct Scatter;

impl FieldOperator for Scatter {
    fn apply(&self, point: Vector3, accumulator: &mut FieldOperationAccumulator, field: &Field) {
        let dir_before = accumulator.direction;
        let mag_before = accumulator.magnitude;
        let cfg = &field.config;
        let back_wall_z = field.boundary.origin.z;
        let dist = (point.z - back_wall_z).abs();
        let max_dist = cfg.wall_redirect_distance;
        if max_dist <= 0.0 || dist >= max_dist {
            return;
        }

        let base = cfg.wall_redirect_strength.clamp(0.0, 1.0);
        let falloff = 1.0 - (dist / max_dist);
        let weight = base * falloff;
        let center_x = field.boundary.origin.x + field.boundary.w * 0.5;
        let lateral = point.x - center_x;
        let desired_dir = Vector2::new(if lateral >= 0.0 { 1.0 } else { -1.0 }, 0.0);
        let dir_after = blend_directions(dir_before, desired_dir, weight);
        let mag_after = mag_before;
        let wall_weight = weight;
        if dir_before.length_squared() > 0.0 && dir_after.length_squared() > 0.0 {
            let angle = angle_between(dir_before, dir_after);
            let wall_angle_weight = (angle / FRAC_PI_2).clamp(0.0, 1.0);
            let wall_strength_weight = wall_weight.clamp(0.0, 1.0);
            let greater_weight = wall_angle_weight.max(wall_strength_weight);
            if greater_weight > accumulator.wall_component {
                accumulator.wall_component = greater_weight;
            }
        }

        accumulator.direction = dir_after;
        accumulator.magnitude = mag_after;
    }
}

pub struct Room {
    pub w: i32,
    pub h: i32,
    pub d: i32,
    pub origin: Vector3,
    pub field: Field,
}

impl Default for Room {
    fn default() -> Self {
        let origin = Vector3::new(-(ROOM_W as f32) / 2.0, -(ROOM_H as f32) / 2.0, -(ROOM_D as f32) / 2.0);
        let boundary = FieldBoundary {
            origin,
            w: ROOM_W as f32,
            h: ROOM_H as f32,
            d: ROOM_D as f32,
        };

        let north_z = origin.z + ROOM_D as f32;
        let west_x = origin.x;
        let center_x = origin.x + ROOM_W as f32 * 0.5;
        let center_z = origin.z + ROOM_D as f32 * 0.5;

        let door = FieldEntity {
            p0: Vector3::new(center_x - 1.0, origin.y, north_z),
            p1: Vector3::new(center_x + 1.0, origin.y, north_z),
            h0: 0.0,
            kind: FieldEntityKind::Door { primary: true },
            model_index: None,
        };

        let window = FieldEntity {
            p0: Vector3::new(west_x, origin.y, center_z - 1.5),
            p1: Vector3::new(west_x, origin.y, center_z + 1.5),
            h0: 0.0,
            kind: FieldEntityKind::Window,
            model_index: None,
        };

        let entities = vec![door, window];
        let mut field = Field::new(boundary, entities, FieldConfig::default());
        field.operators.push(Box::new(Emit { field_entity_index: 0 }));
        field.operators.push(Box::new(Absorb { field_entity_index: 1 }));
        field.operators.push(Box::new(Scatter));

        field.regenerate_samples(field.config.chi_sample_height);

        Room {
            w: ROOM_W,
            h: ROOM_H,
            d: ROOM_D,
            origin,
            field,
        }
    }
}

pub fn build_field_model_lines(handle: &mut RaylibHandle, thread: &RaylibThread, room: &Room) -> Model {
    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    let mut colors = Vec::new();
    let mut texcoords = Vec::new();

    for field_sample in room.field_samples() {
        vertices.push(field_sample.position);
        normals.push(Vector3::new(
            field_sample.direction.x,
            field_sample.magnitude,
            field_sample.direction.y,
        ));
        texcoords.push(Vector2::new(field_sample.door_component, field_sample.window_component));
        colors.push(field_sample.dominant_field_operator.color());
    }

    let mesh = Mesh::init_mesh(&vertices)
        .normals(&normals)
        .colors(&colors)
        .texcoords(&texcoords)
        .build_dynamic(thread)
        .expect("failed to build chi field mesh");

    handle
        .load_model_from_mesh(thread, mesh)
        .expect("failed to create chi field model")
}

pub fn build_field_model_ribbons(handle: &mut RaylibHandle, thread: &RaylibThread, room: &Room) -> Model {
    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    let mut colors = Vec::new();
    let mut texcoords = Vec::new();
    let base_length = room.field.config.chi_arrow_length;
    let base_half_width = base_length * 0.15;
    for field_sample in room.field_samples().iter() {
        let dir2 = field_sample.direction;
        if dir2.length_squared() < 1e-6 {
            continue;
        }
        let dir3 = Vector3::new(dir2.x, 0.0, dir2.y).normalize();
        let ortho = Vector3::new(-dir3.z, 0.0, dir3.x);
        let m = field_sample.magnitude.clamp(0.0, 1.0);
        let half_length = base_length * m * 0.5;
        let half_width = base_half_width;
        let center = field_sample.position;
        let p0 = center - dir3 * half_length - ortho * half_width;
        let p1 = center - dir3 * half_length + ortho * half_width;
        let p2 = center + dir3 * half_length + ortho * half_width;
        let p3 = center + dir3 * half_length - ortho * half_width;
        let st0 = Vector2::new(0.0, 0.0);
        let st1 = Vector2::new(0.0, 1.0);
        let st2 = Vector2::new(1.0, 1.0);
        let st3 = Vector2::new(1.0, 0.0);
        let red = field_sample.door_component;
        let green = field_sample.window_component;
        let blue = field_sample.wall_component;
        let sum = red + green + blue;
        let (mut r, mut g, mut b) = if sum > 0.0 {
            (red / sum, green / sum, blue / sum)
        } else {
            (0.0, 0.0, 0.0)
        };

        let brightness = 0.25 + 0.75 * m;
        r *= brightness;
        g *= brightness;
        b *= brightness;

        let color = Color {
            r: (r * 255.0).round() as u8,
            g: (g * 255.0).round() as u8,
            b: (b * 255.0).round() as u8,
            a: 255,
        };

        let normal = Vector3::new(0.0, 1.0, 0.0);
        let positions = [p0, p1, p2, p0, p2, p3];
        let dst_texcoords = [st0, st1, st2, st0, st2, st3];
        for i in 0..6 {
            vertices.push(positions[i]);
            normals.push(normal);
            texcoords.push(dst_texcoords[i]);
            colors.push(color);
        }
    }

    let mesh = Mesh::init_mesh(&vertices)
        .normals(&normals)
        .colors(&colors)
        .texcoords(&texcoords)
        .build_dynamic(thread)
        .expect("failed to build chi field mesh");

    handle
        .load_model_from_mesh(thread, mesh)
        .expect("failed to create chi field model")
}

pub fn build_field_model_arrows(
    handle: &mut RaylibHandle,
    thread: &RaylibThread,
    room: &Room,
    arrow_mesh: &WeakMesh,
) -> Model {
    let samples: &[FieldSample] = room.field_samples();
    let sample_count = samples.len();
    assert!(sample_count > 0, "need at least one field sample");
    let first_mesh = {
        let vertices: Vec<Vector3> = arrow_mesh.vertices().iter().map(|&v| v + samples[0].position).collect();
        Mesh::init_mesh(&vertices)
            .normals_opt(arrow_mesh.normals())
            .texcoords_opt(arrow_mesh.texcoords())
            .colors_opt(arrow_mesh.colors())
            .indices_opt(arrow_mesh.indices())
            .build_dynamic(thread)
            .unwrap()
    };

    let model = handle.load_model_from_mesh(thread, first_mesh).unwrap();
    let mut raw_model = model.to_raw();
    let mesh_size = (sample_count * size_of::<ffi::Mesh>()) as u32;
    let new_meshes = unsafe { ffi::MemAlloc(mesh_size) } as *mut ffi::Mesh;
    assert!(!new_meshes.is_null(), "MemAlloc failed for meshes");

    unsafe { std::ptr::copy_nonoverlapping(raw_model.meshes, new_meshes, 1) };
    unsafe { ffi::MemFree(raw_model.meshes as *mut std::ffi::c_void) };

    let mesh_material_size = (sample_count * size_of::<i32>()) as u32;
    let new_mesh_material = unsafe { ffi::MemAlloc(mesh_material_size) } as *mut i32;
    assert!(!new_mesh_material.is_null(), "MemAlloc failed for meshMaterial");

    for i in 0..sample_count {
        unsafe { std::ptr::write(new_mesh_material.add(i), 0) };
    }

    unsafe { ffi::MemFree(raw_model.meshMaterial as *mut std::ffi::c_void) };
    raw_model.meshes = new_meshes;
    raw_model.meshCount = sample_count as i32;
    raw_model.meshMaterial = new_mesh_material;
    for i in 1..sample_count {
        let vertices: Vec<Vector3> = arrow_mesh.vertices().iter().map(|&v| v + samples[i].position).collect();
        let mesh = Mesh::init_mesh(&vertices)
            .normals_opt(arrow_mesh.normals())
            .texcoords_opt(arrow_mesh.texcoords())
            .colors_opt(arrow_mesh.colors())
            .indices_opt(arrow_mesh.indices())
            .build_dynamic(thread)
            .unwrap();

        unsafe { std::ptr::write(new_meshes.add(i), mesh.to_raw()) };
    }

    unsafe { Model::from_raw(raw_model) }
}

pub fn update_field_model_arrows(
    field_model_arrows: &mut Model,
    room: &Room,
    arrow_mesh: &WeakMesh,
    dynamic_mesh_metrics: &mut DynamicMeshMetrics,
) {
    let samples = room.field_samples();
    let meshes = field_model_arrows.meshes_mut();

    assert_eq!(
        samples.len(),
        meshes.len(),
        "update_field_model_arrows: sample/mesh count mismatch (samples: {}, meshes: {})",
        samples.len(),
        meshes.len()
    );

    let current_vertices = arrow_mesh.vertices();
    let current_normals = arrow_mesh.normals().expect("arrow template must have prebaked normals");
    let vertex_count = current_vertices.len();

    const MIN_DIR_LEN2: f32 = 1e-6;
    const MIN_MAG: f32 = 0.01;

    let arrow_scale = room.field.config.chi_arrow_length;

    for (i, sample) in samples.iter().enumerate() {
        let mesh = &mut meshes[i];
        let mesh_vertex_count = mesh.vertices().len();

        debug_assert_eq!(
            mesh_vertex_count, vertex_count,
            "arrow vertex count mismatch at mesh {}",
            i
        );

        let dir2 = sample.direction;
        let dir_len2 = dir2.x * dir2.x + dir2.y * dir2.y;
        let has_dir = dir_len2 > MIN_DIR_LEN2;
        let m = sample.magnitude.clamp(0.0, 1.0);
        let active = has_dir && m > MIN_MAG;

        let (cos_yaw, sin_yaw) = if has_dir {
            let yaw = (-dir2.x).atan2(-dir2.y);
            (yaw.cos(), yaw.sin())
        } else {
            (1.0_f32, 0.0_f32)
        };

        let (r_f, g_f, b_f) = {
            let red = sample.door_component;
            let green = sample.window_component;
            let blue = sample.wall_component;
            let sum = red + green + blue;

            let (mut r, mut g, mut b) = if sum > 0.0 {
                (red / sum, green / sum, blue / sum)
            } else {
                (0.0, 0.0, 0.0)
            };

            let brightness = 0.25 + 0.75 * m;
            r *= brightness;
            g *= brightness;
            b *= brightness;

            (r, g, b)
        };

        let vertex_color = Color {
            r: (r_f * 255.0).round() as u8,
            g: (g_f * 255.0).round() as u8,
            b: (b_f * 255.0).round() as u8,
            a: 255,
        };

        let center = sample.position;
        let mut new_vertices = Vec::with_capacity(vertex_count);
        let mut new_normals = Vec::with_capacity(vertex_count);
        let mut new_colors = Vec::with_capacity(vertex_count);

        for idx in 0..vertex_count {
            if !active {
                new_vertices.push(center);
                new_normals.push(Vector3::ZERO);
                new_colors.push(Color { r: 0, g: 0, b: 0, a: 0 });
                continue;
            }

            let base_v = current_vertices[idx];
            let sx = base_v.x * arrow_scale;
            let sy = base_v.y * arrow_scale;
            let sz = base_v.z * arrow_scale;

            let rx = sx * cos_yaw + sz * sin_yaw;
            let rz = -sx * sin_yaw + sz * cos_yaw;

            new_vertices.push(Vector3::new(center.x + rx, center.y + sy, center.z + rz));

            let base_n = current_normals[idx];
            let nx0 = base_n.x;
            let ny0 = base_n.y;
            let nz0 = base_n.z;
            let nx = nx0 * cos_yaw + nz0 * sin_yaw;
            let nz = -nx0 * sin_yaw + nz0 * cos_yaw;

            new_normals.push(Vector3::new(nx, ny0, nz));
            new_colors.push(vertex_color);
        }

        mesh.vertices_mut().copy_from_slice(&new_vertices);
        mesh.normals_mut().unwrap().copy_from_slice(&new_normals);
        mesh.colors_mut().unwrap().copy_from_slice(&new_colors);

        dynamic_mesh_metrics.warm_vertex_positions_written += new_vertices.len();
        dynamic_mesh_metrics.warm_vertex_normals_written += new_normals.len();
        dynamic_mesh_metrics.warm_vertex_colors_written += new_colors.len();
    }
}

impl Room {
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
        self.field.config = config;
        self.generate_chi_field();
    }

    pub fn generate_chi_field(&mut self) {
        let sample_height = self.field.config.chi_sample_height;
        self.field.regenerate_samples(sample_height);
    }

    pub fn get_dominant_field_operator_at(&self, point: Vector3) -> FieldOperatorKind {
        self.field.sample_at(point).dominant_field_operator
    }

    pub fn field_samples(&self) -> &[FieldSample] {
        &self.field.samples
    }
}

#[derive(Copy, Clone)]
pub struct StaticMeshMetrics {
    pub cold_vertex_count: usize,
    pub cold_triangle_count: usize,
    pub cold_normal_count: usize,
    pub cold_texcoord_count: usize,
    pub cold_color_count: usize,
    pub cold_index_count: usize,
    pub cold_total_bytes: usize,
}

impl StaticMeshMetrics {
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

        StaticMeshMetrics {
            cold_vertex_count: vertex_count,
            cold_triangle_count: triangle_count,
            cold_normal_count: normal_count,
            cold_texcoord_count: texcoord_count,
            cold_color_count: color_count,
            cold_index_count: index_count,
            cold_total_bytes: total_bytes,
        }
    }
}

pub fn gpu_vertex_stride_bytes(metrics: &StaticMeshMetrics) -> usize {
    let mut stride = size_of::<Vector3>();
    if metrics.cold_normal_count > 0 {
        stride += size_of::<Vector3>();
    }
    if metrics.cold_texcoord_count > 0 {
        stride += size_of::<Vector2>();
    }
    if metrics.cold_color_count > 0 {
        stride += size_of::<Color>();
    }
    stride
}

#[derive(Copy, Clone, Default)]
pub struct DynamicMeshMetrics {
    pub hot_vertex_positions_written: usize,
    pub hot_vertex_normals_written: usize,
    pub hot_vertex_colors_written: usize,

    pub warm_vertex_positions_written: usize,
    pub warm_vertex_normals_written: usize,
    pub warm_vertex_colors_written: usize,

    pub warm_anim_sample_count: usize,
    pub warm_anim_verts_per_sample: usize,
    pub warm_anim_total_bytes: usize,

    pub warm_arrow_instance_count: usize,
    pub warm_arrow_verts_per_instance: usize,
    pub warm_arrow_total_verts: usize,
    pub warm_arrow_total_bytes: usize,
}

impl DynamicMeshMetrics {
    pub fn hot_reset(&mut self) {
        self.hot_vertex_positions_written = 0;
        self.hot_vertex_normals_written = 0;
        self.hot_vertex_colors_written = 0;
    }

    pub fn warm_reset(&mut self) {
        self.warm_vertex_positions_written = 0;
        self.warm_vertex_normals_written = 0;
        self.warm_vertex_colors_written = 0;
    }

    pub fn update_arrow_pool(&mut self, arrow_mesh: &WeakMesh, instance_count: usize) {
        let verts_per_instance = arrow_mesh.vertices().len();
        self.warm_arrow_instance_count = instance_count;
        self.warm_arrow_verts_per_instance = verts_per_instance;
        self.warm_arrow_total_verts = instance_count * verts_per_instance;
        self.warm_arrow_total_bytes = self.warm_arrow_total_verts * (size_of::<Vector3>() * 2 + size_of::<Color>());
    }

    pub fn total_bytes_written(&self) -> usize {
        self.hot_vertex_positions_written * size_of::<Vector3>()
            + self.hot_vertex_normals_written * size_of::<Vector3>()
            + self.hot_vertex_colors_written * size_of::<Color>()
    }
    pub fn measure_animation(mesh_samples: &[Vec<Vector3>]) -> Self {
        let mut dynamic_mesh_metrics = DynamicMeshMetrics::default();
        dynamic_mesh_metrics.update_animation(mesh_samples);
        dynamic_mesh_metrics
    }

    pub fn update_animation(&mut self, mesh_samples: &[Vec<Vector3>]) {
        if mesh_samples.is_empty() {
            self.warm_anim_sample_count = 0;
            self.warm_anim_verts_per_sample = 0;
            self.warm_anim_total_bytes = 0;
            return;
        }

        let sample_count = mesh_samples.len();
        let verts_per_sample = mesh_samples[0].len();
        let total_bytes = sample_count * verts_per_sample * size_of::<Vector3>();

        self.warm_anim_sample_count = sample_count;
        self.warm_anim_verts_per_sample = verts_per_sample;
        self.warm_anim_total_bytes = total_bytes;
    }
}

pub struct Layer3MeshData {
    pub name: &'static str,
    pub world: Model,
    pub ndc: Model,
    pub texture: Texture2D,
    pub static_metrics_world: StaticMeshMetrics,
    pub static_metrics_ndc: StaticMeshMetrics,
    pub combined_bytes: usize,
    pub z_shift_anisotropic: f32,
    pub z_shift_isotropic: f32,
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
    dynamic_mesh_metrics: &mut DynamicMeshMetrics,
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
    dynamic_mesh_metrics.hot_vertex_positions_written += out_vertices.len();
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
    dynamic_mesh_metrics: &mut DynamicMeshMetrics,
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
            dynamic_mesh_metrics.hot_vertex_positions_written += 1;
        }
    }
}

pub fn blend_world_and_ndc_vertices(
    world_model: &Model,
    ndc_model: &mut Model,
    blend: f32,
    dynamic_mesh_metrics: &mut DynamicMeshMetrics,
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
            dynamic_mesh_metrics.hot_vertex_positions_written += 1;
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
    let angle = sample_y.atan2(sample_x).rem_euclid(TAU);
    let index = angle / TAU * RADIAL_FIELD_SIZE as f32;
    let i0 = index.floor() as usize % RADIAL_FIELD_SIZE;
    let i1 = (i0 + 1) % RADIAL_FIELD_SIZE;
    let t = index.fract();
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
    dynamic_mesh_metrics: &mut DynamicMeshMetrics,
    _field_config: &FieldConfig,
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

    dynamic_mesh_metrics.hot_vertex_positions_written += dst.len();
}

pub fn calculate_average_ndc_z_shift(world_model: &Model, ndc_model: &Model) -> f32 {
    let world_vertices = world_model.meshes()[0].vertices();
    let ndc_vertices = ndc_model.meshes()[0].vertices();
    let mut sum = 0.0;
    let mut count = 0usize;

    for (a, b) in world_vertices.iter().zip(ndc_vertices.iter()) {
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

fn blend_directions(current: Vector2, target: Vector2, weight: f32) -> Vector2 {
    let w = weight.clamp(0.0, 1.0);
    let c = current.normalize_or_zero();
    let t = target.normalize_or_zero();
    c.lerp(t, w).normalize_or_zero()
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

fn angle_between(a: Vector2, b: Vector2) -> f32 {
    let an = a.normalize_or_zero();
    let bn = b.normalize_or_zero();
    let dot = (an.x * bn.x + an.y * bn.y).clamp(-1.0, 1.0);
    dot.acos()
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

pub fn fill_room_with_ghosts(
    room_w: i32,
    room_h: i32,
    room_d: i32,
    placed_time: f32,
    texture_enabled: bool,
    color_enabled: bool,
) -> Vec<PlacedCell> {
    let mut cells = Vec::new();
    for ix in 0..room_w {
        for iy in 0..room_h {
            for iz in 0..room_d {
                cells.push(PlacedCell {
                    ix,
                    iy,
                    iz,
                    mesh_index: 0,
                    placed_time,
                    settled: false,
                    texture_enabled,
                    color_enabled,
                });
            }
        }
    }
    cells
}
