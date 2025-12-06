use asset_payload::{
    CHI_CONFIG_PATH, FONT_PATH, FUSUMA_GLTF_PATH, SPHERE_GLTF_PATH, SPHERE_PATH, VIEW_CONFIG_PATH, WINDOW_GLTF_PATH,
};
use bath::fu4seoi3::config_and_state::*;
use bath::fu4seoi3::core::*;
use bath::fu4seoi3::draw::*;
use bath::fu4seoi3::hud::*;
use raylib::consts::CameraProjection::{CAMERA_ORTHOGRAPHIC, CAMERA_PERSPECTIVE};
use raylib::consts::MaterialMapIndex::MATERIAL_MAP_ALBEDO;
use raylib::prelude::*;

const NUM_MODELS: usize = 2;

fn main() {
    let mut i_time = 0.0f32;
    let mut total_time = 0.0f32;
    let mut view_state = ViewState::new();
    let mut placed_cells: Vec<PlacedCell> = Vec::new();
    let mut edit_stack: Vec<EditStack> = Vec::new();
    let mut edit_cursor: usize = 0;

    let (mut handle, thread) = init()
        .size(DC_WIDTH, DC_HEIGHT)
        .title("raylib [core] example - fixed function didactic")
        .build();

    let font = handle
        .load_font_ex(&thread, FONT_PATH, 32, None)
        .expect("Failed to load font")
        .make_weak();

    let font = handle.get_font_default();

    let near = 1.0;
    let far = 3.0;
    let mut aspect = handle.get_screen_width() as f32 / handle.get_screen_height() as f32;
    let mut mesh_rotation = 0.0f32;

    let mut field_config_watcher: ConfigWatcher<FieldConfig> =
        ConfigWatcher::new(CHI_CONFIG_PATH, FieldConfig::load_from_file);
    let mut view_config_watcher: ConfigWatcher<ViewConfig> =
        ConfigWatcher::new(VIEW_CONFIG_PATH, ViewConfig::load_from_file);

    let mut field_config = FieldConfig::default();
    let mut view_config = ViewConfig::default();

    let mut main = Camera3D {
        position: MAIN_POS,
        target: MODEL_POS,
        up: Y_AXIS,
        fovy: if view_state.ortho_mode {
            near_plane_height_orthographic(&view_config)
        } else {
            view_config.fovy_perspective
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
                view_config.jugemu_distance_ortho
            } else {
                view_config.jugemu_distance_perspective
            },
        target: MODEL_POS,
        up: Y_AXIS,
        fovy: view_config.fovy_orthographic,
        projection: CAMERA_ORTHOGRAPHIC,
    };

    view_state.jugemu_state.fovy_ortho = view_config.fovy_orthographic;
    view_state.jugemu_state.fovy_perspective = view_config.fovy_perspective;
    view_state.jugemu_state.distance_ortho = view_config.jugemu_distance_ortho;
    view_state.jugemu_state.distance_perspective = view_config.jugemu_distance_perspective;

    let checked_img = Image::gen_image_checked(16, 16, 1, 1, Color::BLACK, Color::WHITE);
    let checked_texture = handle
        .load_texture_from_image(&thread, &checked_img)
        .expect("Failed to create texture");

    let mut fusuma = handle
        .load_model(&thread, FUSUMA_GLTF_PATH)
        .expect("Failed to load fusuma GLTF");

    let fusuma_bb = &fusuma.get_model_bounding_box();
    fill_planar_texcoords(&mut fusuma.meshes_mut()[0]);
    fusuma.materials_mut()[0].set_material_texture(MATERIAL_MAP_ALBEDO, &checked_texture);

    let mut window = handle
        .load_model(&thread, WINDOW_GLTF_PATH)
        .expect("Failed to load fusuma GLTF");

    let window_bb = &window.get_model_bounding_box();
    fill_planar_texcoords(&mut window.meshes_mut()[0]);
    window.materials_mut()[0].set_material_texture(MATERIAL_MAP_ALBEDO, &checked_texture);

    let mut opening_models: Vec<Model> = vec![fusuma, window];
    let opening_metrics: Vec<MeshMetrics> = opening_models
        .iter()
        .map(|m| MeshMetrics::measure(&m.meshes()[0]))
        .collect();
    let mut meshes: Vec<MeshDescriptor> = Vec::new();

    let mut world_ghost = handle
        .load_model(&thread, SPHERE_GLTF_PATH)
        .expect("Failed to load ghost GLTF");

    world_ghost.materials_mut()[0].set_material_texture(MATERIAL_MAP_ALBEDO, &checked_texture);

    let world_ghost_pre_animation_vertices = world_ghost.meshes()[0].vertices().to_vec();

    let mut mesh_samples = collect_deformed_vertex_samples(world_ghost.meshes()[0].vertices(), &field_config);
    let mut preload_dynamic_metrics_for_ghost = FrameDynamicMetrics::new();
    interpolate_between_deformed_vertices(
        &mut world_ghost,
        i_time,
        &mesh_samples,
        &mut preload_dynamic_metrics_for_ghost,
        &field_config,
    );

    let ndc_ghost_mesh = {
        let world_mesh = &world_ghost.meshes()[0];
        Mesh::init_mesh(world_mesh.vertices())
            .texcoords_opt(world_mesh.texcoords())
            .colors_opt(world_mesh.colors())
            .normals_opt(world_mesh.normals())
            .indices_opt(world_mesh.indices())
            .build_dynamic(&thread)
            .unwrap()
    };
    let mut ndc_ghost = handle
        .load_model_from_mesh(&thread, ndc_ghost_mesh)
        .expect("Failed to create ghost NDC model");
    ndc_ghost.materials_mut()[0].set_material_texture(MATERIAL_MAP_ALBEDO, &checked_texture);

    let ghost_metrics_world = MeshMetrics::measure(&world_ghost.meshes()[0]);
    let ghost_metrics_ndc = MeshMetrics::measure(&ndc_ghost.meshes()[0]);
    let ghost_combined_bytes = ghost_metrics_world.total_bytes + ghost_metrics_ndc.total_bytes;

    meshes.push(MeshDescriptor {
        name: "GHOST",
        world: world_ghost,
        ndc: ndc_ghost,
        texture: checked_texture,
        metrics_world: ghost_metrics_world,
        metrics_ndc: ghost_metrics_ndc,
        combined_bytes: ghost_combined_bytes,
        z_shift_anisotropic: 0.0,
        z_shift_isotropic: 0.0,
    });

    let texture_config: [i32; NUM_MODELS] = [4, 16];

    for i in 0..NUM_MODELS {
        let (name, world_model) = match i {
            0 => (
                "CUBE",
                handle
                    .load_model_from_mesh(&thread, Mesh::try_gen_mesh_cube(&thread, 1.0, 1.0, 1.0).unwrap())
                    .unwrap(),
            ),
            _ => (
                "SPHERE",
                handle
                    .load_model(&thread, SPHERE_PATH)
                    .expect("Failed to load sphere OBJ"),
            ),
        };

        let mut world_model = world_model;

        {
            let world_mesh = &mut world_model.meshes_mut()[0];
            fill_planar_texcoords(world_mesh);
            fill_vertex_colors(world_mesh);
        }

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

        let metrics_world = MeshMetrics::measure(&world_model.meshes()[0]);
        let metrics_ndc = MeshMetrics::measure(&ndc_model.meshes()[0]);
        let combined_bytes = metrics_world.total_bytes + metrics_ndc.total_bytes;

        meshes.push(MeshDescriptor {
            name,
            world: world_model,
            ndc: ndc_model,
            texture: mesh_texture,
            metrics_world,
            metrics_ndc,
            combined_bytes,
            z_shift_anisotropic: 0.0,
            z_shift_isotropic: 0.0,
        });
    }

    let mut preload_dynamic_metrics = FrameDynamicMetrics::new();
    for mesh_descriptor in meshes.iter_mut() {
        world_to_ndc_space(
            &main,
            aspect,
            near,
            far,
            &mesh_descriptor.world,
            &mut mesh_descriptor.ndc,
            0.0,
            0.0,
            1.0,
            &mut preload_dynamic_metrics,
            &view_config,
        );
        mesh_descriptor.z_shift_anisotropic =
            calculate_average_ndc_z_shift(&mesh_descriptor.world, &mesh_descriptor.ndc);

        world_to_ndc_space(
            &main,
            aspect,
            near,
            far,
            &mesh_descriptor.world,
            &mut mesh_descriptor.ndc,
            0.0,
            0.0,
            0.0,
            &mut preload_dynamic_metrics,
            &view_config,
        );
        mesh_descriptor.z_shift_isotropic = calculate_average_ndc_z_shift(&mesh_descriptor.world, &mesh_descriptor.ndc);
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

    if let Some(door) = room.field.entities.get_mut(0) {
        door.model_index = Some(0);
        door.h0 = -fusuma_bb.min.y;
    }
    if let Some(window) = room.field.entities.get_mut(1) {
        window.model_index = Some(1);
        window.h0 = -window_bb.min.y;
    }

    let mut chi_field_model = build_chi_field_model(&mut handle, &thread, &room);
    chi_field_model.materials_mut()[0].set_material_texture(MATERIAL_MAP_ALBEDO, &meshes[0].texture);
    let mut animated_meshes_need_regeneration = false;

    while !handle.window_should_close() {
        if let Some(new_field_config) = field_config_watcher.check_reload() {
            let samples_changed = new_field_config.log_delta(&field_config);
            field_config = new_field_config;
            room.reload_config(field_config.clone());
            chi_field_model = build_chi_field_model(&mut handle, &thread, &room);
            chi_field_model.materials_mut()[0].set_material_texture(MATERIAL_MAP_ALBEDO, &meshes[0].texture);

            if samples_changed {
                animated_meshes_need_regeneration = true;
            }
        }

        if let Some(new_view_cfg) = view_config_watcher.check_reload() {
            new_view_cfg.log_delta(&view_config);
            view_config = new_view_cfg;

            view_state.jugemu_state.fovy_ortho = view_config.fovy_orthographic;
            view_state.jugemu_state.fovy_perspective = view_config.fovy_perspective;
            view_state.jugemu_state.distance_ortho = view_config.jugemu_distance_ortho;
            view_state.jugemu_state.distance_perspective = view_config.jugemu_distance_perspective;

            let jugemu_dir = jugemu.position.normalize();
            let jugemu_dist = if view_state.jugemu_ortho_mode {
                view_config.jugemu_distance_ortho
            } else {
                view_config.jugemu_distance_perspective
            };
            jugemu.position = jugemu_dir * jugemu_dist;

            jugemu.fovy = if view_state.jugemu_ortho_mode {
                view_config.fovy_orthographic
            } else {
                view_config.fovy_perspective
            };

            main.fovy = if view_state.ortho_mode {
                near_plane_height_orthographic(&view_config)
            } else {
                view_config.fovy_perspective
            };
        }

        if animated_meshes_need_regeneration {
            mesh_samples = collect_deformed_vertex_samples(&world_ghost_pre_animation_vertices, &field_config);
            println!(
                "{} Animated vertex samples regenerated: {} samples",
                timestamp(),
                mesh_samples.len()
            );
            animated_meshes_need_regeneration = false;
        }

        let dt = handle.get_frame_time();
        aspect = handle.get_screen_width() as f32 / handle.get_screen_height() as f32;
        frame_dynamic_metrics.reset();

        update_view_from_input(&handle, &mut view_state, &mut jugemu);
        update_blend(&mut view_state.space_blend, dt, view_state.ndc_space, &view_config);
        update_blend(
            &mut view_state.aspect_blend,
            dt,
            view_state.aspect_correct,
            &view_config,
        );
        update_blend(&mut view_state.ortho_blend, dt, view_state.ortho_mode, &view_config);

        if !view_state.paused {
            mesh_rotation -= angular_velocity(&field_config) * dt;
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
            near_plane_height_orthographic(&view_config)
        } else {
            view_config.fovy_perspective
        };

        let target_mesh = view_state.target_mesh_index;

        if target_mesh == 0 && !view_state.paused {
            let animated_mesh_descriptor = &mut meshes[0];
            update_animated_mesh(
                &mut animated_mesh_descriptor.ndc,
                &mut animated_mesh_descriptor.world,
                mesh_rotation,
                &mesh_samples,
                &main,
                mesh_rotation,
                &mut frame_dynamic_metrics,
                &field_config,
            );
        }

        {
            let mesh_descriptor = &mut meshes[target_mesh];
            world_to_ndc_space(
                &main,
                aspect,
                near,
                far,
                &mesh_descriptor.world,
                &mut mesh_descriptor.ndc,
                mesh_rotation,
                view_state.ortho_blend,
                view_state.aspect_blend,
                &mut frame_dynamic_metrics,
                &view_config,
            );

            blend_world_and_ndc_vertices(
                &mesh_descriptor.world,
                &mut mesh_descriptor.ndc,
                view_state.space_blend,
                &mut frame_dynamic_metrics,
            );
        }

        {
            let desc = &meshes[target_mesh];
            let z_shift_for_aspect = lerp(
                desc.z_shift_isotropic,
                desc.z_shift_anisotropic,
                view_state.aspect_blend,
            );
            jugemu.target = Vector3::new(
                MODEL_POS.x,
                MODEL_POS.y,
                lerp(MODEL_POS.z, z_shift_for_aspect, view_state.space_blend),
            );
        }

        update_spatial_frame(
            &main,
            aspect,
            near,
            far,
            &mut spatial_frame_model.meshes_mut()[0],
            view_state.space_blend,
            view_state.aspect_blend,
            view_state.ortho_blend,
            &view_config,
        );

        let hover_state = compute_hover_state(&handle, &jugemu, &room, &placed_cells);

        if let Some(cell_index) = hover_state.placed_cell_index {
            let (ix, iy, iz) = {
                let c = &placed_cells[cell_index];
                (c.ix, c.iy, c.iz)
            };

            if handle.is_key_pressed(KeyboardKey::KEY_T) {
                if edit_cursor < edit_stack.len() {
                    edit_stack.truncate(edit_cursor);
                }
                let op = EditStack::ToggleTexture {
                    ix,
                    iy,
                    iz,
                    time: total_time,
                };
                redo(&op, &mut placed_cells);
                edit_stack.push(op);
                edit_cursor += 1;
            }

            if handle.is_key_pressed(KeyboardKey::KEY_C) {
                if edit_cursor < edit_stack.len() {
                    edit_stack.truncate(edit_cursor);
                }
                let op = EditStack::ToggleColor {
                    ix,
                    iy,
                    iz,
                    time: total_time,
                };
                redo(&op, &mut placed_cells);
                edit_stack.push(op);
                edit_cursor += 1;
            }
            if handle.is_key_pressed(KeyboardKey::KEY_D) {
                //TODO i want this to be CTRL CLICK OR SOMETHING CLEARER??
                if edit_cursor < edit_stack.len() {
                    edit_stack.truncate(edit_cursor);
                }
                let cell = placed_cells[cell_index].clone();
                let op = EditStack::RemoveCell { cell, time: total_time };
                redo(&op, &mut placed_cells);
                edit_stack.push(op);
                edit_cursor += 1;
            }
        }

        if handle.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            if let (Some((ix, iy, iz)), false) = (hover_state.indices, hover_state.is_occupied()) {
                if edit_cursor < edit_stack.len() {
                    edit_stack.truncate(edit_cursor);
                }

                let cell = PlacedCell {
                    ix,
                    iy,
                    iz,
                    mesh_index: view_state.target_mesh_index,
                    placed_time: total_time,
                    settled: false,
                    texture_enabled: view_state.texture_mode,
                    color_enabled: view_state.color_mode,
                };

                let edit = EditStack::PlaceCell { cell, time: total_time };
                redo(&edit, &mut placed_cells);
                edit_stack.push(edit);
                edit_cursor += 1;
            }
        }

        if handle.is_key_down(KeyboardKey::KEY_LEFT_CONTROL)
            && !handle.is_key_down(KeyboardKey::KEY_LEFT_SHIFT)
            && handle.is_key_pressed(KeyboardKey::KEY_Z)
        {
            if edit_cursor > 0 {
                edit_cursor -= 1;
                let edit = &edit_stack[edit_cursor];
                undo(edit, &mut placed_cells);
                log_edit_stack(&edit_stack, edit_cursor);
            }
        }

        if handle.is_key_down(KeyboardKey::KEY_LEFT_CONTROL)
            && handle.is_key_down(KeyboardKey::KEY_LEFT_SHIFT)
            && handle.is_key_pressed(KeyboardKey::KEY_Z)
        {
            if edit_cursor < edit_stack.len() {
                let edit = &edit_stack[edit_cursor];
                redo(edit, &mut placed_cells);
                edit_cursor += 1;
                log_edit_stack(&edit_stack, edit_cursor);
            }
        }

        let (depth, right, up) = basis_vector(&main);
        let mut draw_handle = handle.begin_drawing(&thread);
        draw_handle.clear_background(Color::BLACK);

        let active_camera = if view_state.jugemu_mode { jugemu } else { main };

        draw_handle.draw_mode3D(active_camera, |mut rl3d| {
            draw_camera_basis(&mut rl3d, &main, depth, right, up);

            if view_state.jugemu_mode {
                draw_spatial_frame(&mut rl3d, &spatial_frame_model.meshes_mut()[0]);
            }

            draw_room_floor_grid(&mut rl3d, &room);
            rl3d.draw_cube_wires(MODEL_POS, ROOM_W as f32, ROOM_H as f32, ROOM_D as f32, RED_DAMASK);

            if let Some(center) = hover_state.center {
                rl3d.draw_cube_wires(center, 1.0, 1.0, 1.0, NEON_CARROT);
            }

            draw_placed_cells(
                &mut rl3d,
                &mut meshes,
                &mut placed_cells,
                total_time,
                &room,
                &view_config,
            );
            {
                let desc = &mut meshes[target_mesh];
                draw_filled_with_overlay(
                    &mut rl3d,
                    &mut desc.ndc,
                    &desc.texture,
                    MODEL_POS,
                    mesh_rotation.to_degrees(),
                    MODEL_SCALE,
                    view_state.color_mode,
                    view_state.texture_mode,
                    None,
                );
                if let Some(center) = hover_state.center {
                    let hint_scale =
                        Vector3::new(view_config.hint_scale, view_config.hint_scale, view_config.hint_scale);
                    draw_hint(
                        &mut rl3d,
                        &mut desc.ndc,
                        center,
                        mesh_rotation.to_degrees(),
                        hint_scale,
                        hover_state.is_occupied(),
                    );
                }
            }
            draw_chi_field(&mut rl3d, &room, &mut chi_field_model, &mut opening_models);
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
            &meshes,
            &mesh_samples,
            &frame_dynamic_metrics,
            &room,
            &edit_stack,
            edit_cursor,
            &opening_metrics,
        );
    }
}
