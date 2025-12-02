use crate::fu4seoi3::core::*;
use raylib::ffi;
use raylib::prelude::*;

pub fn draw_chi_field(rl3d: &mut RaylibMode3D<RaylibDrawHandle>, room: &Room) {
    unsafe {
        ffi::rlSetLineWidth(2.0);
    }

    for sample in &room.field_samples {
        let center = sample.position;
        let m = sample.magnitude.clamp(0.0, 1.0);
        let scaled_length = CHI_ARROW_LENGTH * m;
        let half = scaled_length * 0.5;

        let start = Vector3::new(
            center.x - sample.direction.x * half,
            center.y,
            center.z - sample.direction.y * half,
        );
        let end = Vector3::new(
            center.x + sample.direction.x * half,
            center.y,
            center.z + sample.direction.y * half,
        );

        rl3d.draw_line3D(start, end, SUNFLOWER);
    }

    for door in &room.doors {
        let color = if door.is_primary { ANAKIWA } else { LILAC };
        rl3d.draw_line3D(door.p1, door.p2, color);
        rl3d.draw_sphere(door.center(), 0.5, color);
    }

    for window in &room.windows {
        rl3d.draw_line3D(window.p1, window.p2, PALE_CANARY);
        rl3d.draw_sphere(window.center(), 0.25, PALE_CANARY);
    }

    unsafe {
        ffi::rlSetLineWidth(1.0);
    }
}

pub fn draw_camera_basis(
    rl3d: &mut RaylibMode3D<RaylibDrawHandle>,
    main: &Camera3D,
    depth: Vector3,
    right: Vector3,
    up: Vector3,
) {
    rl3d.draw_line3D(
        main.position,
        Vector3::new(
            main.position.x + right.x,
            main.position.y + right.y,
            main.position.z + right.z,
        ),
        NEON_CARROT,
    );
    rl3d.draw_line3D(
        main.position,
        Vector3::new(main.position.x + up.x, main.position.y + up.y, main.position.z + up.z),
        LILAC,
    );
    rl3d.draw_line3D(
        main.position,
        Vector3::new(
            main.position.x + depth.x,
            main.position.y + depth.y,
            main.position.z + depth.z,
        ),
        MARINER,
    );
}

pub fn draw_model_filled_at(
    rl3d: &mut RaylibMode3D<RaylibDrawHandle>,
    model: &mut Model,
    texture: &Texture2D,
    position: Vector3,
    rotation_deg: f32,
    scale: Vector3,
    color_enabled: bool,
    texture_enabled: bool,
) {
    if !(color_enabled || texture_enabled) {
        return;
    }

    let _color_guard = if texture_enabled && !color_enabled {
        Some(ColorGuard::hide(&mut model.meshes_mut()[0]))
    } else {
        None
    };

    let texture_id = if texture_enabled { texture.id } else { 0 };
    let _texture_guard = TextureGuard::set_texture(model, texture_id);

    rl3d.draw_model_ex(model, position, Y_AXIS, rotation_deg, scale, Color::WHITE);
}

pub fn draw_model_wires_and_points_at(
    rl3d: &mut RaylibMode3D<RaylibDrawHandle>,
    model: &mut Model,
    position: Vector3,
    rotation_deg: f32,
    scale: Vector3,
    wire_color: Color,
    point_color: Color,
) {
    let _color_guard = ColorGuard::hide(&mut model.meshes_mut()[0]);
    let _texture_guard = TextureGuard::hide(model);

    rl3d.draw_model_wires_ex(&model, position, Y_AXIS, rotation_deg, scale, wire_color);

    unsafe { ffi::rlSetPointSize(4.0) }
    rl3d.draw_model_points_ex(&model, position, Y_AXIS, rotation_deg, scale, point_color);
}

pub fn draw_placed_cells(
    rl3d: &mut RaylibMode3D<RaylibDrawHandle>,
    ndc_models: &mut [Model],
    mesh_textures: &[Texture2D],
    placed_cells: &mut [PlacedCell],
    total_time: f32,
) {
    for placed_cell in placed_cells.iter_mut() {
        let cell_pos = cell_center(placed_cell.ix, placed_cell.iy, placed_cell.iz);
        let age = total_time - placed_cell.placed_time;
        let scale_progress = (age / PLACEMENT_ANIM_DUR_SECONDS).clamp(0.0, 1.0);
        let current_scale = lerp(HINT_SCALE, 1.0, scale_progress);

        if scale_progress >= 1.0 {
            placed_cell.settled = true;
        }

        let placed_model = &mut ndc_models[placed_cell.mesh_index];

        if placed_cell.settled {
            draw_model_filled_at(
                rl3d,
                placed_model,
                &mesh_textures[placed_cell.mesh_index],
                cell_pos,
                0.0,
                MODEL_SCALE,
                placed_cell.color_enabled,
                placed_cell.texture_enabled,
            );

            draw_model_wires_and_points_at(rl3d, placed_model, cell_pos, 0.0, MODEL_SCALE, MARINER, LILAC);
        } else {
            draw_model_wires_and_points_at(
                rl3d,
                placed_model,
                cell_pos,
                0.0,
                Vector3::new(current_scale, current_scale, current_scale),
                ANAKIWA,
                ANAKIWA,
            );
        }
    }
}

