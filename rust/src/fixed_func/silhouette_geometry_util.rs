use crate::fixed_func::silhouette_constants::{
    GRID_ORIGIN_UV_OFFSET, GRID_SCALE, LIGHT_WAVE_AMPLITUDE_X, LIGHT_WAVE_AMPLITUDE_Y, LIGHT_WAVE_SPATIAL_FREQ_X,
    LIGHT_WAVE_SPATIAL_FREQ_Y, LIGHT_WAVE_TEMPORAL_FREQ_X, LIGHT_WAVE_TEMPORAL_FREQ_Y, SILHOUETTE_RADII_RESOLUTION,
    UMBRAL_MASK_CENTER, UMBRAL_MASK_OUTER_RADIUS,
};

use raylib::math::glam::{Vec2, Vec3};
use raylib::math::{Vector2, Vector3};
use std::f32::consts::TAU;

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
        LIGHT_WAVE_AMPLITUDE_X * (phase.x).cos(),
        LIGHT_WAVE_AMPLITUDE_Y * (phase.y).sin(),
    )
}

#[inline]
pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[inline]
pub fn uv_to_grid_space(uv: Vector2) -> Vector2 {
    (uv - GRID_ORIGIN_UV_OFFSET) * GRID_SCALE
}

#[inline]
pub fn rotate_vertices(vertices: &mut Vec<Vector3>, rotation: f32) {
    for vertex in vertices {
        let (x0, z0) = (vertex.x, vertex.z);
        vertex.x = x0 * rotation.cos() + z0 * rotation.sin();
        vertex.z = -x0 * rotation.sin() + z0 * rotation.cos();
    }
}

#[inline]
pub fn warped_distance_from_mask_center_in_grid(grid_point: &mut Vector2, time_seconds: f32) -> f32 {
    let mut grid_phase = spatial_phase(*grid_point);
    grid_phase += temporal_phase(time_seconds);
    *grid_point += add_phase(grid_phase);
    grid_point.distance(UMBRAL_MASK_CENTER)
}

pub fn silhouette_uvs_polar(x: f32, y: f32, radii: &[f32]) -> (f32, f32) {
    let angle = y.atan2(x).rem_euclid(TAU);
    let sample_index = angle / TAU * (SILHOUETTE_RADII_RESOLUTION as f32);
    let i0 = sample_index.floor() as usize % SILHOUETTE_RADII_RESOLUTION;
    let i1 = (i0 + 1) % SILHOUETTE_RADII_RESOLUTION;
    let lerp_t = sample_index.fract();
    let edge_radius = radii[i0] * (1.0 - lerp_t) + radii[i1] * lerp_t;
    let distance_from_center = (x * x + y * y).sqrt();
    let u = sample_index / (SILHOUETTE_RADII_RESOLUTION as f32);
    let v = (distance_from_center / (edge_radius + 1e-6)).clamp(0.0, 1.0);
    (u, v)
}

#[inline]
pub fn silhouette_radius_at_angle(theta: f32, time_s: f32) -> f32 {
    let dir = Vector2::new(theta.cos(), theta.sin());
    let wiggle = LIGHT_WAVE_AMPLITUDE_X.hypot(LIGHT_WAVE_AMPLITUDE_Y) + 2.0;
    let mut lo = 0.0_f32;
    let mut hi = UMBRAL_MASK_OUTER_RADIUS + wiggle;
    for _ in 0..8 {
        let d = warped_distance_from_mask_center_in_grid(&mut (UMBRAL_MASK_CENTER + dir * hi), time_s);
        if d >= UMBRAL_MASK_OUTER_RADIUS {
            break;
        }
        hi *= 1.5;
    }
    for _ in 0..20 {
        let mid = 0.5 * (lo + hi);
        let d = warped_distance_from_mask_center_in_grid(&mut (UMBRAL_MASK_CENTER + dir * mid), time_s);
        if d >= UMBRAL_MASK_OUTER_RADIUS {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    hi
}

#[inline]
pub fn lift_dimension(vertex: Vec2) -> Vec3 {
    Vec3::new(vertex.x, vertex.y, 0.0)
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
    //rotate in the xy plane
    let rotated_c = rotated_x_component + rotated_y_component + ac_z_component;
    origin + rotated_c
}
