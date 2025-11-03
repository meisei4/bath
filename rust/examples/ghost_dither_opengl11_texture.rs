use std::f32::consts::FRAC_PI_2;

use asset_payload::SPHERE_PATH;

use bath::fixed_func::papercraft::{fold, /*recompute_unfold_into_existing_mesh,*/ unfold};
use bath::fixed_func::silhouette::{
    build_inverted_hull, collect_deformed_vertex_samples, draw_inverted_hull_guassian_silhouette_stack,
    interpolate_between_deformed_vertices, rotate_inverted_hull, ANGULAR_VELOCITY, FOVY_ORTHOGRAPHIC, MODEL_POS,
    MODEL_SCALE, TIME_BETWEEN_SAMPLES,
};
use bath::fixed_func::texture::{
    dither, generate_silhouette_texture, rotate_silhouette_texture_dither, screen_pass_dither, ScreenPassDither,
};
use bath::fixed_func::topology::observed_line_of_sight;
use bath::render::raylib::RaylibRenderer;
use bath::render::raylib_util::N64_WIDTH;
use bath::render::renderer::Renderer;

use raylib::camera::Camera3D;
use raylib::color::Color;
use raylib::consts::CameraProjection;
use raylib::consts::KeyboardKey::*;
use raylib::consts::MaterialMapIndex::MATERIAL_MAP_ALBEDO;
use raylib::consts::PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8;
use raylib::drawing::{RaylibDraw, RaylibDraw3D, RaylibMode3DExt};
use raylib::ffi::{rlDisableDepthMask, rlEnableDepthMask, CSSPalette};
use raylib::math::Vector3;
use raylib::models::{RaylibMaterial, RaylibMesh, RaylibModel};
use raylib::texture::Image;
use raylib::RaylibHandle;

/// --- Jugemu-style control constants ---
const ROLL_ROTATION_VELOCITY: f32 = 2.0;
const JUGEMU_LONGITUDINAL_ORBIT_SPEED: f32 = 1.5;
const JUGEMU_LATITUDINAL_ORBIT_SPEED: f32 = 1.0;
const JUGEMU_ZOOM_SPEED: f32 = 2.0;
const JUGEMU_MIN_RADIUS: f32 = 0.25;
const JUGEMU_MAX_RADIUS: f32 = 25.0;

// Orthographic “zoom” is fovy (ortho width/height)
const ORTHO_MIN_FOVY: f32 = 4.0;
const ORTHO_MAX_FOVY: f32 = 100.0;
const ORTHO_ZOOM_SPEED: f32 = 35.0;

/// When paused, render a fully-folded snapshot for clarity.
const FULLY_FOLDED_TIME: f32 = 1_000_000.0;

/// Basic vec helpers
fn v3_dot(a: Vector3, b: Vector3) -> f32 {
    a.x * b.x + a.y * b.y + a.z * b.z
}
fn v3_cross(a: Vector3, b: Vector3) -> Vector3 {
    Vector3::new(a.y * b.z - a.z * b.y, a.z * b.x - a.x * b.z, a.x * b.y - a.y * b.x)
}
fn v3_scale(a: Vector3, s: f32) -> Vector3 {
    Vector3::new(a.x * s, a.y * s, a.z * s)
}
fn v3_add(a: Vector3, b: Vector3) -> Vector3 {
    Vector3::new(a.x + b.x, a.y + b.y, a.z + b.z)
}
fn v3_sub(a: Vector3, b: Vector3) -> Vector3 {
    Vector3::new(a.x - b.x, a.y - b.y, a.z - b.z)
}
fn v3_len(a: Vector3) -> f32 {
    (a.x * a.x + a.y * a.y + a.z * a.z).sqrt()
}
fn v3_normalize(a: Vector3) -> Vector3 {
    let l = v3_len(a);
    if l > 0.0 {
        v3_scale(a, 1.0 / l)
    } else {
        a
    }
}

