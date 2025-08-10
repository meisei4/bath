use asset_payload::SPHERE_PATH;
use bath::fixed_func::silhouette_inverse_projection_util::{
    generate_mesh_and_texcoord_samples_from_silhouette, generate_silhouette_texture,
    interpolate_mesh_and_texcoord_samples, ANGULAR_VELOCITY, SILHOUETTE_TEXTURE_RES, TIME_BETWEEN_SAMPLES,
};
use bath::geometry::papercraft::unfold_sphere_like;
use bath::geometry::welding::weld_and_index_mesh_for_unfolding;
use bath::render::raylib::RaylibRenderer;
use bath::render::raylib_util::{MODEL_POS, MODEL_SCALE, N64_WIDTH};
use bath::render::renderer::Renderer;
use raylib::camera::Camera3D;
use raylib::color::Color;
use raylib::consts::CameraProjection;
use raylib::consts::MaterialMapIndex::MATERIAL_MAP_ALBEDO;
use raylib::drawing::{RaylibDraw, RaylibDraw3D, RaylibDrawHandle, RaylibMode3DExt};
use raylib::ffi::rlViewport;
use raylib::math::Vector3;
use raylib::models::{Model, RaylibMaterial, RaylibModel};

fn main() {
    let mut i_time = 0.0f32;
    let mut mesh_rotation = 0.0f32;
    let mut render = RaylibRenderer::init(N64_WIDTH * 2, N64_WIDTH);

    let observer = Camera3D {
        position: Vector3::new(0.0, 0.0, 2.0),
        target: Vector3::ZERO,
        up: Vector3::Y,
        fovy: 2.0,
        projection: CameraProjection::CAMERA_ORTHOGRAPHIC,
    };
    let screen_w = render.handle.get_screen_width();
    let screen_h = render.handle.get_screen_height();
    let mut wire_model = render.handle.load_model(&render.thread, SPHERE_PATH).unwrap();
    let mut model = render.handle.load_model(&render.thread, SPHERE_PATH).unwrap();
    let mut papercraft_model = render.handle.load_model(&render.thread, SPHERE_PATH).unwrap();
    weld_and_index_mesh_for_unfolding(&mut papercraft_model.meshes_mut()[0], 1e-6);

    let (mesh_samples, texcoord_samples) =
        generate_mesh_and_texcoord_samples_from_silhouette(screen_w, screen_h, &mut render);
    interpolate_mesh_and_texcoord_samples(&mut wire_model, i_time, &mesh_samples, &texcoord_samples);
    interpolate_mesh_and_texcoord_samples(&mut model, i_time, &mesh_samples, &texcoord_samples);
    interpolate_mesh_and_texcoord_samples(&mut papercraft_model, i_time, &mesh_samples, &texcoord_samples);

    let silhouette_texture =
        generate_silhouette_texture(&mut render, vec![SILHOUETTE_TEXTURE_RES, SILHOUETTE_TEXTURE_RES]);

    model.materials_mut()[0].maps_mut()[MATERIAL_MAP_ALBEDO as usize].texture = *silhouette_texture;
    papercraft_model.materials_mut()[0].maps_mut()[MATERIAL_MAP_ALBEDO as usize].texture = *silhouette_texture;

    let (cameras, labels) = happo_giri_setup();
    while !render.handle.window_should_close() {
        i_time += render.handle.get_frame_time();
        mesh_rotation -= ANGULAR_VELOCITY * render.handle.get_frame_time();
        interpolate_mesh_and_texcoord_samples(&mut model, i_time, &mesh_samples, &texcoord_samples);
        interpolate_mesh_and_texcoord_samples(&mut wire_model, i_time, &mesh_samples, &texcoord_samples);
        let duration = mesh_samples.len() as f32 * TIME_BETWEEN_SAMPLES;
        let time = i_time % duration;
        let frame = time / TIME_BETWEEN_SAMPLES;
        let current_frame = frame.floor() as usize % mesh_samples.len();
        interpolate_mesh_and_texcoord_samples(
            &mut papercraft_model,
            (current_frame as f32 * TIME_BETWEEN_SAMPLES).floor(),
            &mesh_samples,
            &texcoord_samples,
        );
        let unfolded_mesh = unsafe { unfold_sphere_like(&mut papercraft_model.meshes_mut()[0]).make_weak() };
        let mut unfolded_model = render
            .handle
            .load_model_from_mesh(&render.thread, unfolded_mesh.clone())
            .unwrap();
        unfolded_model.materials_mut()[0].maps_mut()[MATERIAL_MAP_ALBEDO as usize].texture = *silhouette_texture;

        let mut draw_handle = render.handle.begin_drawing(&render.thread);
        draw_handle.clear_background(Color::BLACK);
        // {
        //     let mut rl3d = draw_handle.begin_mode3D(observer);
        //     rl3d.draw_model_ex(
        //         &model,
        //         MODEL_POS,
        //         Vector3::Y,
        //         mesh_rotation.to_degrees(),
        //         MODEL_SCALE / 4.0,
        //         Color::WHITE,
        //     );
        // }
        // rl3d.draw_model_ex(&unfolded_model, MODEL_POS, Vector3::Y, 0.0, MODEL_SCALE, Color::WHITE);
        // rl3d.draw_model_wires_ex(
        //     &wire_model,
        //     MODEL_POS,
        //     Vector3::Y,
        //     mesh_rotation.to_degrees(),
        //     MODEL_SCALE / 2.0,
        //     Color::WHITE,
        // );
        happo_giri_draw(
            &mut draw_handle,
            &cameras,
            &labels,
            4,
            2,
            &model,
            &wire_model,
            mesh_rotation,
        );
    }
}

