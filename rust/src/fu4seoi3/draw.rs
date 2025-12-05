use crate::fu4seoi3::core::*;
use raylib::consts::MaterialMapIndex::MATERIAL_MAP_ALBEDO;
use raylib::ffi;
use raylib::prelude::*;

pub const BAHAMA_BLUE: Color = Color::new(0, 102, 153, 255);
pub const SUNFLOWER: Color = Color::new(255, 204, 153, 255);
pub const PALE_CANARY: Color = Color::new(255, 255, 153, 255);
pub const ANAKIWA: Color = Color::new(153, 204, 255, 255);
pub const MARINER: Color = Color::new(51, 102, 204, 255);
pub const NEON_CARROT: Color = Color::new(255, 153, 51, 255);
pub const EGGPLANT: Color = Color::new(102, 68, 102, 255);
pub const HOPBUSH: Color = Color::new(204, 102, 153, 255);
pub const LILAC: Color = Color::new(204, 153, 204, 255);
pub const RED_DAMASK: Color = Color::new(221, 102, 68, 255);
pub const CHESTNUT_ROSE: Color = Color::new(204, 102, 102, 255);

pub struct ColorGuard {
    cached_colors_ptr: *mut std::ffi::c_uchar,
    restore_target: *mut ffi::Mesh,
}

impl ColorGuard {
    pub fn hide(mesh: &mut WeakMesh) -> Self {
        let mesh_ptr = mesh.as_mut() as *mut ffi::Mesh;
        let colors_ptr = unsafe { (*mesh_ptr).colors };
        unsafe {
            (*mesh_ptr).colors = std::ptr::null_mut();
        }
        Self {
            cached_colors_ptr: colors_ptr,
            restore_target: mesh_ptr,
        }
    }
}

impl Drop for ColorGuard {
    fn drop(&mut self) {
        unsafe {
            (*self.restore_target).colors = self.cached_colors_ptr;
        }
    }
}

pub struct TextureGuard {
    cached_texture_id: std::ffi::c_uint,
    restore_target: *mut Model,
}

impl TextureGuard {
    pub fn hide(model: &mut Model) -> Self {
        use raylib::consts::MaterialMapIndex::MATERIAL_MAP_ALBEDO;
        let cached_id = model.materials_mut()[0].maps_mut()[MATERIAL_MAP_ALBEDO as usize]
            .texture
            .id;
        model.materials_mut()[0].maps_mut()[MATERIAL_MAP_ALBEDO as usize]
            .texture
            .id = 0;
        Self {
            cached_texture_id: cached_id,
            restore_target: model as *mut Model,
        }
    }

    pub fn set_texture(model: &mut Model, texture_id: u32) -> Self {
        use raylib::consts::MaterialMapIndex::MATERIAL_MAP_ALBEDO;
        let cached_id = model.materials_mut()[0].maps_mut()[MATERIAL_MAP_ALBEDO as usize]
            .texture
            .id;
        model.materials_mut()[0].maps_mut()[MATERIAL_MAP_ALBEDO as usize]
            .texture
            .id = texture_id;
        Self {
            cached_texture_id: cached_id,
            restore_target: model as *mut Model,
        }
    }
}

impl Drop for TextureGuard {
    fn drop(&mut self) {
        use raylib::consts::MaterialMapIndex::MATERIAL_MAP_ALBEDO;
        unsafe {
            (*self.restore_target).materials_mut()[0].maps_mut()[MATERIAL_MAP_ALBEDO as usize]
                .texture
                .id = self.cached_texture_id;
        }
    }
}

pub fn draw_filled(
    rl3d: &mut RaylibMode3D<RaylibDrawHandle>,
    model: &mut Model,
    texture: &Texture2D,
    position: Vector3,
    rotation_deg: f32,
    scale: Vector3,
    vertex_colors_enabled: bool,
    texture_enabled: bool,
    color: Option<Color>,
) {
    if !vertex_colors_enabled && !texture_enabled && color.is_none() {
        return;
    }
    let _color_guard = if !vertex_colors_enabled {
        Some(ColorGuard::hide(&mut model.meshes_mut()[0]))
    } else {
        None
    };
    let texture_id = if texture_enabled { texture.id } else { 0 };
    let _texture_guard = TextureGuard::set_texture(model, texture_id);
    let tint = if vertex_colors_enabled {
        Color::WHITE
    } else {
        color.unwrap_or(Color::WHITE)
    };
    rl3d.draw_model_ex(model, position, Y_AXIS, rotation_deg, scale, tint);
}

