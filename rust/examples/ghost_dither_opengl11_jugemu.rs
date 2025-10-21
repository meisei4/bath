use bath::fixed_func::jugemu::{
    draw_frustum, draw_near_plane_intersectional_disk_mesh, draw_observed_axes, map_frustum_to_ndc_cube,
    move_jugemu_orbital, observed_orthonormal_basis_vectors,
};
use bath::fixed_func::silhouette::{ANGULAR_VELOCITY, FOVY_PERSPECTIVE, MODEL_POS, MODEL_SCALE};
use bath::fixed_func::topology::Topology;
use bath::render::raylib::RaylibRenderer;
use bath::render::raylib_util::{N64_HEIGHT, N64_WIDTH};
use bath::render::renderer::Renderer;
use raylib::camera::Camera3D;
use raylib::color::Color;
use raylib::consts::CameraProjection;
use raylib::consts::MaterialMapIndex::MATERIAL_MAP_ALBEDO;
use raylib::drawing::{RaylibDraw, RaylibDraw3D, RaylibMode3DExt};
use raylib::ffi::KeyboardKey::KEY_E;
use raylib::ffi::{
    rlBegin, rlColor4f, rlDisableBackfaceCulling, rlDisableTexture, rlEnableTexture, rlEnd, rlSetLineWidth,
    rlSetPointSize, rlSetTexture, rlTexCoord2f, rlVertex3f, Texture2D, RL_TRIANGLES,
};
use raylib::math::Vector3;
use raylib::models::{Mesh, RaylibMaterial, RaylibMesh, RaylibModel, WeakMesh};
use raylib::texture::Image;

pub const OBSERVER_POS: Vector3 = Vector3::new(0.0, 0.0, 2.0);
pub const JUGEMU_POS_ISO: Vector3 = Vector3::new(3.0, 1.0, 3.0);