pub fn draw_hint_mesh(
    rl3d: &mut RaylibMode3D<RaylibDrawHandle>,
    model: &mut Model,
    center: Vector3,
    mesh_rotation: f32,
    occupied: bool,
) {
    let _color_guard = ColorGuard::hide(&mut model.meshes_mut()[0]);
    let _texture_guard = TextureGuard::hide(model);

    let hint_color = if occupied { RED_DAMASK } else { ANAKIWA };

    unsafe { ffi::rlDisableDepthTest() };

    rl3d.draw_model_wires_ex(
        model,
        center,
        Y_AXIS,
        mesh_rotation.to_degrees(),
        HINT_SCALE_VEC,
        hint_color,
    );

    unsafe { ffi::rlEnableDepthTest() };
}

pub fn draw_spatial_frame(rl3d: &mut RaylibMode3D<RaylibDrawHandle>, spatial_frame: &WeakMesh) {
    const FRONT_FACES: [[usize; 2]; 4] = [[0, 1], [1, 2], [2, 3], [3, 0]];
    const BACK_FACES: [[usize; 2]; 4] = [[4, 5], [5, 6], [6, 7], [7, 4]];
    const RIB_FACES: [[usize; 2]; 4] = [[0, 4], [1, 7], [2, 6], [3, 5]];
    const FACES: [[[usize; 2]; 4]; 3] = [FRONT_FACES, BACK_FACES, RIB_FACES];

    for (i, face) in FACES.iter().enumerate() {
        for [start_pos, end_pos] in *face {
            rl3d.draw_line3D(
                spatial_frame.vertices()[start_pos],
                spatial_frame.vertices()[end_pos],
                if i == 0 {
                    NEON_CARROT
                } else if i == 1 {
                    EGGPLANT
                } else {
                    HOPBUSH
                },
            );
        }
    }
}

pub fn draw_room_floor_grid(rl3d: &mut RaylibMode3D<RaylibDrawHandle>) {
    let origin = room_origin();
    let floor_y = origin.y;

    for x in 0..=ROOM_W {
        let x_world = origin.x + x as f32;
        let start = Vector3::new(x_world, floor_y, origin.z);
        let end = Vector3::new(x_world, floor_y, origin.z + ROOM_D as f32);
        rl3d.draw_line3D(start, end, HOPBUSH);
    }

    for z in 0..=ROOM_D {
        let z_world = origin.z + z as f32;
        let start = Vector3::new(origin.x, floor_y, z_world);
        let end = Vector3::new(origin.x + ROOM_W as f32, floor_y, z_world);
        rl3d.draw_line3D(start, end, HOPBUSH);
    }
}

fn compute_hud_layout(draw_handle: &RaylibDrawHandle, font: &WeakFont) -> HudLayout {
    let screen_width = draw_handle.get_screen_width();
    let screen_height = draw_handle.get_screen_height();

    let font_size_main = FONT_SIZE;
    let font_size_debug = (FONT_SIZE as f32 * 0.5).round() as i32;

    let line_height_main = HUD_LINE_HEIGHT;
    let line_height_debug = font_size_debug;

    let margin = HUD_MARGIN;

    let left_labels = ["JUGEMU [ P ]:", "FOVY[ + - ]:", "ZOOM [ W S ]:"];
    let mut max_left_label_width = 0;
    for label in &left_labels {
        let w = font.measure_text(label, font_size_main as f32, HUD_CHAR_SPACING).x as i32;
        max_left_label_width = max_left_label_width.max(w);
    }

    let col_gap_px = (font_size_main as f32 * 0.75).round() as i32;

    let left_label_x = margin;
    let left_value_x = left_label_x + max_left_label_width + col_gap_px;

    let right_labels = ["TXTR [ T ]:", "CLR [ C ]:"];
    let mut max_right_label_width = 0;
    for label in &right_labels {
        let w = font.measure_text(label, font_size_main as f32, HUD_CHAR_SPACING).x as i32;
        max_right_label_width = max_right_label_width.max(w);
    }

    let possible_values = ["ORTHOGRAPHIC", "PERSPECTIVE", "WORLD", "NDC", "ON", "OFF"];
    let mut max_value_width = 0;
    for value in &possible_values {
        let w = font.measure_text(value, font_size_main as f32, HUD_CHAR_SPACING).x as i32;
        max_value_width = max_value_width.max(w);
    }

    let right_margin = margin;
    let right_value_x = screen_width - right_margin - max_value_width;

    let right_label_gap_px = (font_size_main as f32 * 0.5).round() as i32;
    let right_label_x = right_value_x - right_label_gap_px - max_right_label_width;

    let bottom_rows = 3;
    let bottom_block_start_y = screen_height - margin - line_height_main * bottom_rows;

    let perf_x = margin;
    let perf_y = (screen_height as f32 * (200.0 / 720.0)).round() as i32;

    let debug_padding = 4;

    HudLayout {
        font_size_main,
        font_size_debug,
        line_height_main,
        line_height_debug,
        margin,
        left_label_x,
        left_value_x,
        right_label_x,
        right_value_x,
        right_value_max_width: max_value_width,
        bottom_block_start_y,
        perf_x,
        perf_y,
        debug_padding,
    }
}

