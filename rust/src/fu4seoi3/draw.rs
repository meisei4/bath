use crate::fu4seoi3::config_and_state::*;
use crate::fu4seoi3::core::*;
use raylib::consts::MaterialMapIndex::MATERIAL_MAP_ALBEDO;
use raylib::ffi;
use raylib::prelude::*;

pub struct ColorGuard {
    mesh_colors_entries: Vec<(*mut ffi::Mesh, *mut std::ffi::c_uchar)>,
}

impl ColorGuard {
    pub fn hide(model: &mut Model) -> Self {
        let mut entries = Vec::new();
        for mesh in model.meshes_mut() {
            let mesh_ptr = mesh.as_mut() as *mut ffi::Mesh;
            let colors_ptr = unsafe { (*mesh_ptr).colors };
            unsafe {
                (*mesh_ptr).colors = std::ptr::null_mut();
            }

            entries.push((mesh_ptr, colors_ptr));
        }
        Self {
            mesh_colors_entries: entries,
        }
    }
}

impl Drop for ColorGuard {
    fn drop(&mut self) {
        for (mesh_ptr, cached_color_ptrs) in self.mesh_colors_entries.iter() {
            unsafe {
                (**mesh_ptr).colors = *cached_color_ptrs;
            }
        }
    }
}

pub struct TextureGuard {
    cached_texture_id: std::ffi::c_uint,
    restore_target: *mut Model,
}

impl TextureGuard {
    pub fn hide(model: &mut Model) -> Self {
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
    texture: &WeakTexture2D,
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
        Some(ColorGuard::hide(model))
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
    let _color_guard = ColorGuard::hide(model);
    let _texture_guard = TextureGuard::hide(model);
    rl3d.draw_model_wires_ex(&mut *model, position, Y_AXIS, rotation_deg, scale, ANAKIWA);
    unsafe { ffi::rlSetPointSize(4.0) };
    rl3d.draw_model_points_ex(&mut *model, position, Y_AXIS, rotation_deg, scale, ANAKIWA);
}

pub fn draw_filled_with_overlay(
    rl3d: &mut RaylibMode3D<RaylibDrawHandle>,
    model: &mut Model,
    texture: &WeakTexture2D,
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
    let _color_guard = ColorGuard::hide(model);
    let _texture_guard = TextureGuard::hide(model);
    unsafe {
        ffi::rlSetLineWidth(1.0);
    }
    rl3d.draw_model_wires_ex(&mut *model, position, Y_AXIS, rotation_deg, scale, MARINER);
    rl3d.draw_model_points_ex(&mut *model, position, Y_AXIS, rotation_deg, scale, LILAC);
}

//TODO: this needs to go, keep the draqw line segments, but the moment you get the arrow meshes colored and oriented uniquely nuke the entire "ribbon" stuff completely
pub fn draw_meta_field(
    rl3d: &mut RaylibMode3D<RaylibDrawHandle>,
    room: &Room,
    meta_model: &mut Model,
    opening_models: &mut Vec<Model>,
) {
    unsafe {
        ffi::rlSetLineWidth(2.0);
    }

    let mesh = &meta_model.meshes()[0];
    let vertices = mesh.vertices();
    let normals = mesh.normals().unwrap(); //TODO idk itll die

    for i in mesh.triangles().iter_vertices() {
        let dir = Vector2::new(normals[i].x, normals[i].z);
        let mag = normals[i].y.clamp(0.0, 1.0);
        let scaled_half_length = room.field.config.chi_arrow_length * mag * 0.5;
        let start = Vector3::new(
            vertices[i].x - dir.x * scaled_half_length,
            vertices[i].y,
            vertices[i].z - dir.y * scaled_half_length,
        );
        let end = Vector3::new(
            vertices[i].x + dir.x * scaled_half_length,
            vertices[i].y,
            vertices[i].z + dir.y * scaled_half_length,
        );

        draw_partitioned_line(rl3d, room, start, end);
    }

    let texture = {
        &meta_model.materials()[0]
            .get_material_texture(MATERIAL_MAP_ALBEDO)
            .cloned() //TODO dear lord
            .unwrap()
    };
    unsafe { ffi::rlSetPointSize(2.0) };
    draw_filled_with_overlay(rl3d, meta_model, texture, MODEL_POS, 0.0, MODEL_SCALE, true, true, None);

    for field_entity in &room.field.entities {
        let field_disrupter_color = match field_entity.kind {
            FieldEntityKind::Door { primary: true } => FieldOperatorKind::Emit.color(),
            FieldEntityKind::Door { primary: false } => Color::WHITE,
            FieldEntityKind::Window => FieldOperatorKind::Absorb.color(),
            FieldEntityKind::BackWall => FieldOperatorKind::Scatter.color(),
        };

        rl3d.draw_line3D(field_entity.p0, field_entity.p1, field_disrupter_color);

        if let Some(opening_model_index) = field_entity.model_index {
            //TODO: HOW TO AVOID BACKWALL? just dont make a model? Option on rotation is a dumb check imo
            let texture = {
                &opening_models[opening_model_index].materials()[0]
                    .get_material_texture(MATERIAL_MAP_ALBEDO)
                    .cloned() //TODO dear lord
                    .unwrap()
            };

            draw_filled_with_overlay(
                rl3d,
                &mut opening_models[opening_model_index],
                texture,
                field_entity.position(room),
                field_entity.rotation_into_room(room).unwrap(), //TODO: just be careful here later
                MODEL_SCALE,
                false,
                true,
                Some(field_disrupter_color),
            );
        } else {
            rl3d.draw_sphere(field_entity.center(), 0.33, field_disrupter_color);
        }
    }
    unsafe {
        ffi::rlSetLineWidth(1.0);
    }
}

fn draw_partitioned_line(rl3d: &mut RaylibMode3D<RaylibDrawHandle>, room: &Room, start: Vector3, end: Vector3) {
    const SEGMENTS: usize = 3;
    let mut prev_pos = start;
    let mut prev_kind = room.get_dominant_field_operator_at(start);

    for i in 1..=SEGMENTS {
        let t = i as f32 / SEGMENTS as f32;
        let curr_pos = Vector3::new(
            start.x + (end.x - start.x) * t,
            start.y + (end.y - start.y) * t,
            start.z + (end.z - start.z) * t,
        );

        let curr_kind = room.get_dominant_field_operator_at(curr_pos);
        let color = prev_kind.color();
        rl3d.draw_line3D(prev_pos, curr_pos, color);

        prev_pos = curr_pos;
        prev_kind = curr_kind;
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
    let _color_guard = ColorGuard::hide(model);
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
            let texture = {
                &desc.ndc.materials()[0]
                    .get_material_texture(MATERIAL_MAP_ALBEDO)
                    .cloned()
                    .unwrap()
            };
            draw_filled_with_overlay(
                rl3d,
                &mut desc.ndc,
                texture, //TODO: raylib probably wont free this?
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

pub fn update_animated_mesh(
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
