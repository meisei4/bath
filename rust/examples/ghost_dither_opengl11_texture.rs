use asset_payload::SPHERE_PATH;
use bath::fixed_func::constants::{
    ANGULAR_VELOCITY, MODEL_POS, MODEL_SCALE, SILHOUETTE_TEXTURE_RES, TIME_BETWEEN_SAMPLES,
};
use bath::fixed_func::happo_giri_observer::happo_giri_setup;
use bath::fixed_func::papercraft::unfold_sphere_like;
use bath::fixed_func::silhouette_inverse_projection_util::{debug_papercraft, generate_mesh_and_texcoord_samples_from_silhouette, generate_silhouette_texture, interpolate_mesh_and_texcoord_samples, rotate_vertices};
use bath::fixed_func::welding::{weld_and_index_mesh_for_unfolding, weld_for_smoothing_topo};
use bath::render::raylib::RaylibRenderer;
use bath::render::raylib_util::N64_WIDTH;
use bath::render::renderer::Renderer;
use raylib::camera::Camera3D;
use raylib::color::Color;
use raylib::consts::CameraProjection;
use raylib::consts::MaterialMapIndex::MATERIAL_MAP_ALBEDO;
use raylib::drawing::{RaylibDraw, RaylibDraw3D, RaylibDrawHandle, RaylibMode3DExt};
use raylib::math::Vector3;
use raylib::models::{RaylibMaterial, RaylibMesh, RaylibModel, WeakMesh};
use std::slice::from_raw_parts;

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
    weld_for_smoothing_topo(&mut wire_model.meshes_mut()[0]);

    let mut main_model = render.handle.load_model(&render.thread, SPHERE_PATH).unwrap();
    weld_for_smoothing_topo(&mut main_model.meshes_mut()[0]);
    let mut papercraft_model = render.handle.load_model(&render.thread, SPHERE_PATH).unwrap();
    weld_and_index_mesh_for_unfolding(&mut papercraft_model.meshes_mut()[0]);

    let (mesh_samples, texcoord_samples) = generate_mesh_and_texcoord_samples_from_silhouette(&mut render);
    interpolate_mesh_and_texcoord_samples(&mut wire_model, i_time, &mesh_samples, &texcoord_samples);
    interpolate_mesh_and_texcoord_samples(&mut main_model, i_time, &mesh_samples, &texcoord_samples);
    interpolate_mesh_and_texcoord_samples(&mut papercraft_model, i_time, &mesh_samples, &texcoord_samples);

    let silhouette_texture =
        generate_silhouette_texture(&mut render, vec![SILHOUETTE_TEXTURE_RES, SILHOUETTE_TEXTURE_RES]);

    // main_model.materials_mut()[0].maps_mut()[MATERIAL_MAP_ALBEDO as usize].texture = *silhouette_texture;
    // papercraft_model.materials_mut()[0].maps_mut()[MATERIAL_MAP_ALBEDO as usize].texture = *silhouette_texture;

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
            //     rl3d.draw_model_wires_ex(
            //         &wire_model,
            //         MODEL_POS,
            //         Vector3::Y,
            //         mesh_rotation.to_degrees(),
            //         MODEL_SCALE / 2.0,
            //         Color::WHITE,
            //     );

            // rl3d.draw_model_ex(&unfolded_model, MODEL_POS, Vector3::Y, 0.0, MODEL_SCALE, Color::WHITE);
        }
        debug_papercraft(main_observer, &mut draw_handle, &mut wire_model.meshes_mut()[0], mesh_rotation);
        // happo_giri_draw(
        //     &mut draw_handle,
        //     &observers,
        //     &labels,
        //     4,
        //     2,
        //     &main_model,
        //     &wire_model,
        //     mesh_rotation,
        // );
    }
}