pub fn draw_hud(
    draw_handle: &mut RaylibDrawHandle,
    font: &WeakFont,
    view_state: &ViewState,
    jugemu: &Camera3D,
    target_mesh: usize,
    hover_state: &HoverState,
    placed_cells: &[PlacedCell],
    i_time: f32,
    world_models: &[Model],
    ndc_models: &[Model],
    mesh_samples: &[Vec<Vector3>],
    frame_dynamic_metrics: &FrameDynamicMetrics,
) {
    let screen_width = draw_handle.get_screen_width();
    let screen_height = draw_handle.get_screen_height();

    let layout = compute_hud_layout(draw_handle, font);

    let mut line_y = layout.margin;

    hud_text(
        draw_handle,
        font,
        "JUGEMU [ P ]:",
        layout.left_label_x,
        line_y,
        layout.font_size_main,
        SUNFLOWER,
    );
    hud_text(
        draw_handle,
        font,
        if view_state.jugemu_ortho_mode {
            "ORTHOGRAPHIC"
        } else {
            "PERSPECTIVE"
        },
        layout.left_value_x,
        line_y,
        layout.font_size_main,
        if view_state.jugemu_ortho_mode {
            BAHAMA_BLUE
        } else {
            ANAKIWA
        },
    );
    line_y += layout.line_height_main;

    hud_text(
        draw_handle,
        font,
        "FOVY[ + - ]:",
        layout.left_label_x,
        line_y,
        layout.font_size_main,
        SUNFLOWER,
    );
    hud_text(
        draw_handle,
        font,
        &format!("{:.2}", jugemu.fovy),
        layout.left_value_x,
        line_y,
        layout.font_size_main,
        LILAC,
    );
    line_y += layout.line_height_main;

    hud_text(
        draw_handle,
        font,
        "ZOOM [ W S ]:",
        layout.left_label_x,
        line_y,
        layout.font_size_main,
        SUNFLOWER,
    );
    let jugemu_distance = camera_distance(jugemu);
    hud_text(
        draw_handle,
        font,
        &format!("{:.2}", jugemu_distance),
        layout.left_value_x,
        line_y,
        layout.font_size_main,
        HOPBUSH,
    );

    let mut right_line_y = layout.margin;
    let right_margin = layout.margin;
    let gap_px = (layout.font_size_main as f32 * 0.75).round() as i32;

    let txtr_label = "TXTR [ T ]:";
    let txtr_value = if view_state.texture_mode { "ON" } else { "OFF" };

    let txtr_label_w = font
        .measure_text(txtr_label, layout.font_size_main as f32, HUD_CHAR_SPACING)
        .x as i32;
    let txtr_value_w = font
        .measure_text(txtr_value, layout.font_size_main as f32, HUD_CHAR_SPACING)
        .x as i32;

    let txtr_value_x = screen_width - right_margin - txtr_value_w;
    let txtr_label_x = txtr_value_x - gap_px - txtr_label_w;

    hud_text(
        draw_handle,
        font,
        txtr_label,
        txtr_label_x,
        right_line_y,
        layout.font_size_main,
        SUNFLOWER,
    );
    hud_text(
        draw_handle,
        font,
        txtr_value,
        txtr_value_x,
        right_line_y,
        layout.font_size_main,
        if view_state.texture_mode {
            ANAKIWA
        } else {
            CHESTNUT_ROSE
        },
    );
    right_line_y += layout.line_height_main;

    let clr_label = "CLR [ C ]:";
    let clr_value = if view_state.color_mode { "ON" } else { "OFF" };

    let clr_label_w = font
        .measure_text(clr_label, layout.font_size_main as f32, HUD_CHAR_SPACING)
        .x as i32;
    let clr_value_w = font
        .measure_text(clr_value, layout.font_size_main as f32, HUD_CHAR_SPACING)
        .x as i32;

    let clr_value_x = screen_width - right_margin - clr_value_w;
    let clr_label_x = clr_value_x - gap_px - clr_label_w;

    hud_text(
        draw_handle,
        font,
        clr_label,
        clr_label_x,
        right_line_y,
        layout.font_size_main,
        SUNFLOWER,
    );
    hud_text(
        draw_handle,
        font,
        clr_value,
        clr_value_x,
        right_line_y,
        layout.font_size_main,
        if view_state.color_mode { ANAKIWA } else { CHESTNUT_ROSE },
    );

    let mut bottom_line_y = layout.bottom_block_start_y;

    hud_text(
        draw_handle,
        font,
        "ASPECT [ Q ]:",
        layout.left_label_x,
        bottom_line_y,
        layout.font_size_main,
        SUNFLOWER,
    );
    hud_text(
        draw_handle,
        font,
        if view_state.aspect_correct {
            "CORRECT"
        } else {
            "INCORRECT"
        },
        layout.left_value_x,
        bottom_line_y,
        layout.font_size_main,
        if view_state.aspect_correct {
            ANAKIWA
        } else {
            CHESTNUT_ROSE
        },
    );
    bottom_line_y += layout.line_height_main;

    hud_text(
        draw_handle,
        font,
        "LENS [ O ]:",
        layout.left_label_x,
        bottom_line_y,
        layout.font_size_main,
        SUNFLOWER,
    );
    hud_text(
        draw_handle,
        font,
        if view_state.ortho_mode {
            "ORTHOGRAPHIC"
        } else {
            "PERSPECTIVE"
        },
        layout.left_value_x,
        bottom_line_y,
        layout.font_size_main,
        if view_state.ortho_mode { BAHAMA_BLUE } else { ANAKIWA },
    );
    bottom_line_y += layout.line_height_main;

    hud_text(
        draw_handle,
        font,
        "SPACE [ N ]:",
        layout.left_label_x,
        bottom_line_y,
        layout.font_size_main,
        SUNFLOWER,
    );
    hud_text(
        draw_handle,
        font,
        if view_state.ndc_space { "NDC" } else { "WORLD" },
        layout.left_value_x,
        bottom_line_y,
        layout.font_size_main,
        if view_state.ndc_space { BAHAMA_BLUE } else { ANAKIWA },
    );

    draw_perf_hud(
        draw_handle,
        font,
        &layout,
        view_state,
        placed_cells,
        world_models,
        ndc_models,
        mesh_samples,
        frame_dynamic_metrics,
    );

    if let Some(cell_idx) = hover_state.placed_cell_index {
        let cell = &placed_cells[cell_idx];
        let corner_world = cell_top_right_front_corner(cell.ix, cell.iy, cell.iz, jugemu);
        let screen_pos = draw_handle.get_world_to_screen(corner_world, *jugemu);

        let anchor_x = screen_pos.x as i32;
        let anchor_y = screen_pos.y as i32;

        let mesh_name = match cell.mesh_index {
            0 => "GHOST",
            1 => "CUBE",
            2 => "SPHERE",
            _ => "UNKNOWN",
        };

        let age_seconds = i_time - cell.placed_time;
        let state_label = if cell.settled { "SETTLED" } else { "ANIM" };

        let debug_lines = [
            format!("MESH: {}", mesh_name),
            format!("GRID: ({}, {}, {})", cell.ix, cell.iy, cell.iz),
            format!("AGE: {:.2}s", age_seconds),
            format!("STATE: {}", state_label),
            format!("TXTR: {}", if cell.texture_enabled { "ON" } else { "OFF" }),
            format!("CLR: {}", if cell.color_enabled { "ON" } else { "OFF" }),
        ];

        let mut max_line_w = 0;
        for line in &debug_lines {
            let w = font
                .measure_text(line, layout.font_size_debug as f32, HUD_CHAR_SPACING)
                .x as i32;
            max_line_w = max_line_w.max(w);
        }

        let debug_width = max_line_w + layout.debug_padding * 2;
        let debug_height = layout.line_height_debug * debug_lines.len() as i32 + layout.debug_padding * 2;

        let mut rect_x = anchor_x;
        let mut rect_y = anchor_y - debug_height;

        if rect_y < 0 {
            rect_y = anchor_y;
        }
        if rect_x + debug_width > screen_width {
            rect_x = anchor_x - debug_width;
        }

        draw_handle.draw_rectangle_lines(rect_x, rect_y, debug_width, debug_height, SUNFLOWER);

        let mut text_y = rect_y + layout.debug_padding;

        for (idx, line) in debug_lines.iter().enumerate() {
            let text_x = rect_x + layout.debug_padding;
            let color = match idx {
                3 => {
                    if cell.settled {
                        ANAKIWA
                    } else {
                        NEON_CARROT
                    }
                },
                4 => {
                    if cell.texture_enabled {
                        ANAKIWA
                    } else {
                        CHESTNUT_ROSE
                    }
                },
                5 => {
                    if cell.color_enabled {
                        ANAKIWA
                    } else {
                        CHESTNUT_ROSE
                    }
                },
                _ => SUNFLOWER,
            };
            hud_text(draw_handle, font, line, text_x, text_y, layout.font_size_debug, color);
            text_y += layout.line_height_debug;
        }
    }
}

