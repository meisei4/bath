use asset_payload::SPHERE_PATH;

use bath::fixed_func::silhouette_inverse_projection_util::{
    build_silhouette_radii, generate_inverse_projection_samples_from_silhouette, generate_silhouette_image,
    make_radial_gradient_face, map_mesh_vertices_to_silhouette_texcoords, update_mesh_with_vertex_sample_interpolation,
    update_texcoords, ANGULAR_VELOCITY,
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
use raylib::models::{RaylibMaterial, RaylibMesh, RaylibModel};
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
    let circle_img_2d = generate_silhouette_image(screen_w, screen_h, i_time);
    // let texture_2d = render.handle.load_texture_from_image(&render.thread, &circle_img).unwrap();
    let radii_normals_2d = build_silhouette_radii(&circle_img_2d);
    let mut model = render.handle.load_model(&render.thread, SPHERE_PATH).unwrap();
    let per_frame_vertex_samples = generate_inverse_projection_samples_from_silhouette(screen_w, screen_h, &mut render);
    update_mesh_with_vertex_sample_interpolation(i_time, &per_frame_vertex_samples, &mut model.meshes_mut()[0]);
    let circle_img_3d = make_radial_gradient_face(256, &radii_normals_2d);
    let texture_3d = render
        .handle
        .load_texture_from_image(&render.thread, &circle_img_3d)
        .unwrap();
    texture_3d.set_texture_wrap(&render.thread, TEXTURE_WRAP_CLAMP);
    let mut silhouette_uv = map_mesh_vertices_to_silhouette_texcoords(
        &mut model.meshes_mut()[0].vertices().to_vec(),
        0.0,
        &radii_normals_2d,
    );
    unsafe {
        update_texcoords(&mut model.meshes_mut()[0], &silhouette_uv);
    }
    model.materials_mut()[0].maps_mut()[MATERIAL_MAP_ALBEDO as usize].texture = *texture_3d;
    while !render.handle.window_should_close() {
        i_time += render.handle.get_frame_time();
        mesh_rotation -= ANGULAR_VELOCITY * render.handle.get_frame_time();
        update_mesh_with_vertex_sample_interpolation(i_time, &per_frame_vertex_samples, &mut model.meshes_mut()[0]);
        silhouette_uv = map_mesh_vertices_to_silhouette_texcoords(
            &mut model.meshes_mut()[0].vertices().to_vec(),
            mesh_rotation,
            &radii_normals_2d,
        );
        unsafe {
            update_texcoords(&mut model.meshes_mut()[0], &silhouette_uv);
        }
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
    }
}
