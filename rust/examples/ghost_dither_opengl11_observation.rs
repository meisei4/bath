use asset_payload::SPHERE_PATH;
use bath::fixed_func::happo_giri_observer::{happo_giri_draw, happo_giri_setup};
use bath::fixed_func::silhouette::{build_inverted_hull, FOVY};
use bath::fixed_func::silhouette::{
    collect_deformed_vertex_samples, interpolate_between_deformed_vertices, rotate_inverted_hull, ANGULAR_VELOCITY,
};
use bath::fixed_func::texture::{dither, generate_silhouette_texture};
use bath::fixed_func::topology::observed_line_of_sight;
use bath::render::raylib::RaylibRenderer;
use bath::render::raylib_util::{N64_HEIGHT, N64_WIDTH};
use bath::render::renderer::Renderer;
use raylib::camera::Camera3D;
use raylib::consts::CameraProjection;
use raylib::consts::MaterialMapIndex::MATERIAL_MAP_ALBEDO;
use raylib::ffi::{rlSetLineWidth, rlSetPointSize};
use raylib::math::Vector3;
use raylib::models::{RaylibMaterial, RaylibMesh, RaylibModel};

fn main() {
    let mut i_time = 0.0f32;
    let mut mesh_rotation = 0.0f32;
    let mut render = RaylibRenderer::init(N64_WIDTH * 2, N64_HEIGHT * 2);
    let main_observer = Camera3D {
        position: Vector3::new(0.0, 0.0, 2.0),
        target: Vector3::ZERO,
        up: Vector3::Y,
        fovy: FOVY,
        projection: CameraProjection::CAMERA_ORTHOGRAPHIC,
    };
    let (observers, labels) = happo_giri_setup();
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
    main_model.materials_mut()[0].maps_mut()[MATERIAL_MAP_ALBEDO as usize].texture = *silhouette_texture;
    let observed_los = observed_line_of_sight(&main_observer);
    while !render.handle.window_should_close() {
        i_time += render.handle.get_frame_time();
        mesh_rotation -= ANGULAR_VELOCITY * render.handle.get_frame_time();
        interpolate_between_deformed_vertices(&mut main_model, i_time, &mesh_samples);
        rotate_inverted_hull(&main_model.meshes()[0], &mut inverted_hull, observed_los, mesh_rotation);
        unsafe { rlSetLineWidth(3.0) };
        unsafe { rlSetPointSize(6.0) };
        happo_giri_draw(
            &mut render,
            &observers,
            &labels,
            4,
            2,
            &main_model,
            Some(&inverted_hull),
            mesh_rotation,
        );
    }
}
