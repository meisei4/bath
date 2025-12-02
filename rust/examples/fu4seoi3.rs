use asset_payload::{CHI_CONFIG_PATH, FONT_IMAGE_PATH, FONT_PATH, SPHERE_GLTF_PATH, SPHERE_PATH};
use bath::fu4seoi3::core::*;
use bath::fu4seoi3::draw::*;
use raylib::consts::CameraProjection::{CAMERA_ORTHOGRAPHIC, CAMERA_PERSPECTIVE};
use raylib::consts::MaterialMapIndex::MATERIAL_MAP_ALBEDO;
use raylib::prelude::*;

const NUM_MODELS: usize = 2;

fn main() {
    let mut i_time = 0.0f32;
    let mut total_time = 0.0f32;
    let mut view_state = ViewState::new();
    let mut placed_cells: Vec<PlacedCell> = Vec::new();
    let (mut handle, thread) = init()
        .size(DC_WIDTH, DC_HEIGHT)
        .title("raylib [core] example - fixed function didactic")
        .build();

    let font_image = Image::load_image(FONT_IMAGE_PATH).unwrap();
    let font = unsafe {
        handle
            .load_font_ex(&thread, FONT_PATH, 32, None)
            .expect("Failed to load font")
            .make_weak()
    };

    let font = handle.get_font_default();

    let near = 1.0;
    let far = 3.0;
    let mut aspect = handle.get_screen_width() as f32 / handle.get_screen_height() as f32;
    let mut mesh_rotation = 0.0f32;

    let mut main = Camera3D {
        position: MAIN_POS,
        target: MODEL_POS,
        up: Y_AXIS,
        fovy: if view_state.ortho_mode {
            NEAR_PLANE_HEIGHT_ORTHOGRAPHIC()
        } else {
            FOVY_PERSPECTIVE
        },
        projection: if view_state.ortho_mode {
            CAMERA_ORTHOGRAPHIC
        } else {
            CAMERA_PERSPECTIVE
        },
    };
    let mut jugemu = Camera3D {
        position: JUGEMU_POS_ISO.normalize()
            * if view_state.jugemu_ortho_mode {
                JUGEMU_DISTANCE_ORTHO
            } else {
                JUGEMU_DISTANCE_PERSPECTIVE
            },
        target: MODEL_POS,
        up: Y_AXIS,
        fovy: FOVY_ORTHOGRAPHIC,
        projection: CAMERA_ORTHOGRAPHIC,
    };

    let mut prev_fovy_ortho = FOVY_ORTHOGRAPHIC;
    let mut prev_fovy_perspective = FOVY_PERSPECTIVE;
    let mut prev_distance_ortho = JUGEMU_DISTANCE_ORTHO;
    let mut prev_distance_perspective = JUGEMU_DISTANCE_PERSPECTIVE;

    let mut world_models: Vec<Model> = Vec::new();
    let mut ndc_models: Vec<Model> = Vec::new();
    let mut mesh_textures = Vec::new();

    let mut ghost_model = handle.load_model(&thread, SPHERE_GLTF_PATH).unwrap();
    let checked_img = Image::gen_image_checked(16, 16, 1, 1, Color::BLACK, Color::WHITE);
    let mesh_texture = handle.load_texture_from_image(&thread, &checked_img).unwrap();
    ghost_model.materials_mut()[0].set_material_texture(MATERIAL_MAP_ALBEDO, &mesh_texture);

    let mesh_samples = collect_deformed_vertex_samples(ghost_model.meshes()[0].vertices());
    let mut preload_dynamic_metrics_for_ghost = FrameDynamicMetrics::new();
    interpolate_between_deformed_vertices(
        &mut ghost_model,
        i_time,
        &mesh_samples,
        &mut preload_dynamic_metrics_for_ghost,
    );

    let ghost_ndc_mesh = {
        let world_mesh = &ghost_model.meshes()[0];
        Mesh::init_mesh(world_mesh.vertices())
            .texcoords_opt(world_mesh.texcoords())
            .colors_opt(world_mesh.colors())
            .normals_opt(world_mesh.normals())
            .indices_opt(world_mesh.indices())
            .build_dynamic(&thread)
            .unwrap()
    };
    let mut ghost_ndc_model = handle.load_model_from_mesh(&thread, ghost_ndc_mesh).unwrap();
    ghost_ndc_model.materials_mut()[0].set_material_texture(MATERIAL_MAP_ALBEDO, &mesh_texture);

    world_models.push(ghost_model);
    ndc_models.push(ghost_ndc_model);
    mesh_textures.push(mesh_texture);

    let texture_config: [i32; NUM_MODELS] = [4, 16];

    for i in 0..NUM_MODELS {
        let mut world_model = match i {
            0 => handle
                .load_model_from_mesh(&thread, Mesh::try_gen_mesh_cube(&thread, 1.0, 1.0, 1.0).unwrap())
                .unwrap(),
            _ => handle.load_model(&thread, SPHERE_PATH).expect("load sphere obj"),
        };

        let world_mesh = &mut world_model.meshes_mut()[0];
        fill_planar_texcoords(world_mesh);
        fill_vertex_colors(world_mesh);

        let checked_img =
            Image::gen_image_checked(texture_config[i], texture_config[i], 1, 1, Color::BLACK, Color::WHITE);
        let mesh_texture = handle.load_texture_from_image(&thread, &checked_img).unwrap();
        world_model.materials_mut()[0].set_material_texture(MATERIAL_MAP_ALBEDO, &mesh_texture);

        let ndc_mesh = {
            let world_mesh = &world_model.meshes()[0];
            Mesh::init_mesh(world_mesh.vertices())
                .texcoords_opt(world_mesh.texcoords())
                .colors_opt(world_mesh.colors())
                .normals_opt(world_mesh.normals())
                .indices_opt(world_mesh.indices())
                .build_dynamic(&thread)
                .unwrap()
        };
        let mut ndc_model = handle.load_model_from_mesh(&thread, ndc_mesh).unwrap();
        ndc_model.materials_mut()[0].set_material_texture(MATERIAL_MAP_ALBEDO, &mesh_texture);

        world_models.push(world_model);
        ndc_models.push(ndc_model);
        mesh_textures.push(mesh_texture);
    }

    let mut preload_dynamic_metrics = FrameDynamicMetrics::new();

    let mut cached_ndc_z_shifts_anisotropic = Vec::new();
    let mut cached_ndc_z_shifts_isotropic = Vec::new();

    for i in 0..world_models.len() {
        world_to_ndc_space(
            &mut main,
            aspect,
            near,
            far,
            &world_models[i],
            &mut ndc_models[i],
            0.0,
            0.0,
            1.0,
            &mut preload_dynamic_metrics,
        );
        let z_shift_aniso = calculate_average_ndc_z_shift(&world_models[i], &ndc_models[i]);
        cached_ndc_z_shifts_anisotropic.push(z_shift_aniso);
        world_to_ndc_space(
            &mut main,
            aspect,
            near,
            far,
            &world_models[i],
            &mut ndc_models[i],
            0.0,
            0.0,
            0.0,
            &mut preload_dynamic_metrics,
        );
        let z_shift_iso = calculate_average_ndc_z_shift(&world_models[i], &ndc_models[i]);
        cached_ndc_z_shifts_isotropic.push(z_shift_iso);
    }

    let mut spatial_frame_model = {
        let mut temp_cube = Mesh::try_gen_mesh_cube(&thread, 1.0, 1.0, 1.0).unwrap();
        let colors = temp_cube.init_colors_mut().unwrap();
        colors.fill(Color { a: 0, ..Color::WHITE });
        colors.iter_mut().take(4).for_each(|c| c.a = 255);
        let spatial_frame_mesh = Mesh::init_mesh(temp_cube.vertices())
            .texcoords_opt(temp_cube.texcoords())
            .colors_opt(temp_cube.colors())
            .indices_opt(temp_cube.indices())
            .build_dynamic(&thread)
            .unwrap();

        handle.load_model_from_mesh(&thread, spatial_frame_mesh).unwrap()
    };

    handle.set_target_fps(60);
    let mut frame_dynamic_metrics = FrameDynamicMetrics::new();

    let mut room = Room::default();
    let mut config_watcher = ConfigWatcher::new(CHI_CONFIG_PATH);
    while !handle.window_should_close() {
        if let Some(new_config) = config_watcher.check_reload() {
            room.reload_config(new_config);
        }
        let dt = handle.get_frame_time();
        aspect = handle.get_screen_width() as f32 / handle.get_screen_height() as f32;
        frame_dynamic_metrics.reset();

        handle_view_toggles(&handle, &mut view_state);
        handle_jugemu_projection_toggle(
            &handle,
            &mut view_state,
            &mut jugemu,
            &mut prev_fovy_ortho,
            &mut prev_fovy_perspective,
            &mut prev_distance_ortho,
            &mut prev_distance_perspective,
        );
        handle_mesh_selection(&handle, &mut view_state);

        update_blend(&mut view_state.space_blend, dt, view_state.ndc_space);
        update_blend(&mut view_state.aspect_blend, dt, view_state.aspect_correct);
        update_blend(&mut view_state.ortho_blend, dt, view_state.ortho_mode);

        if !view_state.paused {
            mesh_rotation -= ANGULAR_VELOCITY * dt;
            total_time += dt;
            i_time += dt;
        }

        jugemu.projection = if view_state.jugemu_ortho_mode {
            CAMERA_ORTHOGRAPHIC
        } else {
            CAMERA_PERSPECTIVE
        };

        orbit_space(&mut handle, &mut jugemu);

        main.projection = if view_state.ortho_mode {
            CAMERA_ORTHOGRAPHIC
        } else {
            CAMERA_PERSPECTIVE
        };
        main.fovy = if view_state.ortho_mode {
            NEAR_PLANE_HEIGHT_ORTHOGRAPHIC()
        } else {
            FOVY_PERSPECTIVE
        };

        let target_mesh = view_state.target_mesh_index;
        if target_mesh == 0 && !view_state.paused {
            update_ghost_mesh(
                &mut ndc_models[0],
                &mut world_models[0],
                i_time,
                &mesh_samples,
                &main,
                mesh_rotation,
                &mut frame_dynamic_metrics,
            );
        }

        world_to_ndc_space(
            &mut main,
            aspect,
            near,
            far,
            &mut world_models[target_mesh],
            &mut ndc_models[target_mesh],
            mesh_rotation,
            view_state.ortho_blend,
            view_state.aspect_blend,
            &mut frame_dynamic_metrics,
        );

        blend_world_and_ndc_vertices(
            &world_models[target_mesh],
            &mut ndc_models[target_mesh],
            view_state.space_blend,
            &mut frame_dynamic_metrics,
        );

        let z_shift_for_aspect = lerp(
            cached_ndc_z_shifts_isotropic[target_mesh],
            cached_ndc_z_shifts_anisotropic[target_mesh],
            view_state.aspect_blend,
        );
        jugemu.target = Vector3::new(
            MODEL_POS.x,
            MODEL_POS.y,
            lerp(MODEL_POS.z, z_shift_for_aspect, view_state.space_blend),
        );

        update_spatial_frame(
            &mut main,
            aspect,
            near,
            far,
            &mut spatial_frame_model.meshes_mut()[0],
            view_state.space_blend,
            view_state.aspect_blend,
            view_state.ortho_blend,
        );

        let hover_state = compute_hover_state(&handle, &jugemu, &placed_cells);

        if let Some(cell_idx) = hover_state.placed_cell_index {
            if handle.is_key_pressed(KeyboardKey::KEY_T) {
                placed_cells[cell_idx].texture_enabled = !placed_cells[cell_idx].texture_enabled;
            }
            if handle.is_key_pressed(KeyboardKey::KEY_C) {
                placed_cells[cell_idx].color_enabled = !placed_cells[cell_idx].color_enabled;
            }
        }

        if handle.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            if let (Some((ix, iy, iz)), false) = (hover_state.indices, hover_state.is_occupied()) {
                placed_cells.push(PlacedCell {
                    ix,
                    iy,
                    iz,
                    mesh_index: view_state.target_mesh_index,
                    placed_time: total_time,
                    settled: false,
                    texture_enabled: view_state.texture_mode,
                    color_enabled: view_state.color_mode,
                });
            }
        }

        let (depth, right, up) = basis_vector(&main);
        let mut draw_handle = handle.begin_drawing(&thread);
        draw_handle.clear_background(Color::BLACK);
        draw_handle.draw_mode3D(if view_state.jugemu_mode { jugemu } else { main }, |mut rl3d| {
            draw_camera_basis(&mut rl3d, &main, depth, right, up);

            if view_state.jugemu_mode {
                draw_spatial_frame(&mut rl3d, &spatial_frame_model.meshes_mut()[0]);
            }

            draw_room_floor_grid(&mut rl3d);
            rl3d.draw_cube_wires(MODEL_POS, ROOM_W as f32, ROOM_H as f32, ROOM_D as f32, RED_DAMASK);

            if let Some(center) = hover_state.center {
                rl3d.draw_cube_wires(center, 1.0, 1.0, 1.0, NEON_CARROT);
            }

            draw_placed_cells(
                &mut rl3d,
                &mut ndc_models,
                &mesh_textures,
                &mut placed_cells,
                total_time,
            );

            draw_model_filled_at(
                &mut rl3d,
                &mut ndc_models[target_mesh],
                &mesh_textures[target_mesh],
                MODEL_POS,
                mesh_rotation.to_degrees(),
                MODEL_SCALE,
                view_state.color_mode,
                view_state.texture_mode,
            );
            draw_model_wires_and_points_at(
                &mut rl3d,
                &mut ndc_models[target_mesh],
                MODEL_POS,
                mesh_rotation.to_degrees(),
                MODEL_SCALE,
                MARINER,
                LILAC,
            );

            if let Some(center) = hover_state.center {
                draw_hint_mesh(
                    &mut rl3d,
                    &mut ndc_models[target_mesh],
                    center,
                    mesh_rotation,
                    hover_state.is_occupied(),
                );
            }

            draw_chi_field(&mut rl3d, &room);
        });

        draw_hud(
            &mut draw_handle,
            &font,
            &view_state,
            &jugemu,
            target_mesh,
            &hover_state,
            &placed_cells,
            i_time,
            &world_models,
            &ndc_models,
            &mesh_samples,
            &frame_dynamic_metrics,
        );
    }
}