fn main() {
    let mut NDC_MODE: u32 = 0;

    let mut mesh_rotation = 0.0f32;
    let mut render = RaylibRenderer::init(N64_WIDTH, N64_HEIGHT);

    let near_clip_plane: f32 = 1.0;
    let far_clip_plane: f32 = 3.0;

    let main_observer = Camera3D {
        position: OBSERVER_POS,
        target: Vector3::ZERO,
        up: Vector3::Y,
        fovy: FOVY_PERSPECTIVE,
        projection: CameraProjection::CAMERA_PERSPECTIVE,
    };

    let screen_w = render.handle.get_screen_width();
    let screen_h = render.handle.get_screen_height();
    let aspect = screen_w as f32 / screen_h as f32;

    let mut jugemu = Camera3D {
        // position: OBSERVER_POS,
        position: JUGEMU_POS_ISO,
        target: Vector3::ZERO,
        up: Vector3::Y,
        fovy: FOVY_PERSPECTIVE,
        projection: CameraProjection::CAMERA_PERSPECTIVE,
    };
    println!(
        "jugemu init pos: ({:.3}, {:.3}, {:.3})",
        jugemu.position.x, jugemu.position.y, jugemu.position.z
    );
    let mut idle_timer = 0.0f32;
    let mut moved_since_last_log = false;
    let mut jugemu_roll_rotation = 0.0f32;

    // let mut main_model = render.handle.load_model(&render.thread, SPHERE_PATH).unwrap();
    let mut cube_mesh = Mesh::try_gen_mesh_cube(&render.thread, 1.0, 1.0, 1.0).unwrap();
    let mut main_model = render
        .handle
        .load_model_from_mesh(&render.thread, unsafe { cube_mesh.make_weak() })
        .unwrap();

    //TODO: this will automatically color the wire mesh and points.
    // apply_barycentric_palette(&mut main_model.meshes_mut()[0]);
    let max_vertices = main_model.meshes()[0].vertexCount as usize;
    let initial_vertices = vec![Vector3::ZERO; max_vertices];
    let mut ndc_mesh_builder = Mesh::init_mesh(&initial_vertices);
    if let Some(src_indices) = main_model.meshes()[0].indices() {
        //TODO: ughhh shouldnt this literally be a inherent handled thing in the meshb uilder? with options?
        ndc_mesh_builder = ndc_mesh_builder.indices(src_indices);
    }
    if let Some(src_texcoords) = main_model.meshes()[0].texcoords() {
        //TODO: ughhh shouldnt this literally be a inherent handled thing in the meshb uilder? with options?
        ndc_mesh_builder = ndc_mesh_builder.texcoords(src_texcoords);
    }
    let mut ndc_mesh = ndc_mesh_builder.build(&render.thread).unwrap();
    let mut ndc_model = render
        .handle
        .load_model_from_mesh(&render.thread, unsafe { ndc_mesh.make_weak() })
        .unwrap();
    // apply_barycentric_palette(&mut ndc_model.meshes_mut()[0]);
    let mut near_plane_intersectional_disk_mesh = Mesh::init_mesh(&initial_vertices).build(&render.thread).unwrap();
    let mut near_plane_intersectional_disk_model = render
        .handle
        .load_model_from_mesh(&render.thread, unsafe {
            near_plane_intersectional_disk_mesh.make_weak()
        })
        .unwrap();

    let checked_img = Image::gen_image_checked(16, 16, 4, 4, Color::BLACK, Color::WHITE);
    let checked_texture = render
        .handle
        .load_texture_from_image(&render.thread, &checked_img)
        .unwrap()
        .to_raw();
    main_model.materials_mut()[0].maps_mut()[MATERIAL_MAP_ALBEDO as usize].texture = checked_texture;
    ndc_model.materials_mut()[0].maps_mut()[MATERIAL_MAP_ALBEDO as usize].texture = checked_texture;
    while !render.handle.window_should_close() {
        if render.handle.is_key_pressed(KEY_E) {
            NDC_MODE = NDC_MODE.wrapping_add(1);
        }
        let delta_time = render.handle.get_frame_time();
        mesh_rotation -= ANGULAR_VELOCITY * delta_time;
        let jugemu_moved = move_jugemu_orbital(&mut jugemu, &render.handle, delta_time);
        if jugemu_moved {
            moved_since_last_log = true;
            idle_timer = 0.0;
        } else if moved_since_last_log {
            idle_timer += delta_time;
            if idle_timer >= 1.0 {
                println!(
                    "jugemu stopped at: ({:.3}, {:.3}, {:.3})",
                    jugemu.position.x, jugemu.position.y, jugemu.position.z
                );
                moved_since_last_log = false;
                idle_timer = 0.0;
            }
        }
        let mut draw_handle = render.handle.begin_drawing(&render.thread);
        draw_handle.clear_background(Color::BLACK);

        draw_handle.draw_mode3D(jugemu, |mut rl3d| {
            draw_observed_axes(&mut rl3d, &main_observer);
            if (NDC_MODE & 1) == 1 {
                unsafe { rlDisableBackfaceCulling() };
                main_model.materials_mut()[0].maps_mut()[MATERIAL_MAP_ALBEDO as usize]
                    .texture
                    .id = checked_texture.id;
                map_frustum_to_ndc_cube(
                    &mut rl3d,
                    screen_w,
                    screen_h,
                    &main_observer,
                    near_clip_plane,
                    far_clip_plane,
                    &mut main_model,
                    &mut ndc_model,
                    MODEL_POS,
                    MODEL_SCALE,
                    mesh_rotation,
                );
                rl3d.draw_model_ex(
                    &ndc_model,
                    MODEL_POS,
                    Vector3::Y,
                    mesh_rotation.to_degrees(),
                    MODEL_SCALE,
                    Color::WHITE,
                );
                main_model.materials_mut()[0].maps_mut()[MATERIAL_MAP_ALBEDO as usize]
                    .texture
                    .id = 0;
                rl3d.draw_model_wires_ex(
                    &ndc_model,
                    MODEL_POS,
                    Vector3::Y,
                    mesh_rotation.to_degrees(),
                    MODEL_SCALE,
                    Color::BLUE,
                );
                rl3d.draw_model_points_ex(
                    &ndc_model,
                    MODEL_POS,
                    Vector3::Y,
                    mesh_rotation.to_degrees(),
                    MODEL_SCALE,
                    Color::GREEN,
                );
                let ndc_topology = Topology::build_topology(&ndc_model.meshes()[0])
                    .triangles()
                    .vertices_per_triangle()
                    .vertex_normals()
                    .welded_vertices()
                    .welded_vertices_per_triangle()
                    .neighbors_per_triangle()
                    .front_triangles(mesh_rotation, &main_observer)
                    .silhouette_triangles()
                    .build();

                draw_near_plane_intersectional_disk_mesh(
                    &mut rl3d,
                    &main_observer,
                    near_clip_plane,
                    &mut near_plane_intersectional_disk_model,
                    MODEL_POS,
                    MODEL_SCALE,
                    mesh_rotation,
                    &ndc_topology,
                    true,
                );
                perspective_INCORRECT_projection_didactic(
                    screen_w,
                    screen_h,
                    &main_observer,
                    near_clip_plane,
                    &ndc_model.meshes()[0],
                    MODEL_POS,
                    MODEL_SCALE,
                    mesh_rotation,
                    checked_texture,
                );
            } else {
                //TODO: this should be double checked with your PR's iann!!!
                main_model.materials_mut()[0].maps_mut()[MATERIAL_MAP_ALBEDO as usize]
                    .texture
                    .id = checked_texture.id;
                rl3d.draw_model_ex(
                    &main_model,
                    MODEL_POS,
                    Vector3::Y,
                    mesh_rotation.to_degrees(),
                    MODEL_SCALE,
                    Color::WHITE,
                );
                //TODO: this should be double checked with your PR's iann!!!
                main_model.materials_mut()[0].maps_mut()[MATERIAL_MAP_ALBEDO as usize]
                    .texture
                    .id = 0;
                unsafe { rlSetLineWidth(2.0) };
                rl3d.draw_model_wires_ex(
                    &main_model,
                    MODEL_POS,
                    Vector3::Y,
                    mesh_rotation.to_degrees(),
                    MODEL_SCALE,
                    Color::BLUE,
                );
                unsafe { rlSetPointSize(8.0) };
                rl3d.draw_model_points_ex(
                    &main_model,
                    MODEL_POS,
                    Vector3::Y,
                    mesh_rotation.to_degrees(),
                    MODEL_SCALE,
                    Color::GREEN,
                );

                draw_frustum(&mut rl3d, aspect, &main_observer, near_clip_plane, far_clip_plane);
                let topology = Topology::build_topology(&main_model.meshes()[0])
                    .triangles()
                    .vertices_per_triangle()
                    .vertex_normals()
                    .welded_vertices()
                    .welded_vertices_per_triangle()
                    .neighbors_per_triangle()
                    .front_triangles(mesh_rotation, &main_observer)
                    .silhouette_triangles()
                    .build();

                draw_near_plane_intersectional_disk_mesh(
                    &mut rl3d,
                    &main_observer,
                    near_clip_plane,
                    &mut near_plane_intersectional_disk_model,
                    MODEL_POS,
                    MODEL_SCALE,
                    mesh_rotation,
                    &topology,
                    false,
                );
                perspective_INCORRECT_projection_didactic(
                    screen_w,
                    screen_h,
                    &main_observer,
                    near_clip_plane,
                    &main_model.meshes()[0],
                    MODEL_POS,
                    MODEL_SCALE,
                    mesh_rotation,
                    checked_texture,
                );
            }
        });
    }
}