pub fn draw_wires_and_points(
    rl3d: &mut RaylibMode3D<RaylibDrawHandle>,
    model: &mut Model,
    position: Vector3,
    rotation_deg: f32,
    scale: Vector3,
) {
    let _color_guard = ColorGuard::hide(&mut model.meshes_mut()[0]);
    let _texture_guard = TextureGuard::hide(model);
    rl3d.draw_model_wires_ex(&mut *model, position, Y_AXIS, rotation_deg, scale, ANAKIWA);
    unsafe { ffi::rlSetPointSize(4.0) };
    rl3d.draw_model_points_ex(&mut *model, position, Y_AXIS, rotation_deg, scale, ANAKIWA);
}

pub fn draw_filled_with_overlay(
    rl3d: &mut RaylibMode3D<RaylibDrawHandle>,
    model: &mut Model,
    texture: &Texture2D,
    position: Vector3,
    rotation_deg: f32,
    scale: Vector3,
    color_enabled: bool,
    texture_enabled: bool,
    color: Option<Color>,
) {
    if color_enabled || texture_enabled || color.is_some() {
        draw_filled(
            rl3d,
            model,
            texture,
            position,
            rotation_deg,
            scale,
            color_enabled,
            texture_enabled,
            color,
        );
    }
    let _color_guard = ColorGuard::hide(&mut model.meshes_mut()[0]);
    let _texture_guard = TextureGuard::hide(model);
    unsafe {
        ffi::rlSetLineWidth(1.0);
    }
    rl3d.draw_model_wires_ex(&mut *model, position, Y_AXIS, rotation_deg, scale, MARINER);
    unsafe { ffi::rlSetPointSize(4.0) };
    rl3d.draw_model_points_ex(&mut *model, position, Y_AXIS, rotation_deg, scale, LILAC);
}

pub fn draw_chi_field(rl3d: &mut RaylibMode3D<RaylibDrawHandle>, room: &Room, opening_models: &mut Vec<Model>) {
    unsafe {
        ffi::rlSetLineWidth(2.0);
    }

    for sample in &room.field_samples {
        let center = sample.position;
        let m = sample.magnitude.clamp(0.0, 1.0);
        let scaled_length = room.config.chi_arrow_length * m;
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

        let color = chi_disrupter_color(sample.dominant);
        rl3d.draw_line3D(start, end, color);
    }

    for opening in &room.openings {
        let color = match opening.kind {
            OpeningKind::Door { primary: true } => chi_disrupter_color(FieldDisrupter::DoorPrimary),
            OpeningKind::Door { primary: false } => Color::WHITE, // TODO: ew, but fine for now
            OpeningKind::Window => chi_disrupter_color(FieldDisrupter::Window),
        };

        rl3d.draw_line3D(opening.p0, opening.p1, color);

        if let Some(opening_model_index) = opening.model_index {
            let pos = opening.position(room);
            draw_filled_with_overlay(
                rl3d,
                &mut opening_models[opening_model_index],
                &Texture2D::default(), // TODO: get actual Texture or WeakTexture
                pos,
                opening.rotation_into_room(room),
                MODEL_SCALE,
                false,
                false,
                Some(color),
            );
        } else {
            rl3d.draw_sphere(opening.center(), 0.33, color);
        }
    }
    unsafe {
        ffi::rlSetLineWidth(1.0);
    }
}

fn chi_disrupter_color(kind: FieldDisrupter) -> Color {
    match kind {
        FieldDisrupter::Base => SUNFLOWER,
        FieldDisrupter::DoorPrimary => ANAKIWA,
        FieldDisrupter::Window => PALE_CANARY,
        FieldDisrupter::BackWall => LILAC,
    }
}

