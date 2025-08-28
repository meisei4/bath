use asset_payload::SPHERE_PATH;
use bath::fixed_func::silhouette::{
    collect_deformed_mesh_samples, draw_inverted_hull_guassian_multipass, interpolate_between_deformed_meshes,
    update_inverted_hull, SCALE_TWEAK,
};
use bath::fixed_func::silhouette::{ANGULAR_VELOCITY, MODEL_POS, MODEL_SCALE};
use bath::fixed_func::topology::{
    collect_back_faces, collect_face_texcoords, collect_front_faces, collect_neighbors, collect_welded_faces,
    ensure_drawable, observed_line_of_sight, reverse_vertex_winding, topology_init,
};
use bath::render::raylib::RaylibRenderer;
use bath::render::raylib_util::N64_WIDTH;
use bath::render::renderer::Renderer;
use raylib::camera::Camera3D;
use raylib::color::Color;
use raylib::consts::CameraProjection;
use raylib::drawing::{RaylibDraw, RaylibDraw3D, RaylibMode3DExt};
use raylib::math::Vector3;
use raylib::models::RaylibModel;

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

    let mut wire_model = render.handle.load_model(&render.thread, SPHERE_PATH).unwrap();
    let mut main_model = render.handle.load_model(&render.thread, SPHERE_PATH).unwrap();
    let mut inverted_hull_model = render.handle.load_model(&render.thread, SPHERE_PATH).unwrap();

    ensure_drawable(&mut wire_model.meshes_mut()[0]);
    ensure_drawable(&mut main_model.meshes_mut()[0]);
    ensure_drawable(&mut inverted_hull_model.meshes_mut()[0]);
    reverse_vertex_winding(&mut inverted_hull_model.meshes_mut()[0]);

    let mesh_samples = collect_deformed_mesh_samples(&mut render);
    // let (mesh_samples, texcoord_samples) = generate_mesh_and_texcoord_samples_from_silhouette(&mut render);
    interpolate_between_deformed_meshes(&mut wire_model, i_time, &mesh_samples);
    interpolate_between_deformed_meshes(&mut main_model, i_time, &mesh_samples);
    // interpolate_mesh_samples_and_texcoord_samples(&mut wire_model, i_time, &mesh_samples, &texcoord_samples);
    // interpolate_mesh_samples_and_texcoord_samples(&mut main_model, i_time, &mesh_samples, &texcoord_samples);
    // let silhouette_texture = generate_silhouette_texture(
    //     &mut render,
    //     SILHOUETTE_RADII_RESOLUTION as i32,
    //     64,
    //     TEXTURE_MAPPING_BOUNDARY_FADE,
    // );
    while !render.handle.window_should_close() {
        i_time += render.handle.get_frame_time();
        mesh_rotation -= ANGULAR_VELOCITY * render.handle.get_frame_time();
        interpolate_between_deformed_meshes(&mut wire_model, i_time, &mesh_samples);
        interpolate_between_deformed_meshes(&mut main_model, i_time, &mesh_samples);

        let observed_line_of_sight = observed_line_of_sight(&main_observer);
        update_inverted_hull(
            &main_model,
            &mut inverted_hull_model,
            observed_line_of_sight,
            mesh_rotation,
        );

        // interpolate_mesh_samples_and_texcoord_samples(&mut wire_model, i_time, &mesh_samples, &texcoord_samples);
        // interpolate_mesh_samples_and_texcoord_samples(&mut main_model, i_time, &mesh_samples, &texcoord_samples);
        let mut topology = topology_init(&main_model.meshes_mut()[0]);
        collect_welded_faces(&mut topology);
        collect_neighbors(&mut topology);
        collect_front_faces(
            &mut topology,
            &main_model.meshes_mut()[0],
            mesh_rotation,
            &main_observer,
        );
        collect_face_texcoords(&mut topology, &main_model.meshes_mut()[0]);
        collect_back_faces(&mut topology);
        // collect_silhouette_faces(&mut topology);
        // let silhouette_texture = generate_view_silhouette_texture_uvspace(
        //     &mut render,
        //     &topology,
        //     SILHOUETTE_TEXTURE_RES,
        //     SILHOUETTE_TEXTURE_RES,
        //     TEXTURE_MAPPING_BOUNDARY_FADE,
        // );
        // silhouette_texture.set_texture_wrap(&render.thread, TextureWrap::TEXTURE_WRAP_CLAMP);
        // main_model.materials_mut()[0].maps_mut()[MATERIAL_MAP_ALBEDO as usize].texture = *silhouette_texture;
        let mut draw_handle = render.handle.begin_drawing(&render.thread);
        draw_handle.clear_background(Color::BLACK);
        {
            let mut rl3d = draw_handle.begin_mode3D(main_observer);
            draw_inverted_hull_guassian_multipass(&mut rl3d, &inverted_hull_model, mesh_rotation);
            rl3d.draw_model_ex(
                &main_model,
                MODEL_POS,
                Vector3::Y,
                mesh_rotation.to_degrees(),
                MODEL_SCALE * SCALE_TWEAK,
                Color::WHITE,
            );
            // rl3d.draw_model_wires_ex(
            //     &wire_model,
            //     MODEL_POS,
            //     Vector3::Y,
            //     mesh_rotation.to_degrees(),
            //     MODEL_SCALE,
            //     Color::BLACK,
            // );
        }
        // collect_welded_faces(&mut topology);
        // collect_neighbors(&mut topology);
        // collect_front_faces(
        //     &mut topology,
        //     &wire_model.meshes_mut()[0],
        //     mesh_rotation,
        //     &main_observer,
        // );
        // collect_back_faces(&mut topology);
        // collect_silhouette_faces(&mut topology);
        // if let Some(silhouette_faces) = &topology.silhouette_faces {
        //     debug_draw_faces(
        //         main_observer,
        //         &mut draw_handle,
        //         &wire_model.meshes_mut()[0],
        //         mesh_rotation,
        //         silhouette_faces,
        //         Some(Color::new(255, 32, 32, 90)),
        //         true,
        //     );
        // }
    }
}
