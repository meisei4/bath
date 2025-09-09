use asset_payload::SPHERE_PATH;
use bath::fixed_func::silhouette::{
    calibrate_motion_lock, deform_mesh_from_field_phase_derived, generate_silhouette_radial_field,
    rotate_inverted_hull, FOVY,
};
use bath::fixed_func::silhouette::{MODEL_POS, MODEL_SCALE, SCALE_TWEAK};
use bath::fixed_func::texture::{dither, generate_silhouette_texture};
use bath::fixed_func::topology::{observed_line_of_sight, Topology};
use bath::render::raylib::RaylibRenderer;
use bath::render::raylib_util::N64_WIDTH;
use bath::render::renderer::Renderer;
use raylib::camera::Camera3D;
use raylib::color::Color;
use raylib::consts::CameraProjection;
use raylib::drawing::{RaylibDraw, RaylibDraw3D, RaylibMode3DExt};
use raylib::math::Vector3;
use raylib::models::{RaylibMesh, RaylibModel};
use std::ptr::null_mut;

fn main() {
    let mut i_time = 0.0f32;
    let mut mesh_rotation = 0.0f32;
    let mut render = RaylibRenderer::init(N64_WIDTH, N64_WIDTH);
    let main_observer = Camera3D {
        position: Vector3::new(0.0, 0.0, 2.0),
        target: Vector3::ZERO,
        up: Vector3::Y,
        fovy: FOVY,
        projection: CameraProjection::CAMERA_ORTHOGRAPHIC,
    };

    let mut wire_model = render.handle.load_model(&render.thread, SPHERE_PATH).unwrap();
    wire_model.meshes_mut()[0].colors = null_mut();
    wire_model.meshes_mut()[0].ensure_colors().unwrap();

    let mut main_model = render.handle.load_model(&render.thread, SPHERE_PATH).unwrap();
    let base_vertices = main_model.meshes()[0].vertices().to_vec();
    let motion_lock = calibrate_motion_lock(1.0 / 120.0);

    let mut inverted_hull = render.handle.load_model(&render.thread, SPHERE_PATH).unwrap();

    // let mesh_samples = collect_deformed_mesh_samples(&mut render);
    // interpolate_between_deformed_meshes(&mut wire_model, i_time, &mesh_samples);
    // interpolate_between_deformed_meshes(&mut main_model, i_time, &mesh_samples);
    let mut silhouette_img = generate_silhouette_texture(N64_WIDTH, N64_WIDTH);
    dither(&mut silhouette_img);
    let silhouette_texture = render
        .handle
        .load_texture_from_image(&render.thread, &silhouette_img)
        .unwrap();
    //TODO: figure out how to do this better lmao
    // main_model.materials_mut()[0].set_material_texture(MATERIAL_MAP_ALBEDO, silhouette_texture);
    // main_model.materials_mut()[0].maps_mut()[MATERIAL_MAP_ALBEDO as usize].texture = *silhouette_texture;
    let observed_los = observed_line_of_sight(&main_observer);
    while !render.handle.window_should_close() {
        i_time += render.handle.get_frame_time();
        // mesh_rotation -= ANGULAR_VELOCITY * render.handle.get_frame_time();
        mesh_rotation -= motion_lock.angular_velocity * render.handle.get_frame_time();
        let radial_field = generate_silhouette_radial_field(i_time);
        deform_mesh_from_field_phase_derived(&mut main_model, &base_vertices, mesh_rotation, &radial_field);
        // interpolate_between_deformed_meshes(&mut wire_model, i_time, &mesh_samples);
        // interpolate_between_deformed_meshes(&mut main_model, i_time, &mesh_samples);
        rotate_inverted_hull(&main_model, &mut inverted_hull, observed_los, mesh_rotation);
        let mut draw_handle = render.handle.begin_drawing(&render.thread);
        draw_handle.clear_background(Color::BLACK);
        draw_handle.draw_mode3D(main_observer, |mut rl3d| {
            // rl3d.draw_model_ex(
            //     &main_model,
            //     MODEL_POS,
            //     Vector3::Y,
            //     mesh_rotation.to_degrees(),
            //     MODEL_SCALE * SCALE_TWEAK,
            //     Color::WHITE,
            // );
            rl3d.draw_model_wires_ex(
                &main_model,
                MODEL_POS,
                Vector3::Y,
                mesh_rotation.to_degrees(),
                MODEL_SCALE * SCALE_TWEAK,
                Color::BLACK,
            );
            // draw_inverted_hull_guassian_silhouette_stack(&mut rl3d, &inverted_hull, mesh_rotation);
        });
        let topology = Topology::build_topology(&main_model.meshes()[0])
            .welded_vertices()
            .triangles()
            .welded_vertices_per_triangle()
            .neighbors_per_triangle()
            .vertices_per_triangle()
            .front_triangles(mesh_rotation, &main_observer)
            .back_triangles()
            .silhouette_triangles()
            .build();
        if let Some(triangle_set) = topology.silhouette_triangles_snapshot.as_ref() {
            // if let Some(triangle_set) = topology.front_triangles.as_ref() {
            // if let Some(triangle_set) = topology.back_triangles.as_ref() {
            // debug_draw_triangles(
            //     main_observer,
            //     &mut draw_handle,
            //     &topology,
            //     mesh_rotation,
            //     &triangle_set,
            //     Some(Color::new(255, 32, 32, 90)),
            //     true,
            // );
        }
    }
}
