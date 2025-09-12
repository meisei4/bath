use asset_payload::SPHERE_PATH;
use bath::fixed_func::silhouette::{
    build_inverted_hull, collect_deformed_vertex_samples, draw_inverted_hull_guassian_silhouette_stack,
    interpolate_between_deformed_vertices, rotate_inverted_hull, FOVY_ORTHOGRAPHIC,
};
use bath::fixed_func::silhouette::{ANGULAR_VELOCITY, MODEL_POS, MODEL_SCALE};
use bath::fixed_func::texture::{dither, generate_silhouette_texture};
use bath::fixed_func::topology::{debug_draw_triangles, observed_line_of_sight, Topology};
use bath::render::raylib::RaylibRenderer;
use bath::render::raylib_util::N64_WIDTH;
use bath::render::renderer::Renderer;
use raylib::camera::Camera3D;
use raylib::color::Color;
use raylib::consts::CameraProjection;
use raylib::consts::MaterialMapIndex::MATERIAL_MAP_ALBEDO;
use raylib::drawing::{RaylibDraw, RaylibDraw3D, RaylibMode3DExt};
use raylib::ffi::{rlSetLineWidth, rlSetPointSize};
use raylib::math::Vector3;
use raylib::models::{RaylibMaterial, RaylibMesh, RaylibModel};

fn main() {
    let mut i_time = 0.0f32;
    let mut mesh_rotation = 0.0f32;
    let mut render = RaylibRenderer::init(N64_WIDTH, N64_WIDTH);
    let main_observer = Camera3D {
        position: Vector3::new(0.0, 0.0, 2.0),
        target: Vector3::ZERO,
        up: Vector3::Y,
        fovy: FOVY_ORTHOGRAPHIC,
        projection: CameraProjection::CAMERA_ORTHOGRAPHIC,
    };
    let mut main_model = render.handle.load_model(&render.thread, SPHERE_PATH).unwrap();
    let mut inverted_hull = build_inverted_hull(&mut render, &main_model);
    let mesh_samples = collect_deformed_vertex_samples(main_model.meshes()[0].vertices());
    interpolate_between_deformed_vertices(&mut main_model, i_time, &mesh_samples);
    let mut silhouette_img = generate_silhouette_texture(N64_WIDTH, N64_WIDTH);
    dither(&mut silhouette_img);
    let silhouette_texture = render
        .handle
        .load_texture_from_image(&render.thread, &silhouette_img)
        .unwrap();
    //TODO: figure out how to do this better lmao
    // main_model.materials_mut()[0].set_material_texture(MATERIAL_MAP_ALBEDO, silhouette_texture);
    main_model.materials_mut()[0].maps_mut()[MATERIAL_MAP_ALBEDO as usize].texture = *silhouette_texture;
    let observed_los = observed_line_of_sight(&main_observer);
    while !render.handle.window_should_close() {
        i_time += render.handle.get_frame_time();
        mesh_rotation -= ANGULAR_VELOCITY * render.handle.get_frame_time();
        interpolate_between_deformed_vertices(&mut main_model, i_time, &mesh_samples);
        rotate_inverted_hull(&main_model.meshes()[0], &mut inverted_hull, observed_los, mesh_rotation);
        let mut draw_handle = render.handle.begin_drawing(&render.thread);
        draw_handle.clear_background(Color::BLACK);
        draw_handle.draw_mode3D(main_observer, |mut rl3d| {
            rl3d.draw_model_ex(
                &main_model,
                MODEL_POS,
                Vector3::Y,
                mesh_rotation.to_degrees(),
                MODEL_SCALE,
                Color::BLUE,
            );
            unsafe { rlSetLineWidth(5.0) };
            rl3d.draw_model_wires_ex(
                &main_model,
                MODEL_POS,
                Vector3::Y,
                mesh_rotation.to_degrees(),
                MODEL_SCALE,
                Color::RED,
            );
            unsafe { rlSetPointSize(20.0) };
            rl3d.draw_model_points_ex(
                &main_model,
                MODEL_POS,
                Vector3::Y,
                mesh_rotation.to_degrees(),
                MODEL_SCALE,
                Color::GREEN,
            );
            draw_inverted_hull_guassian_silhouette_stack(&mut rl3d, &inverted_hull, mesh_rotation);
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
            debug_draw_triangles(
                main_observer,
                &mut draw_handle,
                &topology,
                mesh_rotation,
                &triangle_set,
                Some(Color::new(255, 32, 32, 90)),
                true,
                12,
            );
        }
    }
}
