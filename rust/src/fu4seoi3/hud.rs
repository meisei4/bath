use crate::fu4seoi3::config_and_state::*;
use crate::fu4seoi3::core::*;
use raylib::prelude::*;

pub const HUD_MARGIN: i32 = 12;
pub const HUD_LINE_HEIGHT: i32 = 22;
pub const FONT_SIZE: i32 = 20;
pub const HUD_CHAR_SPACING: f32 = 2.0;
const EDIT_STACK_VISIBLE_ROWS: usize = 7;
const EDIT_STACK_CURSOR_ROW: usize = EDIT_STACK_VISIBLE_ROWS / 2;

pub struct HudLayout {
    pub font_size_main: i32,
    pub font_size_debug: i32,
    pub line_height_main: i32,
    pub line_height_debug: i32,
    pub margin: i32,
    pub left_label_x: i32,
    pub left_value_x: i32,
    pub right_label_x: i32,
    pub right_value_x: i32,
    pub right_value_max_width: i32,
    pub bottom_block_start_y: i32,
    pub perf_x: i32,
    pub perf_y: i32,
    pub debug_padding: i32,
}

fn compute_hud_layout(draw_handle: &RaylibDrawHandle, font: &WeakFont) -> HudLayout {
    let screen_width = draw_handle.get_screen_width();
    let screen_height = draw_handle.get_screen_height();
    let font_size_main = FONT_SIZE;
    let font_size_debug = (FONT_SIZE as f32 * 0.5).round() as i32;
    let line_height_main = HUD_LINE_HEIGHT;
    let line_height_debug = (font_size_debug as f32 * 1.2).round() as i32;
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
    let right_values = ["ON", "OFF"];
    let mut max_right_value_width = 0;
    for value in &right_values {
        let width = font.measure_text(value, font_size_main as f32, HUD_CHAR_SPACING).x as i32;
        max_right_value_width = max_right_value_width.max(width);
    }

    let right_margin = margin;
    let right_value_x = screen_width - right_margin - max_right_value_width;
    let right_label_gap_px = (font_size_main as f32 * 0.5).round() as i32;
    let right_label_x = right_value_x - right_label_gap_px - max_right_label_width;
    let bottom_rows = 3;
    let bottom_block_start_y = screen_height - margin - line_height_main * bottom_rows;
    let top_rows = 3;
    let perf_x = margin;
    let perf_y = margin + line_height_main * top_rows + margin;
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
        right_value_max_width: max_right_value_width,
        bottom_block_start_y,
        perf_x,
        perf_y,
        debug_padding,
    }
}

fn hud_row(
    draw: &mut RaylibDrawHandle,
    font: &WeakFont,
    label: &str,
    value: &str,
    x_label: i32,
    x_value: i32,
    y: i32,
    size: i32,
    line_height: i32,
    color_label: Color,
    color_value: Color,
) -> i32 {
    hud_text(draw, font, label, x_label, y, size, color_label);
    hud_text(draw, font, value, x_value, y, size, color_value);
    //TODO: this returning something is stupid, fix it
    y + line_height
}

fn draw_debug_box(
    draw: &mut RaylibDrawHandle,
    font: &WeakFont,
    anchor_x: i32,
    anchor_y: i32,
    lines: &[(String, Color)],
    layout: &HudLayout,
) {
    if lines.is_empty() {
        return;
    }

    let screen_width = draw.get_screen_width();
    let screen_height = draw.get_screen_height();
    let mut max_line_w = 0;

    for (text, _) in lines {
        let w = font
            .measure_text(text, layout.font_size_debug as f32, HUD_CHAR_SPACING)
            .x as i32;
        max_line_w = max_line_w.max(w);
    }

    let debug_width = max_line_w + layout.debug_padding * 2;
    let debug_height = layout.line_height_debug * lines.len() as i32 + layout.debug_padding * 2;
    let mut rect_x = anchor_x;
    let mut rect_y = anchor_y - debug_height;

    if rect_y < 0 {
        rect_y = anchor_y;
    }

    if rect_x + debug_width > screen_width {
        rect_x = anchor_x - debug_width;
    }
    if rect_x < 0 {
        rect_x = 0;
    }

    if rect_y + debug_height > screen_height {
        rect_y = screen_height - debug_height;
    }

    draw.draw_rectangle_lines(rect_x, rect_y, debug_width, debug_height, SUNFLOWER);

    let mut text_y = rect_y + layout.debug_padding;
    for (text, color) in lines {
        let text_x = rect_x + layout.debug_padding;
        hud_text(draw, font, text, text_x, text_y, layout.font_size_debug, *color);
        text_y += layout.line_height_debug;
    }
}