fn perspective_INCORRECT_projection_didactic(
    screen_w: i32,
    screen_h: i32,
    main_observer: &Camera3D,
    near_clip_plane: f32,
    mesh: &WeakMesh,
    model_position: Vector3,
    model_scale: Vector3,
    mesh_rotation_radians: f32,
    texture: Texture2D,
) {
    let (observed_line_of_sight, observed_right, observed_up) = observed_orthonormal_basis_vectors(main_observer);
    let aspect = screen_w as f32 / screen_h as f32;
    let center_near_clip_plane = main_observer.position + observed_line_of_sight * near_clip_plane;
    let half_fovy = main_observer.fovy.to_radians() * 0.5;
    let half_height_near_clip_plane = near_clip_plane * half_fovy.tan();
    let half_width_near_clip_plane = half_height_near_clip_plane; // * aspect; //TODO* figure out all the aspect stuff
    let (sine, cosine) = mesh_rotation_radians.sin_cos();
    let vertices = mesh.vertices();
    let texcoords = mesh.texcoords();
    unsafe {
        // rlEnableDepthTest();
        // rlEnableDepthMask();
        // rlEnableBackfaceCulling();
        // rlDisableColorBlend();
        rlColor4f(1.0, 1.0, 1.0, 1.0);
        rlEnableTexture(texture.id);
        rlSetTexture(texture.id);
        rlBegin(RL_TRIANGLES as i32);
    }
    for [i_a, i_b, i_c] in mesh.triangles() {
        let mut world_a = vertices[i_a];
        let mut world_b = vertices[i_b];
        let mut world_c = vertices[i_c];
        {
            let scaled_x = world_a.x * model_scale.x;
            let scaled_y = world_a.y * model_scale.y;
            let scaled_z = world_a.z * model_scale.z;
            let rotated_x = cosine * scaled_x + sine * scaled_z;
            let rotated_z = -sine * scaled_x + cosine * scaled_z;
            world_a = Vector3::new(
                rotated_x + model_position.x,
                scaled_y + model_position.y,
                rotated_z + model_position.z,
            );
        }
        {
            let scaled_x = world_b.x * model_scale.x;
            let scaled_y = world_b.y * model_scale.y;
            let scaled_z = world_b.z * model_scale.z;
            let rotated_x = cosine * scaled_x + sine * scaled_z;
            let rotated_z = -sine * scaled_x + cosine * scaled_z;
            world_b = Vector3::new(
                rotated_x + model_position.x,
                scaled_y + model_position.y,
                rotated_z + model_position.z,
            );
        }
        {
            let scaled_x = world_c.x * model_scale.x;
            let scaled_y = world_c.y * model_scale.y;
            let scaled_z = world_c.z * model_scale.z;
            let rotated_x = cosine * scaled_x + sine * scaled_z;
            let rotated_z = -sine * scaled_x + cosine * scaled_z;
            world_c = Vector3::new(
                rotated_x + model_position.x,
                scaled_y + model_position.y,
                rotated_z + model_position.z,
            );
        }
        let to_a = world_a - main_observer.position;
        let to_b = world_b - main_observer.position;
        let to_c = world_c - main_observer.position;
        let depth_a = to_a.dot(observed_line_of_sight);
        let depth_b = to_b.dot(observed_line_of_sight);
        let depth_c = to_c.dot(observed_line_of_sight);
        let t_a = near_clip_plane / depth_a;
        let t_b = near_clip_plane / depth_b;
        let t_c = near_clip_plane / depth_c;
        let hit_a = main_observer.position + to_a * t_a;
        let hit_b = main_observer.position + to_b * t_b;
        let hit_c = main_observer.position + to_c * t_c;
        const NEAR_CLIP_PLANE_SHIFT_EPSILON: f32 = 0.0001; //TODO: ugh
                                                           //TODO: THIS IS WHERE THE Y UP Y DOWN THING SHOULD BE DIDACTIC!!!!!!!!!!!!
        let plane_a = hit_a + observed_line_of_sight * NEAR_CLIP_PLANE_SHIFT_EPSILON;
        let plane_b = hit_b + observed_line_of_sight * NEAR_CLIP_PLANE_SHIFT_EPSILON;
        let plane_c = hit_c + observed_line_of_sight * NEAR_CLIP_PLANE_SHIFT_EPSILON;
        let from_near_a = hit_a - center_near_clip_plane;
        let from_near_b = hit_b - center_near_clip_plane;
        let from_near_c = hit_c - center_near_clip_plane;
        let s_a = (from_near_a.dot(observed_right) / half_width_near_clip_plane) * 0.5 + 0.5;
        let t_a = (from_near_a.dot(observed_up) / half_width_near_clip_plane) * 0.5 + 0.5;
        let s_b = (from_near_b.dot(observed_right) / half_width_near_clip_plane) * 0.5 + 0.5;
        let t_b = (from_near_b.dot(observed_up) / half_width_near_clip_plane) * 0.5 + 0.5;
        let s_c = (from_near_c.dot(observed_right) / half_width_near_clip_plane) * 0.5 + 0.5;
        let t_c = (from_near_c.dot(observed_up) / half_width_near_clip_plane) * 0.5 + 0.5;
        let (u_a, v_a) = match texcoords {
            Some(tex) => (tex[i_a].x, tex[i_a].y),
            None => (s_a, t_a),
        };
        let (u_b, v_b) = match texcoords {
            Some(tex) => (tex[i_b].x, tex[i_b].y),
            None => (s_b, t_b),
        };
        let (u_c, v_c) = match texcoords {
            Some(tex) => (tex[i_c].x, tex[i_c].y),
            None => (s_c, t_c),
        };
        unsafe {
            rlTexCoord2f(u_a, v_a);
            rlVertex3f(plane_a.x, plane_a.y, plane_a.z);
            rlTexCoord2f(u_b, v_b);
            rlVertex3f(plane_b.x, plane_b.y, plane_b.z);
            rlTexCoord2f(u_c, v_c);
            rlVertex3f(plane_c.x, plane_c.y, plane_c.z);
        }
    }
    unsafe {
        rlEnd();
        rlDisableTexture();
    }
}
