use asset_payload::SPHERE_PATH;

use bath::fixed_func::silhouette_inverse_projection_util::{generate_inverse_projection_samples_from_silhouette, generate_inverse_projection_texcoords, generate_silhouette_texture, update_mesh_with_vertex_sample_lerp, update_texcoords_with_silhouette_samples_lerp, ANGULAR_VELOCITY};
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
use raylib::models::{RaylibMaterial, RaylibModel};
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

    let mut wire = render.handle.load_model(&render.thread, SPHERE_PATH).unwrap();
    let mut model = render.handle.load_model(&render.thread, SPHERE_PATH).unwrap();

    let silhouette_texture = generate_silhouette_texture(&mut render, screen_w, screen_h, vec![128, 128], i_time);
    silhouette_texture.set_texture_wrap(&render.thread, TEXTURE_WRAP_CLAMP);
    model.materials_mut()[0].maps_mut()[MATERIAL_MAP_ALBEDO as usize].texture = *silhouette_texture;

    let mut per_frame_vertex_samples = generate_inverse_projection_samples_from_silhouette(screen_w, screen_h, &mut render);
    // let per_frame_silhouette_samples =
    //     generate_silhouette_texcoord_samples(screen_w, screen_h, &per_frame_vertex_samples, mesh_rotation, i_time);
    let per_frame_silhouette_samples = generate_inverse_projection_texcoords(screen_w, screen_h, &mut per_frame_vertex_samples);
    update_mesh_with_vertex_sample_lerp(i_time, &per_frame_vertex_samples, &mut wire.meshes_mut()[0]);
    update_mesh_with_vertex_sample_lerp(i_time, &per_frame_vertex_samples, &mut model.meshes_mut()[0]);
    update_texcoords_with_silhouette_samples_lerp(i_time, &per_frame_silhouette_samples, &mut model.meshes_mut()[0]);
    while !render.handle.window_should_close() {
        i_time += render.handle.get_frame_time();
        mesh_rotation -= ANGULAR_VELOCITY * render.handle.get_frame_time();
        update_mesh_with_vertex_sample_lerp(i_time, &per_frame_vertex_samples, &mut wire.meshes_mut()[0]);
        update_mesh_with_vertex_sample_lerp(i_time, &per_frame_vertex_samples, &mut model.meshes_mut()[0]);
        update_texcoords_with_silhouette_samples_lerp(
            i_time,
            &per_frame_silhouette_samples,
            &mut model.meshes_mut()[0],
        );

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
        //     &wire,
        //     MODEL_POS,
        //     Vector3::Y,
        //     mesh_rotation.to_degrees(),
        //     MODEL_SCALE,
        //     Color::WHITE,
        // );
    }
}