fn draw_perf_hud(
    draw_handle: &mut RaylibDrawHandle,
    font: &WeakFont,
    layout: &HudLayout,
    view_state: &ViewState,
    placed_cells: &[PlacedCell],
    world_models: &[Model],
    ndc_models: &[Model],
    mesh_samples: &[Vec<Vector3>],
    frame_dynamic_metrics: &FrameDynamicMetrics,
) {
    let screen_width = draw_handle.get_screen_width();

    let perf_x = layout.perf_x;
    let mut y = layout.perf_y;
    let line = layout.line_height_main;
    let font_sz = layout.font_size_main;

    hud_text(draw_handle, font, "LAYER 3 METRICS", perf_x, y, font_sz, NEON_CARROT);
    y += line;

    let mesh_count = ndc_models.len();
    let mut per_mesh_instance_counts = vec![0usize; mesh_count];
    for cell in placed_cells {
        if cell.mesh_index < mesh_count {
            per_mesh_instance_counts[cell.mesh_index] += 1;
        }
    }

    let mut per_mesh_world_metrics: Vec<MeshMetrics> = Vec::with_capacity(mesh_count);
    let mut per_mesh_ndc_metrics: Vec<MeshMetrics> = Vec::with_capacity(mesh_count);
    let mut per_mesh_combined_bytes: Vec<usize> = Vec::with_capacity(mesh_count);
    let mut total_geom_bytes_shared = 0usize;

    for i in 0..mesh_count {
        let world_mesh = &world_models[i].meshes()[0];
        let ndc_mesh = &ndc_models[i].meshes()[0];
        let world_metrics = MeshMetrics::measure(world_mesh);
        let ndc_metrics = MeshMetrics::measure(ndc_mesh);
        let combined_bytes = world_metrics.total_bytes + ndc_metrics.total_bytes;
        total_geom_bytes_shared += combined_bytes;

        per_mesh_world_metrics.push(world_metrics);
        per_mesh_ndc_metrics.push(ndc_metrics);
        per_mesh_combined_bytes.push(combined_bytes);
    }

    let mut filled_draws_per_mesh = vec![0usize; mesh_count];
    let mut overlay_calls_per_mesh = vec![0usize; mesh_count];

    if view_state.target_mesh_index < mesh_count {
        let i = view_state.target_mesh_index;

        if view_state.color_mode || view_state.texture_mode {
            filled_draws_per_mesh[i] += 1;
        }

        overlay_calls_per_mesh[i] += 1;
    }

    for cell in placed_cells {
        if cell.mesh_index >= mesh_count {
            continue;
        }
        let i = cell.mesh_index;

        if cell.settled && (cell.color_enabled || cell.texture_enabled) {
            filled_draws_per_mesh[i] += 1;
        }
        overlay_calls_per_mesh[i] += 1;
    }

    let active_index = view_state.target_mesh_index;
    if active_index < mesh_count {
        let active_name = match active_index {
            0 => "GHOST",
            1 => "CUBE",
            2 => "SPHERE",
            _ => "MESH",
        };

        let active_world = per_mesh_world_metrics[active_index];
        let active_ndc = per_mesh_ndc_metrics[active_index];
        let active_bytes = per_mesh_combined_bytes[active_index];
        let active_instances = per_mesh_instance_counts[active_index];

        let active_filled_draws = filled_draws_per_mesh[active_index];
        let active_overlay_calls = overlay_calls_per_mesh[active_index];

        let active_total_vertex_passes = active_filled_draws + active_overlay_calls * 2;

        let active_verts_per_draw = active_ndc.vertex_count;
        let active_tris_per_draw = active_ndc.triangle_count;
        let active_indices_per_draw = active_ndc.index_count;

        let gpu_verts_per_frame = active_verts_per_draw * active_total_vertex_passes;
        let gpu_tris_per_frame = active_tris_per_draw * active_filled_draws;

        let vertex_stride = gpu_vertex_stride_bytes(&active_ndc);
        let vertex_bytes_per_draw = vertex_stride * active_verts_per_draw;
        let index_bytes_per_draw = active_indices_per_draw * std::mem::size_of::<u16>();

        let gpu_bytes_from_tri_draws = active_filled_draws * (vertex_bytes_per_draw + index_bytes_per_draw);
        let gpu_bytes_from_overlay_draws = active_overlay_calls * 2 * vertex_bytes_per_draw;

        let gpu_bytes_per_frame = gpu_bytes_from_tri_draws + gpu_bytes_from_overlay_draws;

        let active_x = (screen_width as f32 * 0.45).round() as i32;
        let mut header_y = layout.margin;

        hud_text(
            draw_handle,
            font,
            "ACTIVE MESH:",
            active_x,
            header_y,
            font_sz,
            SUNFLOWER,
        );
        header_y += line;

        hud_text(
            draw_handle,
            font,
            &format!("{} ({} INST)", active_name, active_instances),
            active_x,
            header_y,
            font_sz,
            NEON_CARROT,
        );
        header_y += line;

        hud_text(
            draw_handle,
            font,
            &format!(
                "WORLD V/T/I: {}/{}/{}",
                active_world.vertex_count, active_world.triangle_count, active_world.index_count
            ),
            active_x,
            header_y,
            font_sz,
            ANAKIWA,
        );
        header_y += line;

        hud_text(
            draw_handle,
            font,
            &format!(
                "NDC   V/T/I: {}/{}/{}",
                active_ndc.vertex_count, active_ndc.triangle_count, active_ndc.index_count
            ),
            active_x,
            header_y,
            font_sz,
            ANAKIWA,
        );
        header_y += line;

        hud_text(
            draw_handle,
            font,
            &format!("GEOM BYTES (W+N): {}", format_bytes(active_bytes)),
            active_x,
            header_y,
            font_sz,
            LILAC,
        );
        header_y += line;

        hud_text(
            draw_handle,
            font,
            "GPU SUBMISSION (EST):",
            active_x,
            header_y,
            font_sz,
            SUNFLOWER,
        );
        header_y += line;

        hud_text(
            draw_handle,
            font,
            &format!("FILLED DRAWS: {}", active_filled_draws),
            active_x,
            header_y,
            font_sz,
            ANAKIWA,
        );
        header_y += line;

        hud_text(
            draw_handle,
            font,
            &format!("WIRES+PTS CALLS: {} (2 passes ea.)", active_overlay_calls),
            active_x,
            header_y,
            font_sz,
            ANAKIWA,
        );
        header_y += line;

        hud_text(
            draw_handle,
            font,
            &format!("GPU VERTS/FRAME: {}", gpu_verts_per_frame),
            active_x,
            header_y,
            font_sz,
            ANAKIWA,
        );
        header_y += line;

        hud_text(
            draw_handle,
            font,
            &format!("GPU TRIS/FRAME:  {}", gpu_tris_per_frame),
            active_x,
            header_y,
            font_sz,
            ANAKIWA,
        );
        header_y += line;

        hud_text(
            draw_handle,
            font,
            &format!("~GPU BYTES/FRAME: {}", format_bytes(gpu_bytes_per_frame)),
            active_x,
            header_y,
            font_sz,
            LILAC,
        );
    }

    hud_text(draw_handle, font, "STATIC MESHES:", perf_x, y, font_sz, SUNFLOWER);
    y += line;

    for i in 0..mesh_count {
        let mesh_name = match i {
            0 => "GHOST",
            1 => "CUBE",
            2 => "SPHERE",
            _ => "MESH",
        };
        let metrics_world = per_mesh_world_metrics[i];
        let metrics_ndc = per_mesh_ndc_metrics[i];
        let combined_bytes = per_mesh_combined_bytes[i];

        hud_text(
            draw_handle,
            font,
            &format!(
                "{}: {} B (W {}v, N {}v)",
                mesh_name, combined_bytes, metrics_world.vertex_count, metrics_ndc.vertex_count
            ),
            perf_x,
            y,
            font_sz,
            ANAKIWA,
        );
        y += line;
    }

    hud_text(
        draw_handle,
        font,
        &format!("GEOM MEM (W+N): {}", format_bytes(total_geom_bytes_shared)),
        perf_x,
        y,
        font_sz,
        LILAC,
    );
    y += line;
    y += line;

    if let Some(anim_metrics) = AnimationMetrics::measure(mesh_samples) {
        hud_text(
            draw_handle,
            font,
            &format!("ANIM SAMPLES: {}", anim_metrics.sample_count),
            perf_x,
            y,
            font_sz,
            ANAKIWA,
        );
        y += line;

        hud_text(
            draw_handle,
            font,
            &format!("VERTS/SAMPLE: {}", anim_metrics.verts_per_sample),
            perf_x,
            y,
            font_sz,
            ANAKIWA,
        );
        y += line;

        hud_text(
            draw_handle,
            font,
            &format!("ANIM MEM: {}", format_bytes(anim_metrics.total_bytes)),
            perf_x,
            y,
            font_sz,
            LILAC,
        );
        y += line;

        let total_layer3_bytes = total_geom_bytes_shared + anim_metrics.total_bytes;
        hud_text(
            draw_handle,
            font,
            &format!("LAYER3 MEM: {}", format_bytes(total_layer3_bytes)),
            perf_x,
            y,
            font_sz,
            LILAC,
        );
        y += line;
        y += line;
    }

    hud_text(
        draw_handle,
        font,
        "DYNAMIC WRITES/FRAME:",
        perf_x,
        y,
        font_sz,
        SUNFLOWER,
    );
    y += line;

    hud_text(
        draw_handle,
        font,
        &format!("POS: {}", frame_dynamic_metrics.vertex_positions_written),
        perf_x,
        y,
        font_sz,
        ANAKIWA,
    );
    y += line;

    hud_text(
        draw_handle,
        font,
        &format!("NRM: {}", frame_dynamic_metrics.vertex_normals_written),
        perf_x,
        y,
        font_sz,
        ANAKIWA,
    );
    y += line;

    hud_text(
        draw_handle,
        font,
        &format!("CLR: {}", frame_dynamic_metrics.vertex_colors_written),
        perf_x,
        y,
        font_sz,
        ANAKIWA,
    );
    y += line;

    hud_text(
        draw_handle,
        font,
        &format!(
            "BYTES WR: {}",
            format_bytes(frame_dynamic_metrics.total_bytes_written())
        ),
        perf_x,
        y,
        font_sz,
        LILAC,
    );
}

