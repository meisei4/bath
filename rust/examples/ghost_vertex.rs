use asset_payload::SPHERE_PATH;
use bath::fixed_func::ghost::{
    add_phase, spatial_phase, temporal_phase, uv_to_grid_space, LIGHT_WAVE_AMPLITUDE_Y, UMBRAL_MASK_CENTER,
    UMBRAL_MASK_OUTER_RADIUS,
};
use bath::render::raylib::RaylibRenderer;
use bath::render::raylib_util::{flip_framebuffer, N64_HEIGHT, N64_WIDTH, ORIGIN};
use bath::render::renderer::Renderer;
use raylib::camera::Camera3D;
use raylib::color::Color;
use raylib::consts::CameraProjection;
use raylib::drawing::{RaylibDraw, RaylibDraw3D, RaylibMode3DExt};
use raylib::math::{Matrix, Vector2, Vector3};
use raylib::models::{Model, RaylibMesh, RaylibModel, WeakMesh};
use raylib::prelude::Image;
use std::f32::consts::PI;

fn main() {
    let mut render = RaylibRenderer::init(N64_WIDTH, N64_HEIGHT);
    let screen_w = render.handle.get_screen_width();
    let screen_h = render.handle.get_screen_height();
    let i_resolution = Vector2::new(screen_w as f32, screen_h as f32);
    let mut i_time = 0.0f32;
    let circle_img = generate_circle_image(screen_w, screen_h, i_time);
    let texture = render
        .handle
        .load_texture_from_image(&render.thread, &circle_img)
        .unwrap();
    let mut model = render.handle.load_model(&render.thread, SPHERE_PATH).unwrap();
    let observer = Camera3D {
        position: Vector3::new(1.0, 0.0, 1.0),
        target: Vector3::ZERO,
        up: Vector3::Y,
        fovy: 45.0,
        projection: CameraProjection::CAMERA_PERSPECTIVE,
    };
    //let model_pos = Vector3::new(0.25, 0.30, -0.25);
    //let model_scale = Vector3::new(0.25, 0.25, 0.25);
    let mesh = &mut model.meshes_mut()[0]; //only one mesh for the sphere
    let max_extent = normalize_mesh(mesh);
    let d = {
        let v = observer.position - observer.target;
        (v.x * v.x + v.y * v.y + v.z * v.z).sqrt()
    };
    let fovy_rad = observer.fovy.to_radians();
    let r_px = UMBRAL_MASK_OUTER_RADIUS * screen_h as f32;
    let cx_px = 0.33 * screen_h as f32;
    let cy_px = 0.01 * screen_h as f32;
    let h = d * (fovy_rad * 0.5).tan();
    let r_world = h * (2.0 * r_px / screen_h as f32) / 2.0;
    let x_ndc = 2.0 * cx_px / screen_w as f32 - 1.0;
    let y_ndc = -2.0 * cy_px / screen_h as f32 + 1.0;
    let aspect = screen_w as f32 / screen_h as f32;
    let x_world = x_ndc * h * aspect;
    let y_world = y_ndc * h;
    println!(
        "Sphere pos = ({:.3}, {:.3}, {:.3}), scale = {:.3}",
        x_world, y_world, -d, r_world
    );
    let model_pos = Vector3::new(x_world, y_world, -d);
    let slices = 8.0;
    let fudge = 1.0 / (PI / slices).cos();
    let model_scale = Vector3::new(r_world * fudge, r_world * fudge, r_world * fudge);
    while !render.handle.window_should_close() {
        i_time += render.handle.get_frame_time();
        let mut draw_handle = render.handle.begin_drawing(&render.thread);
        draw_handle.clear_background(Color::BLACK);
        draw_handle.draw_texture_rec(
            &texture,
            flip_framebuffer(i_resolution.x, i_resolution.y),
            ORIGIN,
            Color::WHITE,
        );
        if i_time == 0.0 {
            let view: Matrix = observer.view_matrix();
            let proj: Matrix = observer.projection_matrix(aspect);
            let model_mat = Matrix::translate(model_pos.x, model_pos.y, model_pos.z)
                * Matrix::scale(model_scale.x, model_scale.y, model_scale.z);

            let mvp = proj * view * model_mat;
            fn mul_mat_vec4(m: &Matrix, p: Vector3) -> (f32, f32, f32, f32) {
                let x = p.x * m.m0 + p.y * m.m4 + p.z * m.m8 + m.m12;
                let y = p.x * m.m1 + p.y * m.m5 + p.z * m.m9 + m.m13;
                let z = p.x * m.m2 + p.y * m.m6 + p.z * m.m10 + m.m14;
                let w = p.x * m.m3 + p.y * m.m7 + p.z * m.m11 + m.m15;
                (x, y, z, w)
            }
            let to_px = |p: Vector3| -> Vector2 {
                let (cx, cy, _cz, cw) = mul_mat_vec4(&mvp, p);
                let inv_w = 1.0 / cw;
                Vector2::new(
                    (cx * inv_w * 0.5 + 0.5) * screen_w as f32,
                    (-cy * inv_w * 0.5 + 0.5) * screen_h as f32,
                )
            };
            let top = to_px(Vector3::new(0.0, 1.0, 0.0));
            let bottom = to_px(Vector3::new(0.0, -1.0, 0.0));
            let left = to_px(Vector3::new(-1.0, 0.0, 0.0));
            let right = to_px(Vector3::new(1.0, 0.0, 0.0));
            let cx = (0.33 + 0.5) * screen_h as f32;
            let cy = (0.01 + 0.5) * screen_h as f32;
            let r = 0.40 * screen_h as f32;
            let disk_top = Vector2::new(cx, cy - r);
            let disk_bottom = Vector2::new(cx, cy + r);
            let disk_left = Vector2::new(cx - r, cy);
            let disk_right = Vector2::new(cx + r, cy);
            println!("PX  sphere  |  disk   |  Δ");
            println!(
                "top    ({:.1},{:.1}) ({:.1},{:.1})  Δy={:.1}",
                top.x,
                top.y,
                disk_top.x,
                disk_top.y,
                top.y - disk_top.y
            );
            println!(
                "bottom ({:.1},{:.1}) ({:.1},{:.1})  Δy={:.1}",
                bottom.x,
                bottom.y,
                disk_bottom.x,
                disk_bottom.y,
                bottom.y - disk_bottom.y
            );
            println!(
                "left   ({:.1},{:.1}) ({:.1},{:.1})  Δx={:.1}",
                left.x,
                left.y,
                disk_left.x,
                disk_left.y,
                left.x - disk_left.x
            );
            println!(
                "right  ({:.1},{:.1}) ({:.1},{:.1})  Δx={:.1}",
                right.x,
                right.y,
                disk_right.x,
                disk_right.y,
                right.x - disk_right.x
            );
        }
        for mesh in model.meshes_mut() {
            for vertex in mesh.vertices_mut() {
                vertex.y += (vertex.x * 2.0 + i_time * 2.0).sin() * 0.015;
            }
        }
        let mut rl3d = draw_handle.begin_mode3D(observer);
        rl3d.draw_model_wires_ex(&model, model_pos, Vector3::Y, i_time * 90.0, model_scale, Color::WHITE);
    }
}

