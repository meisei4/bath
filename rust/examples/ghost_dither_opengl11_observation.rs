use asset_payload::SPHERE_PATH;
use bath::fixed_func::happo_giri_observer::{happo_giri_draw, happo_giri_setup};
use bath::fixed_func::silhouette::{
    generate_mesh_and_texcoord_samples_from_silhouette, generate_silhouette_texture_fast,
};
use bath::fixed_func::silhouette_constants::{
    ANGULAR_VELOCITY, SILHOUETTE_RADII_RESOLUTION, TEXTURE_MAPPING_BOUNDARY_FADE,
};
use bath::fixed_func::silhouette_interpolation::interpolate_mesh_samples_and_texcoord_samples;
use bath::render::raylib::RaylibRenderer;
use bath::render::raylib_util::N64_WIDTH;
use bath::render::renderer::Renderer;
use raylib::camera::Camera3D;
use raylib::color::Color;
use raylib::consts::CameraProjection;
use raylib::consts::MaterialMapIndex::MATERIAL_MAP_ALBEDO;
use raylib::drawing::RaylibDraw;
use raylib::math::Vector3;
use raylib::models::{RaylibMaterial, RaylibModel};

fn main() {
    let mut i_time = 0.0f32;
    let mut mesh_rotation = 0.0f32;
    let mut render = RaylibRenderer::init(N64_WIDTH * 2, N64_WIDTH);

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
    let (mesh_samples, texcoord_samples) = generate_mesh_and_texcoord_samples_from_silhouette(&mut render);
    interpolate_mesh_samples_and_texcoord_samples(&mut wire_model, i_time, &mesh_samples, &texcoord_samples);
    interpolate_mesh_samples_and_texcoord_samples(&mut main_model, i_time, &mesh_samples, &texcoord_samples);
    let silhouette_texture = generate_silhouette_texture_fast(
        &mut render,
        SILHOUETTE_RADII_RESOLUTION as i32,
        64,
        TEXTURE_MAPPING_BOUNDARY_FADE,
    );
    main_model.materials_mut()[0].maps_mut()[MATERIAL_MAP_ALBEDO as usize].texture = *silhouette_texture;

    while !render.handle.window_should_close() {
        i_time += render.handle.get_frame_time();
        mesh_rotation -= ANGULAR_VELOCITY * render.handle.get_frame_time();
        interpolate_mesh_samples_and_texcoord_samples(&mut main_model, i_time, &mesh_samples, &texcoord_samples);
        interpolate_mesh_samples_and_texcoord_samples(&mut wire_model, i_time, &mesh_samples, &texcoord_samples);
        let mut draw_handle = render.handle.begin_drawing(&render.thread);
        draw_handle.clear_background(Color::BLACK);
        happo_giri_draw(
            &mut draw_handle,
            &observers,
            &labels,
            4,
            2,
            &main_model,
            &wire_model,
            mesh_rotation,
        );
    }
}