fn hud_text(
    draw_handle: &mut RaylibDrawHandle,
    font: &WeakFont,
    text: &str,
    x: i32,
    y: i32,
    font_size: i32,
    color: Color,
) {
    draw_handle.draw_text_ex(
        font,
        text,
        Vector2::new(x as f32, y as f32),
        font_size as f32,
        HUD_CHAR_SPACING,
        color,
    );
}

pub fn handle_view_toggles(handle: &RaylibHandle, view_state: &mut ViewState) {
    if handle.is_key_pressed(KeyboardKey::KEY_N) {
        view_state.ndc_space = !view_state.ndc_space;
    }
    if handle.is_key_pressed(KeyboardKey::KEY_Q) {
        view_state.aspect_correct = !view_state.aspect_correct;
    }
    if handle.is_key_pressed(KeyboardKey::KEY_SPACE) {
        view_state.paused = !view_state.paused;
    }
    if handle.is_key_pressed(KeyboardKey::KEY_C) {
        view_state.color_mode = !view_state.color_mode;
    }
    if handle.is_key_pressed(KeyboardKey::KEY_T) {
        view_state.texture_mode = !view_state.texture_mode;
    }
    if handle.is_key_pressed(KeyboardKey::KEY_J) {
        view_state.jugemu_mode = !view_state.jugemu_mode;
    }
    if handle.is_key_pressed(KeyboardKey::KEY_O) {
        view_state.ortho_mode = !view_state.ortho_mode;
    }
}