fn mesh_name(i: usize, meshes: &[MeshDescriptor]) -> &'static str {
    meshes.get(i).map(|m| m.name).unwrap_or("MESH")
}

//TODO: all the hud draw functions result in massive carraige returned parameter style blocks, i hate that
pub fn draw_hud(
    draw_handle: &mut RaylibDrawHandle,
    font: &WeakFont,
    view_state: &ViewState,
    jugemu: &Camera3D,
    _target_mesh: usize,
    hover_state: &HoverState,
    placed_cells: &[PlacedCell],
    i_time: f32,
    meshes: &[MeshDescriptor],
    mesh_samples: &[Vec<Vector3>],
    frame_dynamic_metrics: &FrameDynamicMetrics,
    room: &Room,
    edit_stack: &[EditStack],
    edit_cursor: usize,
    opening_metrics: &[MeshMetrics],
) {
    let layout = compute_hud_layout(draw_handle, font);
    let mut line_y = layout.margin;

    line_y = hud_row(
        draw_handle,
        font,
        "JUGEMU [ P ]:",
        if view_state.jugemu_ortho_mode { "ORTHO" } else { "PERSP" },
        layout.left_label_x,
        layout.left_value_x,
        line_y,
        layout.font_size_main,
        layout.line_height_main,
        SUNFLOWER,
        if view_state.jugemu_ortho_mode {
            BAHAMA_BLUE
        } else {
            ANAKIWA
        },
    );

    line_y = hud_row(
        draw_handle,
        font,
        "FOVY[ + - ]:",
        &format!("{:.2}", jugemu.fovy),
        layout.left_label_x,
        layout.left_value_x,
        line_y,
        layout.font_size_main,
        layout.line_height_main,
        SUNFLOWER,
        LILAC,
    );

    let jugemu_distance = camera_distance(jugemu);
    hud_row(
        draw_handle,
        font,
        "ZOOM [ W S ]:",
        &format!("{:.2}", jugemu_distance),
        layout.left_label_x,
        layout.left_value_x,
        line_y,
        layout.font_size_main,
        layout.line_height_main,
        SUNFLOWER,
        HOPBUSH,
    );

    let mut right_y = layout.margin;

    let txtr_value = if view_state.texture_mode { "ON" } else { "OFF" };
    right_y = hud_row(
        draw_handle,
        font,
        "TXTR [ T ]:",
        txtr_value,
        layout.right_label_x,
        layout.right_value_x,
        right_y,
        layout.font_size_main,
        layout.line_height_main,
        SUNFLOWER,
        if view_state.texture_mode {
            ANAKIWA
        } else {
            CHESTNUT_ROSE
        },
    );

    let clr_value = if view_state.color_mode { "ON" } else { "OFF" };
    hud_row(
        draw_handle,
        font,
        "CLR [ C ]:",
        clr_value,
        layout.right_label_x,
        layout.right_value_x,
        right_y,
        layout.font_size_main,
        layout.line_height_main,
        SUNFLOWER,
        if view_state.color_mode { ANAKIWA } else { CHESTNUT_ROSE },
    );

    let mut bottom_y = layout.bottom_block_start_y;

    bottom_y = hud_row(
        draw_handle,
        font,
        "ASPECT [ Q ]:",
        if view_state.aspect_correct { "Ｘ" } else { "Ｏ" },
        layout.left_label_x,
        layout.left_value_x,
        bottom_y,
        layout.font_size_main,
        layout.line_height_main,
        SUNFLOWER,
        if view_state.aspect_correct {
            ANAKIWA
        } else {
            CHESTNUT_ROSE
        },
    );

    bottom_y = hud_row(
        draw_handle,
        font,
        "LENS [ O ]:",
        if view_state.ortho_mode { "ORTHO" } else { "PERSP" },
        layout.left_label_x,
        layout.left_value_x,
        bottom_y,
        layout.font_size_main,
        layout.line_height_main,
        SUNFLOWER,
        if view_state.ortho_mode { BAHAMA_BLUE } else { ANAKIWA },
    );

    hud_row(
        draw_handle,
        font,
        "SPACE [ N ]:",
        if view_state.ndc_space { "NDC" } else { "WRLD" },
        layout.left_label_x,
        layout.left_value_x,
        bottom_y,
        layout.font_size_main,
        layout.line_height_main,
        SUNFLOWER,
        if view_state.ndc_space { BAHAMA_BLUE } else { ANAKIWA },
    );

    draw_perf_hud(
        draw_handle,
        font,
        &layout,
        view_state,
        placed_cells,
        meshes,
        mesh_samples,
        frame_dynamic_metrics,
        opening_metrics,
    );

    if let (Some(cell_index), Some((ix, iy, iz))) = (hover_state.placed_cell_index, hover_state.indices) {
        if cell_index < placed_cells.len() {
            let cell = &placed_cells[cell_index];
            let corner_world = room.top_right_front_corner(ix, iy, iz, jugemu);
            let screen_pos = draw_handle.get_world_to_screen(corner_world, *jugemu);
            let anchor_x = screen_pos.x as i32;
            let anchor_y = screen_pos.y as i32;
            let mesh_label = mesh_name(cell.mesh_index, meshes);
            let age_seconds = i_time - cell.placed_time;
            let age_clock = format_time_clock(age_seconds);
            let state_label = if cell.settled { "SETTLED" } else { "ANIM" };
            let mut lines: Vec<(String, Color)> = Vec::new();
            lines.push((format!("MESH: {}", mesh_label), SUNFLOWER));
            lines.push((format!("GRID: ({}, {}, {})", cell.ix, cell.iy, cell.iz), SUNFLOWER));
            lines.push((format!("AGE: {}", age_clock), SUNFLOWER));

            lines.push((
                format!("STATE: {}", state_label),
                if cell.settled { ANAKIWA } else { NEON_CARROT },
            ));
            lines.push((
                format!("TXTR: {}", if cell.texture_enabled { "ON" } else { "OFF" }),
                if cell.texture_enabled { ANAKIWA } else { CHESTNUT_ROSE },
            ));
            lines.push((
                format!("CLR: {}", if cell.color_enabled { "ON" } else { "OFF" }),
                if cell.color_enabled { ANAKIWA } else { CHESTNUT_ROSE },
            ));

            draw_debug_box(draw_handle, font, anchor_x, anchor_y, &lines, &layout);
        }
    }
    draw_edit_stack_hud(draw_handle, font, &layout, edit_stack, edit_cursor);

    let fps_text = format!("FPS: {}", draw_handle.get_fps());
    let fps_x = draw_handle.get_screen_width()
        - layout.margin
        - font
            .measure_text(&fps_text, layout.font_size_main as f32, HUD_CHAR_SPACING)
            .x as i32;
    let fps_y = draw_handle.get_screen_height() - layout.margin - layout.line_height_main;
    hud_text(
        draw_handle,
        font,
        &fps_text,
        fps_x,
        fps_y,
        layout.font_size_main,
        SUNFLOWER,
    );
}