/// Rotate vector `v` around unit axis `axis` by `angle` (Rodrigues)
fn rotate_vector_about_axis(v: Vector3, axis: Vector3, angle: f32) -> Vector3 {
    let u = v3_normalize(axis);
    let cos_t = angle.cos();
    let sin_t = angle.sin();
    let term1 = v3_scale(v, cos_t);
    let term2 = v3_scale(v3_cross(u, v), sin_t);
    let term3 = v3_scale(u, v3_dot(u, v) * (1.0 - cos_t));
    v3_add(v3_add(term1, term2), term3)
}

/// Initial camera state for resetting
#[derive(Clone, Copy)]
struct InitialCam {
    position: Vector3,
    up: Vector3,
    fovy: f32,
    projection: CameraProjection,
}

/// Jugemu controls (works in both projections; ortho zoom edits fovy)
fn jugemu_controls(cam: &mut Camera3D, rl: &RaylibHandle, dt: f32, initial: InitialCam) {
    // Reset to initial camera state
    if rl.is_key_pressed(KEY_SPACE) {
        cam.position = initial.position;
        cam.target = Vector3::ZERO;
        cam.up = initial.up;
        cam.fovy = initial.fovy;
        cam.projection = initial.projection;
        return;
    }

    // Read spherical from current position
    let mut radius = v3_len(cam.position);
    let mut az = cam.position.z.atan2(cam.position.x);
    let hr = (cam.position.x * cam.position.x + cam.position.z * cam.position.z).sqrt();
    let mut el = cam.position.y.atan2(hr);

    // az/el
    if rl.is_key_down(KEY_LEFT) {
        az += JUGEMU_LONGITUDINAL_ORBIT_SPEED * dt;
    }
    if rl.is_key_down(KEY_RIGHT) {
        az -= JUGEMU_LONGITUDINAL_ORBIT_SPEED * dt;
    }
    if rl.is_key_down(KEY_UP) {
        el += JUGEMU_LATITUDINAL_ORBIT_SPEED * dt;
    }
    if rl.is_key_down(KEY_DOWN) {
        el -= JUGEMU_LATITUDINAL_ORBIT_SPEED * dt;
    }

    // roll about view axis
    let mut roll_delta = 0.0;
    if rl.is_key_down(KEY_A) {
        roll_delta -= ROLL_ROTATION_VELOCITY * dt;
    }
    if rl.is_key_down(KEY_D) {
        roll_delta += ROLL_ROTATION_VELOCITY * dt;
    }

    // zoom: keyboard + mouse wheel
    let wheel = rl.get_mouse_wheel_move();
    match cam.projection {
        CameraProjection::CAMERA_ORTHOGRAPHIC => {
            let mut fovy = cam.fovy;
            if rl.is_key_down(KEY_W) {
                fovy -= ORTHO_ZOOM_SPEED * dt;
            }
            if rl.is_key_down(KEY_S) {
                fovy += ORTHO_ZOOM_SPEED * dt;
            }
            if wheel.abs() > 0.0 {
                fovy -= wheel * ORTHO_ZOOM_SPEED * 0.75;
            }
            fovy = fovy.clamp(ORTHO_MIN_FOVY, ORTHO_MAX_FOVY);
            cam.fovy = fovy;
        },
        CameraProjection::CAMERA_PERSPECTIVE => {
            if rl.is_key_down(KEY_W) {
                radius -= JUGEMU_ZOOM_SPEED * dt;
            }
            if rl.is_key_down(KEY_S) {
                radius += JUGEMU_ZOOM_SPEED * dt;
            }
            if wheel.abs() > 0.0 {
                radius -= wheel * (JUGEMU_ZOOM_SPEED * 0.75);
            }
        },
        _ => {},
    }

    // clamp & rebuild position
    radius = radius.clamp(JUGEMU_MIN_RADIUS, JUGEMU_MAX_RADIUS);
    const EPS: f32 = 0.0001;
    el = el.clamp(-FRAC_PI_2 + EPS, FRAC_PI_2 - EPS);

    cam.position.x = radius * el.cos() * az.cos();
    cam.position.y = radius * el.sin();
    cam.position.z = radius * el.cos() * az.sin();

    // re-aim at origin, apply roll
    let view_dir = v3_normalize(v3_sub(Vector3::ZERO, cam.position));
    let new_up = v3_normalize(rotate_vector_about_axis(cam.up, view_dir, roll_delta));
    cam.target = Vector3::ZERO;
    cam.up = new_up;
}