pub fn handle_jugemu_projection_toggle(
    handle: &RaylibHandle,
    view_state: &mut ViewState,
    jugemu: &mut Camera3D,
    prev_fovy_ortho: &mut f32,
    prev_fovy_perspective: &mut f32,
    prev_distance_ortho: &mut f32,
    prev_distance_perspective: &mut f32,
) {
    if handle.is_key_pressed(KeyboardKey::KEY_P) {
        if view_state.jugemu_ortho_mode {
            *prev_fovy_ortho = jugemu.fovy;
            *prev_distance_ortho = camera_distance(jugemu);

            jugemu.fovy = *prev_fovy_perspective;
            let dir = jugemu.position.normalize();
            jugemu.position = dir * *prev_distance_perspective;
        } else {
            *prev_fovy_perspective = jugemu.fovy;
            *prev_distance_perspective = camera_distance(jugemu);

            jugemu.fovy = *prev_fovy_ortho;
            let dir = jugemu.position.normalize();
            jugemu.position = dir * *prev_distance_ortho;
        }
        view_state.jugemu_ortho_mode = !view_state.jugemu_ortho_mode;
    }
}

pub fn camera_distance(cam: &Camera3D) -> f32 {
    let dx = cam.position.x - cam.target.x;
    let dy = cam.position.y - cam.target.y;
    let dz = cam.position.z - cam.target.z;
    (dx * dx + dy * dy + dz * dz).sqrt()
}

