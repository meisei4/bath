use asset_payload::SPHERE_PATH;
use bath::fixed_func::silhouette_inverse_projection_util::{
    generate_mesh_and_texcoord_samples_from_silhouette, generate_silhouette_texture,
    lerp_intermediate_mesh_samples_to_single_mesh, ANGULAR_VELOCITY,
};
use bath::render::raylib::RaylibRenderer;
use bath::render::raylib_util::{MODEL_POS, MODEL_SCALE, N64_WIDTH};
use bath::render::renderer::Renderer;
use raylib::camera::Camera3D;
use raylib::color::Color;
use raylib::consts::CameraProjection;
use raylib::consts::MaterialMapIndex::MATERIAL_MAP_ALBEDO;
use raylib::consts::TextureWrap::TEXTURE_WRAP_CLAMP;
use raylib::drawing::{RaylibDraw, RaylibDraw3D, RaylibMode3DExt};
use raylib::math::Vector3;
use raylib::models::{Model, RaylibMaterial, RaylibModel};
use raylib::texture::RaylibTexture2D;

fn main() {
    let mut i_time = 0.0f32;
    let mut mesh_rotation = 0.0f32;
    let mut render = RaylibRenderer::init(N64_WIDTH, N64_WIDTH);
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
    let (mesh_samples, texcoord_samples) =
        generate_mesh_and_texcoord_samples_from_silhouette(screen_w, screen_h, &mut render);
    interpolate_mesh_and_texcoord_samples(&mut wire_model, &mut model, i_time, &mesh_samples, &texcoord_samples);

    let silhouette_texture = generate_silhouette_texture(&mut render, screen_w, screen_h, vec![256, 256], i_time);
    silhouette_texture.set_texture_wrap(&render.thread, TEXTURE_WRAP_CLAMP);
    model.materials_mut()[0].maps_mut()[MATERIAL_MAP_ALBEDO as usize].texture = *silhouette_texture;
    while !render.handle.window_should_close() {
        i_time += render.handle.get_frame_time();
        mesh_rotation -= ANGULAR_VELOCITY * render.handle.get_frame_time();
        interpolate_mesh_and_texcoord_samples(&mut wire_model, &mut model, i_time, &mesh_samples, &texcoord_samples);
        let mut draw_handle = render.handle.begin_drawing(&render.thread);
        draw_handle.clear_background(Color::BLACK);
        let mut rl3d = draw_handle.begin_mode3D(observer);
        rl3d.draw_model_ex(
            &model,
            MODEL_POS,
            Vector3::Y,
            mesh_rotation.to_degrees(),
            MODEL_SCALE,
            Color::WHITE,
        );
        // rl3d.draw_model_wires_ex(
        //     &wire_model,
        //     MODEL_POS,
        //     Vector3::Y,
        //     mesh_rotation.to_degrees(),
        //     MODEL_SCALE,
        //     Color::WHITE,
        // );
    }
}

pub fn interpolate_mesh_and_texcoord_samples(
    wire_model: &mut Model,
    model: &mut Model,
    i_time: f32,
    mesh_samples: &Vec<Vec<Vector3>>,
    texcoord_samples: &Vec<Vec<f32>>,
) {
    let wire_mesh = &mut wire_model.meshes_mut()[0];
    let main_mesh = &mut model.meshes_mut()[0];
    lerp_intermediate_mesh_samples_to_single_mesh(i_time, mesh_samples, texcoord_samples, wire_mesh);
    lerp_intermediate_mesh_samples_to_single_mesh(i_time, mesh_samples, texcoord_samples, main_mesh);
}