pub fn happo_giri_setup() -> (Vec<Camera3D>, Vec<&'static str>) {
    let diag: f32 = 2.0_f32 / 2.0_f32.sqrt();
    let cameras: Vec<Camera3D> = vec![
        create_camera(0.0, 0.0, 2.0),     // front
        create_camera(0.0, 0.0, -2.0),    // back
        create_camera(-2.0, 0.0, 0.0),    // left
        create_camera(2.0, 0.0, 0.0),     // right
        create_camera(diag, 0.0, diag),   // front-right
        create_camera(-diag, 0.0, diag),  // front-left
        create_camera(diag, 0.0, -diag),  // back-right
        create_camera(-diag, 0.0, -diag), // back-left
    ];
    let labels: Vec<&'static str> = vec![
        "front",
        "back",
        "left",
        "right",
        "front-right",
        "front-left",
        "back-right",
        "back-left",
    ];
    (cameras, labels)
}

pub fn happo_giri_draw(
    draw_handle: &mut RaylibDrawHandle,
    cameras: &[Camera3D],
    labels: &[&'static str],
    grid_columns: i32,
    grid_rows: i32,
    target_model: &Model,
    wire_model: &Model,
    mesh_rotation: f32,
) {
    let screen_w: i32 = draw_handle.get_screen_width();
    let screen_h: i32 = draw_handle.get_screen_height();
    let tile_width_pixels: i32 = screen_w / grid_columns;
    let tile_height_pixels: i32 = screen_h / grid_rows;
    let full_viewport_x = 0;
    let full_viewport_y = 0;
    let full_viewport_w = screen_w;
    let full_viewport_h = screen_h;

    for view_index in 0..8 {
        let column_index: i32 = (view_index as i32) % grid_columns;
        let row_index_top_to_bottom: i32 = (view_index as i32) / grid_columns;
        let row_index_opengl_bottom_origin: i32 = (grid_rows - 1) - row_index_top_to_bottom;
        let viewport_x: i32 = column_index * tile_width_pixels;
        let viewport_y: i32 = row_index_opengl_bottom_origin * tile_height_pixels;
        let viewport_w: i32 = tile_width_pixels;
        let viewport_h: i32 = tile_height_pixels;

        unsafe {
            rlViewport(viewport_x, viewport_y, viewport_w, viewport_h);
        }
        {
            let mut rl3d = draw_handle.begin_mode3D(cameras[view_index]);
            rl3d.draw_model_ex(
                target_model,
                MODEL_POS,
                Vector3::Y,
                mesh_rotation.to_degrees(),
                MODEL_SCALE,
                Color::WHITE,
            );
            rl3d.draw_model_wires_ex(
                wire_model,
                MODEL_POS,
                Vector3::Y,
                mesh_rotation.to_degrees(),
                MODEL_SCALE,
                Color::WHITE,
            );
        }

        unsafe {
            rlViewport(full_viewport_x, full_viewport_y, full_viewport_w, full_viewport_h);
        }

        let label_screen_x = column_index * tile_width_pixels + 6;
        let label_screen_y = row_index_top_to_bottom * tile_height_pixels + 6;
        draw_handle.draw_text(labels[view_index], label_screen_x, label_screen_y, 14, Color::WHITE);
    }
}

pub fn create_camera(x: f32, y: f32, z: f32) -> Camera3D {
    Camera3D {
        position: Vector3::new(x, y, z),
        target: Vector3::ZERO,
        up: Vector3::Y,
        fovy: 2.0,
        projection: CameraProjection::CAMERA_ORTHOGRAPHIC,
    }
}