pub fn handle_mesh_selection(handle: &RaylibHandle, view_state: &mut ViewState) {
    if handle.is_key_pressed(KeyboardKey::KEY_ONE) {
        view_state.target_mesh_index = 0;
    }
    if handle.is_key_pressed(KeyboardKey::KEY_TWO) {
        view_state.target_mesh_index = 1;
    }
    if handle.is_key_pressed(KeyboardKey::KEY_THREE) {
        view_state.target_mesh_index = 2;
    }
}

pub fn update_ghost_mesh(
    ndc_model: &mut Model,
    world_model: &mut Model,
    i_time: f32,
    mesh_samples: &[Vec<Vector3>],
    main: &Camera3D,
    mesh_rotation: f32,
    frame_dynamic_metrics: &mut FrameDynamicMetrics,
) {
    interpolate_between_deformed_vertices(ndc_model, i_time, mesh_samples, frame_dynamic_metrics);
    interpolate_between_deformed_vertices(world_model, i_time, mesh_samples, frame_dynamic_metrics);

    update_normals_for_silhouette(&mut ndc_model.meshes_mut()[0], frame_dynamic_metrics);
    update_normals_for_silhouette(&mut world_model.meshes_mut()[0], frame_dynamic_metrics);

    fade_vertex_colors_silhouette_rim(
        &mut ndc_model.meshes_mut()[0],
        main,
        mesh_rotation,
        frame_dynamic_metrics,
    );
    fade_vertex_colors_silhouette_rim(
        &mut world_model.meshes_mut()[0],
        main,
        mesh_rotation,
        frame_dynamic_metrics,
    );
}