pub fn draw_hint(
    rl3d: &mut RaylibMode3D<RaylibDrawHandle>,
    model: &mut Model,
    position: Vector3,
    rotation_deg: f32,
    scale: Vector3,
    occupied: bool,
) {
    let _color_guard = ColorGuard::hide(&mut model.meshes_mut()[0]);
    let _texture_guard = TextureGuard::hide(model);
    let hint_color = if occupied { RED_DAMASK } else { ANAKIWA };
    unsafe { ffi::rlDisableDepthTest() };
    rl3d.draw_model_wires_ex(&mut *model, position, Y_AXIS, rotation_deg, scale, hint_color);
    unsafe { ffi::rlEnableDepthTest() };
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

pub fn draw_placed_cells(
    rl3d: &mut RaylibMode3D<RaylibDrawHandle>,
    meshes: &mut [MeshDescriptor],
    placed_cells: &mut [PlacedCell],
    total_time: f32,
    room: &Room,
    view_config: &ViewConfig,
) {
    for cell in placed_cells.iter_mut() {
        if cell.mesh_index >= meshes.len() {
            continue;
        }
        let cell_pos = room.cell_center(cell.ix, cell.iy, cell.iz);
        let age = cell.age_at(total_time);
        if age >= view_config.placement_anim_dur_seconds {
            cell.settled = true;
        }

        let current_scale = cell.scale_at(total_time, view_config);
        if age >= view_config.placement_anim_dur_seconds {
            cell.settled = true;
        }

        let desc = &mut meshes[cell.mesh_index];
        if cell.settled {
            draw_filled_with_overlay(
                rl3d,
                &mut desc.ndc,
                &desc.texture,
                cell_pos,
                0.0,
                MODEL_SCALE,
                cell.color_enabled,
                cell.texture_enabled,
                None,
            );
        } else {
            let scale_vec = Vector3::new(current_scale, current_scale, current_scale);
            draw_wires_and_points(rl3d, &mut desc.ndc, cell_pos, 0.0, scale_vec);
        }
    }
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

pub fn draw_room_floor_grid(rl3d: &mut RaylibMode3D<RaylibDrawHandle>, room: &Room) {
    let origin = room.origin;
    let floor_y = origin.y;
    for x in 0..=room.w {
        let x_world = origin.x + x as f32;
        let start = Vector3::new(x_world, floor_y, origin.z);
        let end = Vector3::new(x_world, floor_y, origin.z + room.d as f32);
        rl3d.draw_line3D(start, end, HOPBUSH);
    }
    for z in 0..=room.d {
        let z_world = origin.z + z as f32;
        let start = Vector3::new(origin.x, floor_y, z_world);
        let end = Vector3::new(origin.x + room.w as f32, floor_y, z_world);
        rl3d.draw_line3D(start, end, HOPBUSH);
    }
}

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
    target_mesh: usize,
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
        if view_state.jugemu_ortho_mode {
            "ORTHOGRAPHIC"
        } else {
            "PERSPECTIVE"
        },
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
    line_y = hud_row(
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
    right_y = hud_row(
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
        if view_state.aspect_correct {
            "CORRECT"
        } else {
            "INCORRECT"
        },
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
        if view_state.ortho_mode {
            "ORTHOGRAPHIC"
        } else {
            "PERSPECTIVE"
        },
        layout.left_label_x,
        layout.left_value_x,
        bottom_y,
        layout.font_size_main,
        layout.line_height_main,
        SUNFLOWER,
        if view_state.ortho_mode { BAHAMA_BLUE } else { ANAKIWA },
    );

    bottom_y = hud_row(
        draw_handle,
        font,
        "SPACE [ N ]:",
        if view_state.ndc_space { "NDC" } else { "WORLD" },
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

    if let (Some(cell_idx), Some((ix, iy, iz))) = (hover_state.placed_cell_index, hover_state.indices) {
        if cell_idx < placed_cells.len() {
            let cell = &placed_cells[cell_idx];
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

    for (i, desc) in meshes.iter().enumerate() {
        let name = mesh_name(i, meshes);
        let world = desc.metrics_world;
        let ndc = desc.metrics_ndc;
        let combined_bytes = desc.combined_bytes;

        hud_text(
            draw_handle,
            font,
            &format!(
                "{}: {} B (WRLD {}v, NDC {}v)",
                name, combined_bytes, world.vertex_count, ndc.vertex_count
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
        hud_text(draw_handle, font, "ROOM/OPENING MESHES:", perf_x, y, font_sz, SUNFLOWER);
        y += line;

        for (i, m) in opening_metrics.iter().enumerate() {
            hud_text(
                draw_handle,
                font,
                &format!("OPENING_{}: {} B ({}v)", i, m.total_bytes, m.vertex_count),
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
        let pointer_idx = applied_count - 1;
        row_to_index[cursor_row] = Some(pointer_idx);
        let mut row = cursor_row + 1;
        let mut idx = pointer_idx as isize - 1;
        while row < visible_rows && idx >= 0 {
            row_to_index[row] = Some(idx as usize);
            row += 1;
            idx -= 1;
        }
        if idx >= 0 {
            has_below = true;
        }
        let mut row = cursor_row;
        let mut idx = applied_count;
        while row > 0 && idx < len {
            row -= 1;
            row_to_index[row] = Some(idx);
            idx += 1;
        }
        if idx < len {
            has_above = true;
        }
    } else {
        let mut idx = 0;
        row_to_index[cursor_row] = Some(idx);
        idx += 1;

        let mut row = cursor_row;
        while row > 0 && idx < len {
            row -= 1;
            row_to_index[row] = Some(idx);
            idx += 1;
        }
        if idx < len {
            has_above = true;
        }
    }
    let cursor_indent_left = max_dist * indent_step;
    let cursor_y = top_y + (cursor_row as i32) * line_h;
    let cursor_x = base_x - cursor_indent_left - cursor_dx;
    hud_text(draw, font, ">", cursor_x, cursor_y, font_sz, SUNFLOWER);
    for row in 0..visible_rows {
        if let Some(idx) = row_to_index[row] {
            let row_y = top_y + (row as i32) * line_h;
            let text = &formatted[idx];

            let dist = (row as isize - cursor_row as isize).abs() as i32;
            let indent_left = (max_dist - dist).max(0) * indent_step;
            let text_x = base_x - indent_left;

            let color = if row == cursor_row {
                SUNFLOWER
            } else if idx < applied_count {
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

pub fn handle_jugemu_projection_toggle(handle: &RaylibHandle, view_state: &mut ViewState, jugemu: &mut Camera3D) {
    if handle.is_key_pressed(KeyboardKey::KEY_P) {
        if view_state.jugemu_ortho_mode {
            view_state.jugemu_zoom.fovy_ortho = jugemu.fovy;
            view_state.jugemu_zoom.distance_ortho = camera_distance(jugemu);
            jugemu.fovy = view_state.jugemu_zoom.fovy_perspective;
            let dir = jugemu.position.normalize();
            jugemu.position = dir * view_state.jugemu_zoom.distance_perspective;
        } else {
            view_state.jugemu_zoom.fovy_perspective = jugemu.fovy;
            view_state.jugemu_zoom.distance_perspective = camera_distance(jugemu);
            jugemu.fovy = view_state.jugemu_zoom.fovy_ortho;
            let dir = jugemu.position.normalize();
            jugemu.position = dir * view_state.jugemu_zoom.distance_ortho;
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
pub fn update_view_from_input(handle: &RaylibHandle, view_state: &mut ViewState, jugemu: &mut Camera3D) {
    handle_view_toggles(handle, view_state);
    handle_jugemu_projection_toggle(handle, view_state, jugemu);
    handle_mesh_selection(handle, view_state);
}

pub fn update_ghost_mesh(
    ndc_model: &mut Model,
    world_model: &mut Model,
    i_time: f32,
    mesh_samples: &[Vec<Vector3>],
    main: &Camera3D,
    mesh_rotation: f32,
    frame_dynamic_metrics: &mut FrameDynamicMetrics,
    field_config: &FieldConfig,
) {
    interpolate_between_deformed_vertices(ndc_model, i_time, mesh_samples, frame_dynamic_metrics, field_config);
    interpolate_between_deformed_vertices(world_model, i_time, mesh_samples, frame_dynamic_metrics, field_config);
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
        let va = Vector3::new(vertices[a].x, vertices[a].y, vertices[a].z);
        let vb = Vector3::new(vertices[b].x, vertices[b].y, vertices[b].z);
        let vc = Vector3::new(vertices[c].x, vertices[c].y, vertices[c].z);

        let face_normal = triangle_normal(va, vb, vc);
        normals[a] += face_normal;
        normals[b] += face_normal;
        normals[c] += face_normal;
    }

    for i in mesh.triangles().iter_vertices() {
        normals[i] = normals[i].normalize_or_zero();
    }

    let normals_vec: Vec<Vector3> = normals.iter().map(|n| Vector3::new(n.x, n.y, n.z)).collect();

    mesh.normals_mut().unwrap().copy_from_slice(&normals_vec);
    frame_dynamic_metrics.vertex_normals_written += normals_vec.len();
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
        let v = vertices[i];
        let model_center_to_vertex = Vector3::new(v.x, v.y, v.z).normalize_or_zero();
        let cos_theta = model_center_to_vertex.dot(model_center_to_camera);

        if cos_theta <= 0.0 {
            alpha_buffer[i] = 0;
            continue;
        }

        let fade_scalar = (cos_theta / cos_fade_angle).clamp(0.0, 1.0);
        let alpha = fade_scalar.powi(4);
        alpha_buffer[i] = (alpha * 255.0).round() as u8;
    }

    let colors = mesh.colors_mut().unwrap();
    for i in 0..alpha_buffer.len() {
        colors[i].a = alpha_buffer[i];
    }
    frame_dynamic_metrics.vertex_colors_written += alpha_buffer.len();
}
