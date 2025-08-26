use asset_payload::SPHERE_PATH;
use bath::fixed_func::happo_giri_observer::happo_giri_setup;
use bath::fixed_func::papercraft::unfold;
use bath::fixed_func::silhouette::{
    generate_mesh_and_texcoord_samples_from_silhouette, generate_silhouette_texture_fast,
};
use bath::fixed_func::silhouette_constants::{
    ANGULAR_VELOCITY, SILHOUETTE_RADII_RESOLUTION, TEXTURE_MAPPING_BOUNDARY_FADE, TIME_BETWEEN_SAMPLES,
};
use bath::fixed_func::silhouette_interpolation::interpolate_mesh_and_texcoord_samples;
use bath::fixed_func::silhouette_util::debug_uv_seams;
use bath::render::raylib::RaylibRenderer;
use bath::render::raylib_util::N64_WIDTH;
use bath::render::renderer::Renderer;
use raylib::camera::Camera3D;
use raylib::color::Color;
use raylib::consts::CameraProjection;
use raylib::consts::MaterialMapIndex::MATERIAL_MAP_ALBEDO;
use raylib::drawing::{RaylibDraw, RaylibMode3DExt};
use raylib::math::Vector3;
use raylib::models::{RaylibMaterial, RaylibModel};

fn main() {
    let mut i_time = 0.0f32;
    let mut mesh_rotation = 0.0f32;
    let mut render = RaylibRenderer::init(N64_WIDTH, N64_WIDTH);

    let main_observer = Camera3D {
        position: Vector3::new(0.0, 0.0, 2.0),
        target: Vector3::ZERO,
        up: Vector3::Y,
        fovy: 2.0,
        projection: CameraProjection::CAMERA_ORTHOGRAPHIC,
    };
    let (observers, labels) = happo_giri_setup();
    let mut wire_model = render.handle.load_model(&render.thread, SPHERE_PATH).unwrap();

    let mut main_model = render.handle.load_model(&render.thread, SPHERE_PATH).unwrap();
    let mut papercraft_model = render.handle.load_model(&render.thread, SPHERE_PATH).unwrap();

    let (mesh_samples, texcoord_samples) = generate_mesh_and_texcoord_samples_from_silhouette(&mut render);
    interpolate_mesh_and_texcoord_samples(&mut wire_model, i_time, &mesh_samples, &texcoord_samples);
    interpolate_mesh_and_texcoord_samples(&mut main_model, i_time, &mesh_samples, &texcoord_samples);
    interpolate_mesh_and_texcoord_samples(&mut papercraft_model, i_time, &mesh_samples, &texcoord_samples);
    let silhouette_texture = generate_silhouette_texture_fast(
        &mut render,
        SILHOUETTE_RADII_RESOLUTION as i32,
        64,
        TEXTURE_MAPPING_BOUNDARY_FADE,
    );
    main_model.materials_mut()[0].maps_mut()[MATERIAL_MAP_ALBEDO as usize].texture = *silhouette_texture;
    papercraft_model.materials_mut()[0].maps_mut()[MATERIAL_MAP_ALBEDO as usize].texture = *silhouette_texture;

    while !render.handle.window_should_close() {
        i_time += render.handle.get_frame_time();
        mesh_rotation -= ANGULAR_VELOCITY * render.handle.get_frame_time();
        interpolate_mesh_and_texcoord_samples(&mut main_model, i_time, &mesh_samples, &texcoord_samples);
        interpolate_mesh_and_texcoord_samples(&mut wire_model, i_time, &mesh_samples, &texcoord_samples);
        let duration = mesh_samples.len() as f32 * TIME_BETWEEN_SAMPLES;
        let time = i_time % duration;
        let frame = time / TIME_BETWEEN_SAMPLES;
        let current_frame = frame.floor() as usize % mesh_samples.len();
        interpolate_mesh_and_texcoord_samples(
            &mut papercraft_model,
            i_time,
            // (current_frame as f32 * TIME_BETWEEN_SAMPLES).floor(),
            &mesh_samples,
            &texcoord_samples,
        );
        // let unfolded_mesh = unsafe { fold(&mut wire_model.meshes_mut()[0], i_time, true).make_weak() };
        let unfolded_mesh = unsafe { unfold(&mut papercraft_model.meshes_mut()[0]).make_weak() };
        let mut unfolded_model = render
            .handle
            .load_model_from_mesh(&render.thread, unfolded_mesh.clone())
            .unwrap();
        unfolded_model.materials_mut()[0].maps_mut()[MATERIAL_MAP_ALBEDO as usize].texture = *silhouette_texture;

        let mut draw_handle = render.handle.begin_drawing(&render.thread);
        draw_handle.clear_background(Color::BLACK);
        {
            let mut rl3d = draw_handle.begin_mode3D(main_observer);
            // rl3d.draw_model_ex(
            //     &main_model,
            //     MODEL_POS,
            //     Vector3::Y,
            //     mesh_rotation.to_degrees(),
            //     MODEL_SCALE,
            //     Color::WHITE,
            // );
            // rl3d.draw_model_wires_ex(
            //     &wire_model,
            //     MODEL_POS,
            //     Vector3::Y,
            //     mesh_rotation.to_degrees(),
            //     MODEL_SCALE,
            //     Color::BLACK,
            // );
            // rl3d.draw_model_ex(&unfolded_model, MODEL_POS, Vector3::Y, 0.0, MODEL_SCALE, Color::WHITE);
            // rl3d.draw_model_wires_ex(
            //     &unfolded_model,
            //     MODEL_POS,
            //     Vector3::Y,
            //     0.0,
            //     MODEL_SCALE,
            //     Color::BLACK,
            // );
        }
        // debug_uv_seams(
        //     main_observer,
        //     &mut draw_handle,
        //     &wire_model.meshes()[0],
        //     mesh_rotation,
        //     0.90,
        //     24,
        // );
        debug_uv_seams(
            main_observer,
            &mut draw_handle,
            &unfolded_model.meshes()[0],
            0.0,
            0.96,
            24,
        );
    }
}