pub fn fill_planar_texcoords(mesh: &mut WeakMesh) {
    if mesh.texcoords().is_none() {
        let vertices = mesh.vertices();
        let bounds = mesh.get_mesh_bounding_box();
        let extents = Vector3::new(
            bounds.max.x - bounds.min.x,
            bounds.max.y - bounds.min.y,
            bounds.max.z - bounds.min.z,
        );
        let mut planar_texcoords = vec![Vector2::default(); vertices.len()];
        for [a, b, c] in mesh.triangles() {
            for &j in [a, b, c].iter() {
                planar_texcoords[j].x = (vertices[j].x - bounds.min.x) / extents.x;
                planar_texcoords[j].y = (vertices[j].y - bounds.min.y) / extents.y;
            }
        }
        mesh.init_texcoords_mut().unwrap().copy_from_slice(&planar_texcoords);
    }
}

pub fn fill_vertex_colors(mesh: &mut WeakMesh) {
    let bounds = mesh.get_mesh_bounding_box();
    let vertices = mesh.vertices();
    let mut colors = vec![Color::WHITE; vertices.len()];

    for [a, b, c] in mesh.triangles() {
        for &i in [a, b, c].iter() {
            let vertex = vertices[i];
            let nx = (vertex.x - 0.5 * (bounds.min.x + bounds.max.x)) / (0.5 * (bounds.max.x - bounds.min.x));
            let ny = (vertex.y - 0.5 * (bounds.min.y + bounds.max.y)) / (0.5 * (bounds.max.y - bounds.min.y));
            let nz = (vertex.z - 0.5 * (bounds.min.z + bounds.max.z)) / (0.5 * (bounds.max.z - bounds.min.z));
            let len = (nx * nx + ny * ny + nz * nz).sqrt();
            colors[i] = Color::new(
                (127.5 * (nx / len + 1.0)).round() as u8,
                (127.5 * (ny / len + 1.0)).round() as u8,
                (127.5 * (nz / len + 1.0)).round() as u8,
                255,
            );
        }
    }
    mesh.init_colors_mut().unwrap().copy_from_slice(&colors);
}

fn update_normals_for_silhouette(mesh: &mut WeakMesh, frame_dynamic_metrics: &mut FrameDynamicMetrics) {
    let vertices = mesh.vertices();
    let mut normals = vec![Vector3::ZERO; vertices.len()];

    for [a, b, c] in mesh.triangles() {
        let va = vertices[a];
        let vb = vertices[b];
        let vc = vertices[c];
        let face_normal = triangle_normal(va, vb, vc);
        normals[a] += face_normal;
        normals[b] += face_normal;
        normals[c] += face_normal;
    }

    for i in mesh.triangles().iter_vertices() {
        normals[i] = normals[i].normalize_or_zero();
    }

    mesh.normals_mut().unwrap().copy_from_slice(&normals);
    frame_dynamic_metrics.vertex_normals_written += normals.len();
}

fn fade_vertex_colors_silhouette_rim(
    mesh: &mut WeakMesh,
    observer: &Camera3D,
    mesh_rotation: f32,
    frame_dynamic_metrics: &mut FrameDynamicMetrics,
) {
    let model_center_to_camera = rotate_point_about_axis(
        -1.0 * observed_line_of_sight(observer),
        (Vector3::ZERO, Vector3::Y),
        -mesh_rotation,
    )
    .normalize_or_zero();
    const OUTER_FADE_ANGLE: f32 = 70.0_f32.to_radians();
    let cos_fade_angle: f32 = OUTER_FADE_ANGLE.cos();
    let vertices = mesh.vertices();
    let mut alpha_buffer = vec![0u8; vertices.len()];
    for i in mesh.triangles().iter_vertices() {
        let model_center_to_vertex = vertices[i].normalize_or_zero();
        let cos_theta = model_center_to_vertex.dot(model_center_to_camera);
        if cos_theta <= 0.0 {
            alpha_buffer[i] = 0;
            continue;
        }
        let fade_scalar = (cos_theta / cos_fade_angle).clamp(0.0, 1.0);
        let alpha = fade_scalar * fade_scalar * fade_scalar * fade_scalar;
        alpha_buffer[i] = (alpha * 255.0).round() as u8;
    }

    let colors = mesh.colors_mut().unwrap();
    for i in 0..alpha_buffer.len() {
        colors[i].a = alpha_buffer[i];
    }
    frame_dynamic_metrics.vertex_colors_written += alpha_buffer.len();
}
