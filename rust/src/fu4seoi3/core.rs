use crate::fu4seoi3::config_and_state::*;
use raylib::consts::CameraProjection::CAMERA_ORTHOGRAPHIC;
use raylib::math::glam::Mat4;
use raylib::prelude::*;
use std::f32::consts::{FRAC_PI_2, PI, TAU};
use std::mem::size_of;
use std::ops::{Add, Sub};
use std::time::{SystemTime, UNIX_EPOCH};

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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum FieldDisrupter {
    DoorPrimary,
    Window,
    BackWall,
}

pub struct FieldSample {
    pub position: Vector3,
    pub direction: Vector2,
    pub magnitude: f32,
    pub dominant: FieldDisrupter,
    pub door_influence: f32,
    pub window_influence: f32,
    pub wall_influence: f32,
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
                    let (door_inf, window_inf, wall_inf) =
                        self.source_influences_at_point(center_pos, &door, window.as_ref());

                    self.field_samples.push(FieldSample {
                        position: center_pos,
                        direction: dir,
                        magnitude: mag,
                        dominant,
                        door_influence: door_inf,
                        window_influence: window_inf,
                        wall_influence: wall_inf,
                    });

                    for &(dx, dz) in &[(-0.5, -0.5), (0.5, -0.5), (-0.5, 0.5), (0.5, 0.5)] {
                        let pos = Vector3::new(center_pos.x + dx, base_y, center_pos.z + dz);
                        let (d2, m2) = self.compute_energy_at_point(pos, &door, window.as_ref());
                        let dominant2 = self.classify_dominant_disrupter(pos, &door, window.as_ref());
                        let (door_inf2, window_inf2, wall_inf2) =
                            self.source_influences_at_point(pos, &door, window.as_ref());

                        self.field_samples.push(FieldSample {
                            position: pos,
                            direction: d2,
                            magnitude: m2,
                            dominant: dominant2,
                            door_influence: door_inf2,
                            window_influence: window_inf2,
                            wall_influence: wall_inf2,
                        });
                    }
                }
            }
        }
    }

    fn converging_duct_to_opening(
        &self,
        point: Vector3,
        dir: Vector2,
        mag: f32,
        opening: &Opening,
    ) -> (Vector2, f32, f32) {
        if !matches!(opening.kind, OpeningKind::Window) {
            return (dir, mag, 0.0);
        }

        let center = opening.center();
        let p2d = Vector2::new(point.x, point.z);
        let c2d = Vector2::new(center.x, center.z);
        let normal = opening.normal();
        let tangent = opening.tangent();

        let to_point = p2d - c2d;

        let dist_from_window = to_point.dot(normal);
        if dist_from_window <= 0.0 {
            return (dir, mag, 0.0);
        }
        if dist_from_window > self.config.funnel_reach {
            return (dir, mag, 0.0);
        }

        let lateral = to_point.dot(tangent);
        let nd = dist_from_window / self.config.funnel_reach;
        let width_interp = nd.powf(self.config.funnel_curve_power);
        let funnel_radius = self.config.funnel_sink_radius
            + (self.config.funnel_catch_radius - self.config.funnel_sink_radius) * width_interp;

        if lateral.abs() > funnel_radius {
            return (dir, mag, 0.0);
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

        (new_dir, new_mag, weight)
    }

    fn apply_back_wall_redirect(&self, point: Vector3, dir: Vector2, mag: f32) -> (Vector2, f32, f32) {
        let origin = self.origin;
        let back_wall_z = origin.z;
        let dist = (point.z - back_wall_z).abs();
        let max_dist = self.config.wall_redirect_distance;
        if max_dist <= 0.0 || dist >= max_dist {
            return (dir, mag, 0.0);
        }

        let base = self.config.wall_redirect_strength.clamp(0.0, 1.0);
        let falloff = 1.0 - (dist / max_dist);
        let weight = base * falloff;
        let center_x = origin.x + self.w as f32 * 0.5;
        let lateral = point.x - center_x;
        let desired_dir = Vector2::new(if lateral >= 0.0 { 1.0 } else { -1.0 }, 0.0);
        let new_dir = blend_directions(dir, desired_dir, weight);
        (new_dir, mag, weight)
    }

    fn compute_energy_at_point(
        &self,
        point: Vector3,
        door: &Opening,
        maybe_window: Option<&Opening>,
    ) -> (Vector2, f32) {
        let (mut dir, mut mag) = self.rectangular_jet_from_opening(point, door);

        if let Some(win) = maybe_window {
            let (d, m, _weight) = self.converging_duct_to_opening(point, dir, mag, win);
            dir = d;
            mag = m;
        }

        let (d, m, _weight) = self.apply_back_wall_redirect(point, dir, mag);
        dir = d;
        mag = m;

        (dir.normalize_or_zero(), mag)
    }
    fn source_influences_at_point(
        &self,
        point: Vector3,
        door: &Opening,
        maybe_window: Option<&Opening>,
    ) -> (f32, f32, f32) {
        let (door_dir, mag_door) = self.rectangular_jet_from_opening(point, door);

        let mut dir_after_window = door_dir;
        let mut mag_after_window = mag_door;
        let mut window_inf = 0.0;

        if let Some(win) = maybe_window {
            let (dw, mw, window_weight) =
                self.converging_duct_to_opening(point, dir_after_window, mag_after_window, win);

            let mut window_angle_inf = 0.0;
            if dir_after_window.length_squared() > 0.0 && dw.length_squared() > 0.0 {
                let angle = angle_between(dir_after_window, dw);
                window_angle_inf = (angle / FRAC_PI_2).clamp(0.0, 1.0);
            }

            let window_strength_inf = window_weight.clamp(0.0, 1.0);

            window_inf = window_angle_inf.max(window_strength_inf);

            dir_after_window = dw;
            mag_after_window = mw;
        }

        let mut wall_inf = 0.0;
        if dir_after_window.length_squared() > 0.0 {
            let (dir_after_wall, _mag_after_wall, wall_weight) =
                self.apply_back_wall_redirect(point, dir_after_window, mag_after_window);

            let mut wall_angle_inf = 0.0;
            if dir_after_wall.length_squared() > 0.0 {
                let wall_angle = angle_between(dir_after_window, dir_after_wall);
                wall_angle_inf = (wall_angle / FRAC_PI_2).clamp(0.0, 1.0);
            }

            let wall_strength_inf = wall_weight.clamp(0.0, 1.0);
            wall_inf = wall_angle_inf.max(wall_strength_inf);
        }

        let door_inf = if mag_door > 0.0 { 1.0 } else { 0.0 };

        (door_inf, window_inf, wall_inf)
    }

    pub fn classify_dominant_disrupter(
        &self,
        point: Vector3,
        door: &Opening,
        maybe_window: Option<&Opening>,
    ) -> FieldDisrupter {
        let (_door_present, window_inf, wall_inf) = self.source_influences_at_point(point, door, maybe_window);

        if window_inf < 0.05 && wall_inf < 0.05 {
            return FieldDisrupter::DoorPrimary;
        }
        if window_inf >= wall_inf {
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
