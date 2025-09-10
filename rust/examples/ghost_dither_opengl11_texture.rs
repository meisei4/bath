use asset_payload::SPHERE_PATH;
use bath::fixed_func::silhouette::{
    build_inverted_hull, collect_deformed_vertex_samples, draw_inverted_hull_guassian_silhouette_stack,
    interpolate_between_deformed_vertices, rotate_inverted_hull, FOVY,
};
use bath::fixed_func::silhouette::{ANGULAR_VELOCITY, MODEL_POS, MODEL_SCALE, SCALE_TWEAK};
use bath::fixed_func::texture::{
    dither, generate_silhouette_texture, rotate_silhouette_texture, rotate_silhouette_texture_dither,
    screen_pass_dither, ScreenPassDither,
};
use bath::fixed_func::topology::observed_line_of_sight;
use bath::render::raylib::RaylibRenderer;
use bath::render::raylib_util::N64_WIDTH;
use bath::render::renderer::Renderer;
use raylib::camera::Camera3D;
use raylib::color::Color;
use raylib::consts::CameraProjection;
use raylib::consts::MaterialMapIndex::MATERIAL_MAP_ALBEDO;
use raylib::consts::PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8;
use raylib::drawing::{RaylibDraw, RaylibDraw3D, RaylibMode3DExt};
use raylib::ffi::{rlDisableDepthMask, rlEnableDepthMask};
use raylib::math::Vector3;
use raylib::models::{RaylibMaterial, RaylibMesh, RaylibModel};
use raylib::texture::Image;

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

    let mut main_model = render.handle.load_model(&render.thread, SPHERE_PATH).unwrap();
    let mut inverted_hull = build_inverted_hull(&mut render, &main_model);
    let mesh_samples = collect_deformed_vertex_samples(main_model.meshes()[0].vertices());
    interpolate_between_deformed_vertices(&mut main_model, i_time, &mesh_samples);
    // let mut silhouette_img = build_stipple_atlas_rgba();
    // let mut silhouette_img = generate_silhouette_texture(128, 128);
    let mut silhouette_img = generate_silhouette_texture(N64_WIDTH, N64_WIDTH);
    dither(&mut silhouette_img);
    let mut blank_image = Image::gen_image_color(
        render.handle.get_screen_width(),
        render.handle.get_screen_height(),
        Color::BLACK,
    );
    blank_image.set_format(PIXELFORMAT_UNCOMPRESSED_R8G8B8A8);
    let blit_texture = unsafe {
        render
            .handle
            .load_texture_from_image(&render.thread, &blank_image)
            .expect("load_texture_from_image failed")
            .make_weak()
    };
    let mut dither_staging = ScreenPassDither {
        blit_texture,
        is_initialized: true,
        staging_rgba_bytes: Vec::new(),
    };
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
        rotate_silhouette_texture(&mut main_model, &main_observer, mesh_rotation);
        rotate_silhouette_texture_dither(
            &mut main_model,
            &main_observer,
            mesh_rotation,
            render.handle.get_screen_width(),
            render.handle.get_screen_height(),
        );
        let mut draw_handle = render.handle.begin_drawing(&render.thread);
        draw_handle.clear_background(Color::BLACK);
        draw_handle.draw_mode3D(main_observer, |mut rl3d| {
            rl3d.draw_model_ex(
                &main_model,
                MODEL_POS,
                Vector3::Y,
                mesh_rotation.to_degrees(),
                MODEL_SCALE * SCALE_TWEAK,
                Color::WHITE,
            );
            rl3d.draw_model_wires_ex(
                &main_model,
                MODEL_POS,
                Vector3::Y,
                mesh_rotation.to_degrees(),
                MODEL_SCALE * SCALE_TWEAK,
                Color::BLACK,
            );
            unsafe {
                rlDisableDepthMask();
            }
            draw_inverted_hull_guassian_silhouette_stack(&mut rl3d, &inverted_hull, mesh_rotation);
            unsafe {
                rlEnableDepthMask();
            }
        });
        screen_pass_dither(&mut draw_handle, &mut dither_staging);
    }
}