fn draw_perf_hud(
    draw_handle: &mut RaylibDrawHandle,
    font: &WeakFont,
    layout: &HudLayout,
    view_state: &ViewState,
    placed_cells: &[PlacedCell],
    meshes: &[MeshDescriptor],
    mesh_samples: &[Vec<Vector3>],
    frame_dynamic_metrics: &FrameDynamicMetrics,
    opening_metrics: &[MeshMetrics],
) {
    let screen_width = draw_handle.get_screen_width();

    let perf_x = layout.perf_x;
    let mut y = layout.perf_y;
    let line = layout.line_height_main;
    let font_sz = layout.font_size_main;

    hud_text(draw_handle, font, "LAYER 3 METRICS", perf_x, y, font_sz, NEON_CARROT);
    y += line;

    let mesh_count = meshes.len();
    let mut per_mesh_instance_counts = vec![0usize; mesh_count];
    for cell in placed_cells {
        if cell.mesh_index < mesh_count {
            per_mesh_instance_counts[cell.mesh_index] += 1;
        }
    }
    let total_geom_bytes_meshes: usize = meshes.iter().map(|m| m.combined_bytes).sum();
    let total_geom_bytes_openings: usize = opening_metrics.iter().map(|m| m.total_bytes).sum();
    let total_geom_bytes_shared: usize = total_geom_bytes_meshes + total_geom_bytes_openings;
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

        if cell.is_filled() {
            filled_draws_per_mesh[i] += 1;
        }
        overlay_calls_per_mesh[i] += 1;
    }

    let active_index = view_state.target_mesh_index;
    if active_index < mesh_count {
        let desc = &meshes[active_index];

        let active_name = mesh_name(active_index, meshes);
        let active_world = desc.metrics_world;
        let active_ndc = desc.metrics_ndc;
        let active_bytes = desc.combined_bytes;
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
        let index_bytes_per_draw = active_indices_per_draw * size_of::<u16>();
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
                "WRLD XYZ/ST/IDX: {}/{}/{}",
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
                "NDC   XYZ/ST/IDX: {}/{}/{}",
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
            &format!("GEOM BYTES (WRLD+NDC): {}", format_bytes(active_bytes)),
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
            &format!("WIRES+PTS DRAWS: {} (2x)", active_overlay_calls),
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

    for (i, desc) in meshes.iter().enumerate() {
        let name = mesh_name(i, meshes);
        let world = desc.metrics_world;
        let ndc = desc.metrics_ndc;
        let combined_bytes = desc.combined_bytes;

        hud_text(
            draw_handle,
            font,
            &format!(
                "{}: {} (WRLD {}v, NDC {}v)",
                name,
                format_bytes(combined_bytes),
                world.vertex_count,
                ndc.vertex_count
            ),
            perf_x,
            y,
            font_sz,
            ANAKIWA,
        );
        y += line;
    }
    if !opening_metrics.is_empty() {
        y += line;
        hud_text(draw_handle, font, "OPENING MESHES:", perf_x, y, font_sz, SUNFLOWER);
        y += line;

        for (i, m) in opening_metrics.iter().enumerate() {
            hud_text(
                draw_handle,
                font,
                &format!("OPENING_{}: {} ({}v)", i, format_bytes(m.total_bytes), m.vertex_count),
                perf_x,
                y,
                font_sz,
                ANAKIWA,
            );
            y += line;
        }
    }

    hud_text(
        draw_handle,
        font,
        &format!("GEOM MEM (WRLD+NDC): {}", format_bytes(total_geom_bytes_shared)),
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
            "BYTES RW: {}",
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
fn draw_edit_stack_hud(
    draw: &mut RaylibDrawHandle,
    font: &WeakFont,
    layout: &HudLayout,
    edit_stack: &[EditStack],
    edit_cursor: usize,
) {
    let screen_w = draw.get_screen_width();
    let screen_h = draw.get_screen_height();
    let line_h = layout.line_height_main;
    let font_sz = layout.font_size_main;
    let margin = layout.margin;

    let visible_rows = EDIT_STACK_VISIBLE_ROWS;
    let cursor_row = EDIT_STACK_CURSOR_ROW.min(visible_rows - 1);

    let panel_h = (visible_rows as i32) * line_h;
    let foot_rows = 1;
    let top_y = screen_h - margin - panel_h - foot_rows * line_h;

    let mut max_text_w = layout.right_value_max_width;
    let mut formatted: Vec<String> = Vec::with_capacity(edit_stack.len());

    for edit in edit_stack {
        let (name, ix, iy, iz, time) = match edit {
            EditStack::PlaceCell { cell, time } => ("PLC", cell.ix, cell.iy, cell.iz, *time),
            EditStack::RemoveCell { cell, time } => ("RM", cell.ix, cell.iy, cell.iz, *time),
            EditStack::ToggleTexture { ix, iy, iz, time } => ("TXTR", *ix, *iy, *iz, *time),
            EditStack::ToggleColor { ix, iy, iz, time } => ("CLR", *ix, *iy, *iz, *time),
        };

        let time_str = format_time_clock(time);
        let text = format!("{} {} ({:2},{:2},{:2})", time_str, name, ix, iy, iz);
        let w = font.measure_text(&text, font_sz as f32, HUD_CHAR_SPACING).x as i32;

        max_text_w = max_text_w.max(w);
        formatted.push(text);
    }

    let sample_time = "99:59:999";
    let sample_text = format!("{} {} ({:2},{:2},{:2})", sample_time, "TXTR", 99, 99, 99);
    let sample_w = font.measure_text(&sample_text, font_sz as f32, HUD_CHAR_SPACING).x as i32;
    max_text_w = max_text_w.max(sample_w);

    let base_x = screen_w - margin - max_text_w;
    let cursor_dx = (font_sz as f32 * 0.6).round() as i32;
    let indent_step = (font_sz as f32 * 0.5).round() as i32;
    let max_dist = cursor_row.max(visible_rows - 1 - cursor_row) as i32;

    if edit_stack.is_empty() {
        let cursor_indent_left = max_dist * indent_step;
        let cursor_y = top_y + (cursor_row as i32) * line_h;
        let cursor_x = base_x - cursor_indent_left - cursor_dx;
        hud_text(draw, font, ">", cursor_x, cursor_y, font_sz, SUNFLOWER);
        return;
    }

    let len = edit_stack.len();
    let applied_count = edit_cursor.min(len);
    let mut row_to_index: [Option<usize>; EDIT_STACK_VISIBLE_ROWS] = [None; EDIT_STACK_VISIBLE_ROWS];
    let mut has_above = false;
    let mut has_below = false;

    if applied_count > 0 {
        let ptr_index = applied_count - 1;
        row_to_index[cursor_row] = Some(ptr_index);
        let mut row = cursor_row + 1;
        let mut index = ptr_index as isize - 1;
        while row < visible_rows && index >= 0 {
            row_to_index[row] = Some(index as usize);
            row += 1;
            index -= 1;
        }
        if index >= 0 {
            has_below = true;
        }
        let mut row = cursor_row;
        let mut i = applied_count;
        while row > 0 && i < len {
            row -= 1;
            row_to_index[row] = Some(i);
            i += 1;
        }
        if i < len {
            has_above = true;
        }
    } else {
        let mut i = 0;
        row_to_index[cursor_row] = Some(i);
        i += 1;

        let mut row = cursor_row;
        while row > 0 && i < len {
            row -= 1;
            row_to_index[row] = Some(i);
            i += 1;
        }
        if i < len {
            has_above = true;
        }
    }
    let cursor_indent_left = max_dist * indent_step;
    let cursor_y = top_y + (cursor_row as i32) * line_h;
    let cursor_x = base_x - cursor_indent_left - cursor_dx;
    hud_text(draw, font, ">", cursor_x, cursor_y, font_sz, SUNFLOWER);
    for row in 0..visible_rows {
        if let Some(i) = row_to_index[row] {
            let row_y = top_y + (row as i32) * line_h;
            let text = &formatted[i];

            let dist = (row as isize - cursor_row as isize).abs() as i32;
            let indent_left = (max_dist - dist).max(0) * indent_step;
            let text_x = base_x - indent_left;

            let color = if row == cursor_row {
                SUNFLOWER
            } else if i < applied_count {
                NEON_CARROT
            } else {
                Color::LIGHTGRAY
            };

            hud_text(draw, font, text, text_x, row_y, font_sz, color);
        }
    }
    if has_above {
        let ellipsis_y = top_y - line_h;
        if ellipsis_y >= 0 {
            hud_text(draw, font, "[ . . . ]", base_x, ellipsis_y, font_sz, Color::LIGHTGRAY);
        }
    }

    if has_below {
        let ellipsis_y = top_y + (visible_rows as i32) * line_h;
        if ellipsis_y + line_h <= screen_h {
            hud_text(draw, font, "[ . . . ]", base_x, ellipsis_y, font_sz, Color::LIGHTGRAY);
        }
    }
}

pub fn format_time_clock(seconds: f32) -> String {
    let clamped = if seconds.is_sign_negative() { 0.0 } else { seconds };
    let total_ms = (clamped * 1000.0).round() as u64;

    let minutes = total_ms / 60_000;
    let secs = (total_ms / 1000) % 60;
    let millis = total_ms % 1000;
    format!("{:02}:{:02}:{:03}", minutes, secs, millis)
}

pub fn format_bytes(bytes: usize) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = 1024.0 * 1024.0;

    let b = bytes as f64;

    if b < KB {
        format!("{bytes} B")
    } else if b < MB {
        let kb = b / KB;
        if kb < 10.0 {
            format!("{:.2} kB", kb)
        } else if kb < 100.0 {
            format!("{:.1} kB", kb)
        } else {
            format!("{:.0} kB", kb)
        }
    } else {
        let mb = b / MB;
        if mb < 10.0 {
            format!("{:.2} MB", mb)
        } else if mb < 100.0 {
            format!("{:.1} MB", mb)
        } else {
            format!("{:.0} MB", mb)
        }
    }
}