#[inline]
fn generate_circle_image(width: i32, height: i32, _i_time: f32) -> Image {
    let img = Image::gen_image_color(width, height, Color::BLANK);
    let total_bytes = (width * height * 4) as usize;
    let pixels: &mut [u8] = unsafe { std::slice::from_raw_parts_mut(img.data as *mut u8, total_bytes) };
    for y in 0..height {
        for x in 0..width {
            let s = (x as f32 + 0.5) / width as f32;
            let t = (y as f32 + 0.5) / height as f32;
            let uv = Vector2::new(s, t);
            let grid = uv_to_grid_space(uv);
            let body_radius = grid.distance(UMBRAL_MASK_CENTER);
            let lum = if body_radius <= UMBRAL_MASK_OUTER_RADIUS {
                255u8
            } else {
                0u8
            };
            let idx = 4 * (y as usize * width as usize + x as usize);
            pixels[idx] = lum; // R
            pixels[idx + 1] = lum; // G
            pixels[idx + 2] = lum; // B
            pixels[idx + 3] = 255u8; // A
        }
    }
    img
}

fn pos_to_uv(p: Vector3) -> (f32, f32) {
    const INV_TWO_PI: f32 = 1.0 / (2.0 * PI);
    const INV_PI: f32 = 1.0 / PI;
    let u = (p.z.atan2(p.x) * INV_TWO_PI) + 0.5;
    let v = p.y.acos() * INV_PI;
    (u, v)
}

fn normalize_mesh(mesh: &mut WeakMesh) -> f32 {
    let mut min = Vector3::new(f32::MAX, f32::MAX, f32::MAX);
    let mut max = Vector3::new(f32::MIN, f32::MIN, f32::MIN);
    for vertex in mesh.vertices() {
        min = min.min(*vertex);
        max = max.max(*vertex);
    }
    let center = (min + max) * 0.5;
    let mut max_extent = 0.0;
    for v in mesh.vertices() {
        let dist = (*v - center).length();
        if dist > max_extent {
            max_extent = dist;
        }
    }
    for vertex in mesh.vertices_mut() {
        *vertex = (*vertex - center) / max_extent;
    }
    max_extent
}

fn warp_verts(model: &mut Model, param_uvs: Vec<Vector2>, stable_positions: Vec<Vector3>, i_time: f32) {
    let mesh = &mut model.meshes_mut()[0];
    for ((warp_position, stable_pos), uv_param) in mesh
        .vertices_mut()
        .iter_mut()
        .zip(stable_positions.iter())
        .zip(param_uvs.iter())
    {
        let n0 = stable_pos.normalize();
        let mut grid = uv_to_grid_space(*uv_param);
        let mut phase = spatial_phase(grid);
        phase += temporal_phase(i_time);
        grid += add_phase(phase);
        let warp_magnitude = LIGHT_WAVE_AMPLITUDE_Y * phase.y.sin();
        *warp_position = *stable_pos + n0 * warp_magnitude;
    }
}