fn main() {
    // --- time / state ---
    let mut i_time: f32 = 0.0;
    let mut mesh_rotation: f32 = 0.0;

    // Toggles
    let mut show_texture = true; // [T]
    let mut show_hull = true; // [H]
    let mut paused = false; // [P]  (folding only pauses; rotation continues)

    // --- renderer/window ---
    let mut render = RaylibRenderer::init(N64_WIDTH, N64_WIDTH);

    // --- camera: start ORTHO, but allow runtime toggle with [O] ---
    let mut observer = Camera3D {
        position: Vector3::new(0.0, 0.0, 2.0),
        target: Vector3::ZERO,
        up: Vector3::Y,
        fovy: FOVY_ORTHOGRAPHIC,
        projection: CameraProjection::CAMERA_ORTHOGRAPHIC,
    };
    let initial_cam = InitialCam {
        position: observer.position,
        up: observer.up,
        fovy: observer.fovy,
        projection: observer.projection,
    };

    // --- source model + animation samples ---
    let mut main_model = render
        .handle
        .load_model(&render.thread, SPHERE_PATH)
        .expect("failed to load SPHERE_PATH model");
    let mesh_samples = collect_deformed_vertex_samples(main_model.meshes()[0].vertices());
    interpolate_between_deformed_vertices(&mut main_model, i_time, &mesh_samples);

    // --- unfolded target model (we draw this) ---
    let initial_unfolded_mesh = unfold(&render.thread, &mut main_model.meshes_mut()[0]);
    let mut unfolded_model = render
        .handle
        .load_model_from_mesh(&render.thread, unsafe { initial_unfolded_mesh.make_weak() })
        .expect("failed to build unfolded model");

    // --- procedural silhouette texture (dithered) applied to unfolded model ---
    let mut silhouette_img = generate_silhouette_texture(N64_WIDTH, N64_WIDTH);
    dither(&mut silhouette_img);
    let silhouette_texture = render
        .handle
        .load_texture_from_image(&render.thread, &silhouette_img)
        .expect("load_texture_from_image failed");
    unfolded_model.materials_mut()[0].maps_mut()[MATERIAL_MAP_ALBEDO as usize].texture = *silhouette_texture;

    // --- full-screen dither staging (for the screen-space pass) ---
    let mut blank_image = Image::gen_image_color(
        render.handle.get_screen_width(),
        render.handle.get_screen_height(),
        Color::BLACK,
    );
    blank_image.set_format(PIXELFORMAT_UNCOMPRESSED_R8G8B8A8);
    let blit_texture = unsafe {
        render
            .handle
            .load_texture_from_image(&render.thread, &blank_image)
            .expect("load_texture_from_image (blank) failed")
            .make_weak()
    };
    let mut dither_staging = ScreenPassDither {
        blit_texture,
        is_initialized: true,
        staging_rgba_bytes: Vec::new(),
    };

    // --- inverted hull built from the source model ---
    let mut inverted_hull = build_inverted_hull(&mut render, &main_model);

    while !render.handle.window_should_close() {
        // edge toggles
        if render.handle.is_key_pressed(KEY_T) {
            show_texture = !show_texture;
        }
        if render.handle.is_key_pressed(KEY_H) {
            show_hull = !show_hull;
        }
        if render.handle.is_key_pressed(KEY_P) {
            paused = !paused;
        }
        if render.handle.is_key_pressed(KEY_O) {
            observer.projection = match observer.projection {
                CameraProjection::CAMERA_ORTHOGRAPHIC => {
                    observer.fovy = 45.0;
                    CameraProjection::CAMERA_PERSPECTIVE
                },
                _ => {
                    observer.fovy = FOVY_ORTHOGRAPHIC;
                    CameraProjection::CAMERA_ORTHOGRAPHIC
                },
            };
        }

        let dt = render.handle.get_frame_time();

        // Jugemu controls (Space resets to original camera view position)
        jugemu_controls(&mut observer, &render.handle, dt, initial_cam);

        // ---- IMPORTANT: rotation always runs; folding time freezes when paused
        mesh_rotation -= ANGULAR_VELOCITY * dt; // keep spinning even when paused
        if !paused {
            i_time += dt; // folding timeline advances only when not paused
        }

        // Animate the SOURCE model vertices (deformation tied to i_time)
        let duration = mesh_samples.len() as f32 * TIME_BETWEEN_SAMPLES;
        let time = i_time % duration;
        let _current_frame = (time / TIME_BETWEEN_SAMPLES).floor() as usize % mesh_samples.len();
        interpolate_between_deformed_vertices(&mut main_model, i_time, &mesh_samples);

        // Update inverted hull with current camera LOS
        if show_hull {
            let observed_los = observed_line_of_sight(&observer);
            rotate_inverted_hull(&main_model.meshes()[0], &mut inverted_hull, observed_los, mesh_rotation);
        }

        // Fold source -> unfolded target
        {
            let source_mesh: &mut raylib::models::WeakMesh = &mut main_model.meshes_mut()[0];
            let target_mesh: &mut raylib::models::WeakMesh = &mut unfolded_model.meshes_mut()[0];
            let fold_time = if paused { FULLY_FOLDED_TIME } else { i_time }; // paused shows fully-folded snapshot
            *target_mesh = unsafe { fold(&render.thread, source_mesh, fold_time, true).make_weak() };
        } // borrow ends here

        // Dithered texture rotation (match reference). Keep updating while paused so it tracks rotation.
        rotate_silhouette_texture_dither(
            &mut unfolded_model,
            &observer,
            mesh_rotation,
            render.handle.get_screen_width(),
            render.handle.get_screen_height(),
        );

        // draw
        let mut draw_handle = render.handle.begin_drawing(&render.thread);
        draw_handle.clear_background(Color::BLACK);

        draw_handle.draw_mode3D(observer, |mut rl3d| {
            if show_texture {
                rl3d.draw_model_ex(
                    &unfolded_model,
                    MODEL_POS,
                    Vector3::Y,
                    mesh_rotation.to_degrees(),
                    MODEL_SCALE,
                    Color::GREY,
                );
                rl3d.draw_model_wires_ex(
                    &unfolded_model,
                    MODEL_POS,
                    Vector3::Y,
                    mesh_rotation.to_degrees(),
                    MODEL_SCALE,
                    Color::BLACK,
                );
            } else {
                rl3d.draw_model_wires_ex(
                    &unfolded_model,
                    MODEL_POS,
                    Vector3::Y,
                    mesh_rotation.to_degrees(),
                    MODEL_SCALE,
                    Color::WHITE,
                );
            }

            if show_hull {
                unsafe {
                    rlDisableDepthMask();
                }
                draw_inverted_hull_guassian_silhouette_stack(&mut rl3d, &inverted_hull, mesh_rotation);
                unsafe {
                    rlEnableDepthMask();
                }
            }
        });

        // Final screen-space dither pass (gives the stippled look)
        screen_pass_dither(&mut draw_handle, &mut dither_staging);

        // HUD
        let help = format!(
            "[T] Texture:{}  [H] Hull:{}  [P] Paused:{}  [O] Ortho/Persp  [Space] Reset View\nW/S zoom | ←/→/↑/↓ orbit | A/D roll | MouseWheel zoom",
            if show_texture { "ON" } else { "OFF" },
            if show_hull { "ON" } else { "OFF" },
            if paused { "YES" } else { "NO" },
        );
        draw_handle.draw_text(&help, 8, 8, 16, Color::RAYWHITE);
    }
}
