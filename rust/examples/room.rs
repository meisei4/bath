use asset_payload::{SPHERE_GLTF_PATH, SPHERE_PATH};
use raylib::consts::CameraProjection::{CAMERA_ORTHOGRAPHIC, CAMERA_PERSPECTIVE};
use raylib::consts::MaterialMapIndex::MATERIAL_MAP_ALBEDO;
use raylib::ffi::{rlGetTextureIdDefault, rlSetPointSize};
use raylib::math::glam::{Mat4, Vec3};
use raylib::prelude::*;
use std::f32::consts::{PI, TAU};
use std::ops::{Add, Sub};

const ROOM_W: i32 = 9;
const ROOM_H: i32 = 3;
const ROOM_D: i32 = 9;

const ROOM_CENTER_X: i32 = ROOM_W / 2;
const ROOM_CENTER_Y: i32 = ROOM_H / 2;
const ROOM_CENTER_Z: i32 = ROOM_D / 2;

pub const HALF: f32 = 0.5;
pub const GRID_SCALE: f32 = 4.0;
pub const GRID_CELL_SIZE: f32 = 1.0 / GRID_SCALE;
pub const GRID_ORIGIN_INDEX: Vector2 = Vector2::new(0.0, 0.0);
pub const GRID_ORIGIN_OFFSET_CELLS: Vector2 = Vector2::new(2.0, 2.0);
pub const GRID_ORIGIN_UV_OFFSET: Vector2 = Vector2::new(
    (GRID_ORIGIN_INDEX.x + GRID_ORIGIN_OFFSET_CELLS.x) * GRID_CELL_SIZE,
    (GRID_ORIGIN_INDEX.y + GRID_ORIGIN_OFFSET_CELLS.y) * GRID_CELL_SIZE,
);

pub const LIGHT_WAVE_SPATIAL_FREQ_X: f32 = 8.0;
pub const LIGHT_WAVE_SPATIAL_FREQ_Y: f32 = 8.0;
pub const LIGHT_WAVE_TEMPORAL_FREQ_X: f32 = 255.0 * PI / 10.0;
pub const LIGHT_WAVE_TEMPORAL_FREQ_Y: f32 = 7.0 * PI / 10.0;
pub const LIGHT_WAVE_AMPLITUDE_X: f32 = 0.0;
pub const LIGHT_WAVE_AMPLITUDE_Y: f32 = 0.1;
pub const UMBRAL_MASK_OUTER_RADIUS: f32 = 0.40;
pub const UMBRAL_MASK_FADE_BAND: f32 = 0.025;
pub const UMBRAL_MASK_CENTER: Vector2 = Vector2::new(HALF, HALF);
pub const RADIAL_FIELD_SIZE: usize = 64;
pub const ROTATION_FREQUENCY_HZ: f32 = 0.05;
pub const ANGULAR_VELOCITY: f32 = TAU * ROTATION_FREQUENCY_HZ;
pub const TIME_BETWEEN_SAMPLES: f32 = 0.5;
pub const ROTATIONAL_SAMPLES_FOR_INV_PROJ: usize = 40;

const FOVY_PERSPECTIVE: f32 = 50.0;
fn NEAR_PLANE_HEIGHT_ORTHOGRAPHIC() -> f32 {
    2.0 * (FOVY_PERSPECTIVE * 0.5).to_radians().tan()
}
const BLEND_SCALAR: f32 = 5.0;

const Y_AXIS: Vector3 = Vector3::new(0.0, 1.0, 0.0);
const MAIN_POS: Vector3 = Vector3::new(0.0, 0.0, 2.0);
const JUGEMU_POS_ISO: Vector3 = Vector3::new(1.0, 1.0, 1.0);
const JUGEMU_DISTANCE_ORTHO: f32 = 6.5;
const JUGEMU_DISTANCE_PERSPECTIVE: f32 = 9.0;

pub const FOVY_ORTHOGRAPHIC: f32 = 9.0;
pub const MODEL_POS: Vector3 = Vector3::ZERO;
pub const SCALE_ELEMENT: f32 = 1.0;
pub const MODEL_SCALE: Vector3 = Vector3::new(SCALE_ELEMENT, SCALE_ELEMENT, SCALE_ELEMENT);

const RES_SCALE: f32 = 1.5;
const DC_WIDTH_BASE: f32 = 640.0;
const DC_HEIGHT_BASE: f32 = 480.0;
const DC_WIDTH: i32 = (DC_WIDTH_BASE * RES_SCALE) as i32;
const DC_HEIGHT: i32 = (DC_HEIGHT_BASE * RES_SCALE) as i32;

const HUD_MARGIN: i32 = 12;
const HUD_LINE_HEIGHT: i32 = 22;

const FONT_SIZE: i32 = 20;
const BAHAMA_BLUE: Color = Color::new(0, 102, 153, 255);
const SUNFLOWER: Color = Color::new(255, 204, 153, 255);
const ANAKIWA: Color = Color::new(153, 204, 255, 255);
const MARINER: Color = Color::new(51, 102, 204, 255);
const NEON_CARROT: Color = Color::new(255, 153, 51, 255);
const EGGPLANT: Color = Color::new(102, 68, 102, 255);
const HOPBUSH: Color = Color::new(204, 102, 153, 255);
const LILAC: Color = Color::new(204, 153, 204, 255);
const RED_DAMASK: Color = Color::new(221, 102, 68, 255);
const CHESTNUT_ROSE: Color = Color::new(204, 102, 102, 255);

const NUM_MODELS: usize = 2;

const PLACEMENT_SCALE_DURATION: f32 = 0.15;
const HINT_SCALE: f32 = 0.66;
struct PlacedCell {
    ix: i32,
    iy: i32,
    iz: i32,
    mesh_index: usize,
    placed_time: f32,
    settled: bool,
    texture_enabled: bool,
    color_enabled: bool,
}

fn is_cell_occupied(placed_cells: &[PlacedCell], ix: i32, iy: i32, iz: i32) -> bool {
    placed_cells
        .iter()
        .any(|cell| cell.ix == ix && cell.iy == iy && cell.iz == iz)
}

struct ViewState {
    ndc_space: bool,
    aspect_correct: bool,
    paused: bool,
    color_mode: bool,
    texture_mode: bool,
    jugemu_mode: bool,
    ortho_mode: bool,
    jugemu_ortho_mode: bool,
    target_mesh_index: usize,
}

impl ViewState {
    fn new() -> Self {
        Self {
            ndc_space: false,
            aspect_correct: true,
            paused: false,
            color_mode: true,
            texture_mode: false,
            jugemu_mode: true,
            ortho_mode: false,
            jugemu_ortho_mode: true,
            target_mesh_index: 1,
        }
    }

    fn toggle_ndc(&mut self) {
        self.ndc_space = !self.ndc_space;
    }
    fn toggle_aspect(&mut self) {
        self.aspect_correct = !self.aspect_correct;
    }
    fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }
    fn toggle_color(&mut self) {
        self.color_mode = !self.color_mode;
    }
    fn toggle_texture(&mut self) {
        self.texture_mode = !self.texture_mode;
    }
    fn toggle_jugemu(&mut self) {
        self.jugemu_mode = !self.jugemu_mode;
    }
    fn toggle_ortho(&mut self) {
        self.ortho_mode = !self.ortho_mode;
    }
    fn toggle_jugemu_ortho(&mut self) {
        self.jugemu_ortho_mode = !self.jugemu_ortho_mode;
    }

    fn set_mesh(&mut self, index: usize) {
        self.target_mesh_index = index;
    }
}

struct ColorGuard {
    cached_colors_ptr: *mut std::ffi::c_uchar,
    restore_target: *mut ffi::Mesh,
}

impl ColorGuard {
    fn hide(mesh: &mut WeakMesh) -> Self {
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
struct TextureGuard {
    cached_texture_id: std::ffi::c_uint,
    restore_target: *mut Model,
}

impl TextureGuard {
    fn hide(model: &mut Model) -> Self {
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

fn main() {
    let mut i_time = 0.0f32;
    let mut total_time = 0.0f32;
    let mut view_state = ViewState::new();
    let mut placed_cells: Vec<PlacedCell> = Vec::new();
    let (mut handle, thread) = init()
        .size(DC_WIDTH, DC_HEIGHT)
        .title("raylib [core] example - fixed function didactic")
        .build();

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
        // fovy: FOVY_PERSPECTIVE,
        // projection: CAMERA_PERSPECTIVE,
    };

    let mut prev_fovy_ortho = FOVY_ORTHOGRAPHIC;
    let mut prev_fovy_perspective = FOVY_PERSPECTIVE;
    let mut prev_distance_ortho = JUGEMU_DISTANCE_ORTHO;
    let mut prev_distance_perspective = JUGEMU_DISTANCE_PERSPECTIVE;

    let mut world_models: Vec<Model> = Vec::new();
    let mut ndc_models: Vec<Model> = Vec::new();
    let mut mesh_textures = Vec::new();
    //TODO: WHEN YOU TOGGLE BETWEEN THE GHOST MODEL AND OTHER MESHES ITS DEFORMATION CYCLES GETS MESSED UP! I THINK ITS RELATED TO i_time and how that is updated...
    let mut ghost_model = handle.load_model(&thread, SPHERE_GLTF_PATH).unwrap();
    let checked_img = Image::gen_image_checked(16, 16, 1, 1, Color::BLACK, Color::WHITE);
    let mesh_texture = handle.load_texture_from_image(&thread, &checked_img).unwrap();
    ghost_model.materials_mut()[0].set_material_texture(MATERIAL_MAP_ALBEDO, &mesh_texture);

    let mesh_samples = collect_deformed_vertex_samples(ghost_model.meshes()[0].vertices());
    interpolate_between_deformed_vertices(&mut ghost_model, i_time, &mesh_samples);

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

    while !handle.window_should_close() {
        aspect = handle.get_screen_width() as f32 / handle.get_screen_height() as f32;

        if handle.is_key_pressed(KeyboardKey::KEY_N) {
            view_state.toggle_ndc();
        }
        if handle.is_key_pressed(KeyboardKey::KEY_Q) {
            view_state.toggle_aspect();
        }
        if handle.is_key_pressed(KeyboardKey::KEY_SPACE) {
            view_state.toggle_pause();
        }
        if handle.is_key_pressed(KeyboardKey::KEY_C) {
            view_state.toggle_color();
        }
        if handle.is_key_pressed(KeyboardKey::KEY_T) {
            view_state.toggle_texture();
        }
        if handle.is_key_pressed(KeyboardKey::KEY_J) {
            view_state.toggle_jugemu();
        }
        if handle.is_key_pressed(KeyboardKey::KEY_O) {
            view_state.toggle_ortho();
        }
        if handle.is_key_pressed(KeyboardKey::KEY_P) {
            if view_state.jugemu_ortho_mode {
                prev_fovy_ortho = jugemu.fovy;
                let current_distance = {
                    let offset = Vector3::new(
                        jugemu.position.x - jugemu.target.x,
                        jugemu.position.y - jugemu.target.y,
                        jugemu.position.z - jugemu.target.z,
                    );
                    (offset.x * offset.x + offset.y * offset.y + offset.z * offset.z).sqrt()
                };
                prev_distance_ortho = current_distance;

                jugemu.fovy = prev_fovy_perspective;
                let dir = jugemu.position.normalize();
                jugemu.position = dir * prev_distance_perspective;
            } else {
                prev_fovy_perspective = jugemu.fovy;
                let current_distance = {
                    let offset = Vector3::new(
                        jugemu.position.x - jugemu.target.x,
                        jugemu.position.y - jugemu.target.y,
                        jugemu.position.z - jugemu.target.z,
                    );
                    (offset.x * offset.x + offset.y * offset.y + offset.z * offset.z).sqrt()
                };
                prev_distance_perspective = current_distance;

                jugemu.fovy = prev_fovy_ortho;
                let dir = jugemu.position.normalize();
                jugemu.position = dir * prev_distance_ortho;
            }
            view_state.toggle_jugemu_ortho();
        }
        if handle.is_key_pressed(KeyboardKey::KEY_ONE) {
            view_state.set_mesh(0);
        }
        if handle.is_key_pressed(KeyboardKey::KEY_TWO) {
            view_state.set_mesh(1);
        }
        if handle.is_key_pressed(KeyboardKey::KEY_THREE) {
            view_state.set_mesh(2);
        }

        let s_blend = space_blend_factor(handle.get_frame_time(), view_state.ndc_space);
        let a_blend = aspect_blend_factor(handle.get_frame_time(), view_state.aspect_correct);
        let o_blend = ortho_blend_factor(handle.get_frame_time(), view_state.ortho_mode);

        if !view_state.paused {
            mesh_rotation -= ANGULAR_VELOCITY * handle.get_frame_time();
            total_time += handle.get_frame_time(); //TODO this might need to just be i_time? the ghost mesh is confusing here
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
        if target_mesh == 0 {
            if !view_state.paused {
                i_time += handle.get_frame_time();
            }
            interpolate_between_deformed_vertices(&mut ndc_models[0], i_time, &mesh_samples);
            interpolate_between_deformed_vertices(&mut world_models[0], i_time, &mesh_samples);
            update_normals_for_silhouette(&mut ndc_models[0].meshes_mut()[0]);
            update_normals_for_silhouette(&mut world_models[0].meshes_mut()[0]);
            fade_vertex_colors_silhouette_rim(&mut ndc_models[0].meshes_mut()[0], &main, mesh_rotation);
            fade_vertex_colors_silhouette_rim(&mut world_models[0].meshes_mut()[0], &main, mesh_rotation);
        }

        world_to_ndc_space(
            &mut main,
            aspect,
            near,
            far,
            &mut world_models[target_mesh],
            &mut ndc_models[target_mesh],
            mesh_rotation,
            o_blend,
            a_blend,
        );
        {
            let world_mesh = &world_models[target_mesh].meshes()[0];
            let ndc_mesh = &mut ndc_models[target_mesh].meshes_mut()[0];
            let world_vertices = world_mesh.vertices();
            let ndc_vertices = ndc_mesh.vertices_mut();

            for [a, b, c] in world_mesh.triangles() {
                for &i in [a, b, c].iter() {
                    ndc_vertices[i].x = lerp(world_vertices[i].x, ndc_vertices[i].x, s_blend);
                    ndc_vertices[i].y = lerp(world_vertices[i].y, ndc_vertices[i].y, s_blend);
                    ndc_vertices[i].z = lerp(world_vertices[i].z, ndc_vertices[i].z, s_blend);
                }
            }
        }

        let z_shift_for_aspect = lerp(
            cached_ndc_z_shifts_isotropic[target_mesh],
            cached_ndc_z_shifts_anisotropic[target_mesh],
            a_blend,
        );
        jugemu.target = Vector3::new(MODEL_POS.x, MODEL_POS.y, lerp(MODEL_POS.z, z_shift_for_aspect, s_blend));

        update_spatial_frame(
            &mut main,
            aspect,
            near,
            far,
            &mut spatial_frame_model.meshes_mut()[0],
            s_blend,
            a_blend,
            o_blend,
        );

        let hovered_cell_indices = get_hovered_room_floor_cell(&handle, &jugemu);

        let hovered_cell_center = if hovered_cell_indices.is_some() {
            let indices = hovered_cell_indices.unwrap();
            Some(cell_center(indices.0, indices.1, indices.2))
        } else {
            None
        };

        let hovered_cell_occupied = if hovered_cell_indices.is_some() {
            let indices = hovered_cell_indices.unwrap();
            is_cell_occupied(&placed_cells, indices.0, indices.1, indices.2)
        } else {
            false
        };

        let hovered_placed_cell_index = if hovered_cell_indices.is_some() && hovered_cell_occupied {
            let indices = hovered_cell_indices.unwrap();
            placed_cells
                .iter()
                .position(|cell| cell.ix == indices.0 && cell.iy == indices.1 && cell.iz == indices.2)
        } else {
            None
        };
        if let Some(cell_idx) = hovered_placed_cell_index {
            if handle.is_key_pressed(KeyboardKey::KEY_T) {
                placed_cells[cell_idx].texture_enabled = !placed_cells[cell_idx].texture_enabled;
            }
            if handle.is_key_pressed(KeyboardKey::KEY_C) {
                placed_cells[cell_idx].color_enabled = !placed_cells[cell_idx].color_enabled;
            }
        }

        if handle.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            if hovered_cell_indices.is_some() && !hovered_cell_occupied {
                let indices = hovered_cell_indices.unwrap();
                placed_cells.push(PlacedCell {
                    ix: indices.0,
                    iy: indices.1,
                    iz: indices.2,
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

            if view_state.jugemu_mode {
                draw_spatial_frame(&mut rl3d, &spatial_frame_model.meshes_mut()[0]);
            }

            draw_room_floor_grid(&mut rl3d);
            rl3d.draw_cube_wires(MODEL_POS, ROOM_W as f32, ROOM_H as f32, ROOM_D as f32, RED_DAMASK);

            for placed_cell in placed_cells.iter_mut() {
                let cell_pos = cell_center(placed_cell.ix, placed_cell.iy, placed_cell.iz);
                let age = total_time - placed_cell.placed_time;
                let scale_progress = (age / PLACEMENT_SCALE_DURATION).clamp(0.0, 1.0);
                let current_scale = lerp(HINT_SCALE, 1.0, scale_progress);

                if scale_progress >= 1.0 {
                    placed_cell.settled = true;
                }

                let placed_model = &mut ndc_models[placed_cell.mesh_index];

                if placed_cell.settled {
                    if placed_cell.color_enabled || placed_cell.texture_enabled {
                        let _color_guard = if placed_cell.texture_enabled && !placed_cell.color_enabled {
                            Some(ColorGuard::hide(&mut placed_model.meshes_mut()[0]))
                        } else {
                            None
                        };

                        placed_model.materials_mut()[0].maps_mut()[MATERIAL_MAP_ALBEDO as usize]
                            .texture
                            .id = if placed_cell.texture_enabled {
                            mesh_textures[placed_cell.mesh_index].id
                        } else {
                            unsafe { rlGetTextureIdDefault() }
                        };

                        rl3d.draw_model_ex(&mut *placed_model, cell_pos, Y_AXIS, 0.0, MODEL_SCALE, Color::WHITE);

                        placed_model.materials_mut()[0].maps_mut()[MATERIAL_MAP_ALBEDO as usize]
                            .texture
                            .id = unsafe { rlGetTextureIdDefault() };
                    }

                    let _color_guard = ColorGuard::hide(&mut placed_model.meshes_mut()[0]);
                    let cache_texture_id = placed_model.materials()[0].maps()[MATERIAL_MAP_ALBEDO as usize]
                        .texture()
                        .id;
                    placed_model.materials_mut()[0].maps_mut()[MATERIAL_MAP_ALBEDO as usize]
                        .texture
                        .id = unsafe { rlGetTextureIdDefault() };

                    rl3d.draw_model_wires_ex(&mut *placed_model, cell_pos, Y_AXIS, 0.0, MODEL_SCALE, MARINER);

                    unsafe { rlSetPointSize(4.0) }
                    rl3d.draw_model_points_ex(&mut *placed_model, cell_pos, Y_AXIS, 0.0, MODEL_SCALE, LILAC);

                    placed_model.materials_mut()[0].maps_mut()[MATERIAL_MAP_ALBEDO as usize]
                        .texture
                        .id = cache_texture_id;
                } else {
                    let _color_guard = ColorGuard::hide(&mut placed_model.meshes_mut()[0]);
                    let cache_texture_id = placed_model.materials()[0].maps()[MATERIAL_MAP_ALBEDO as usize]
                        .texture()
                        .id;
                    placed_model.materials_mut()[0].maps_mut()[MATERIAL_MAP_ALBEDO as usize]
                        .texture
                        .id = unsafe { rlGetTextureIdDefault() };

                    rl3d.draw_model_wires_ex(
                        &mut *placed_model,
                        cell_pos,
                        Y_AXIS,
                        0.0,
                        Vector3::new(current_scale, current_scale, current_scale),
                        ANAKIWA,
                    );

                    placed_model.materials_mut()[0].maps_mut()[MATERIAL_MAP_ALBEDO as usize]
                        .texture
                        .id = cache_texture_id;
                }
            }

            draw_model_filled(
                &mut rl3d,
                &mut ndc_models[target_mesh],
                &mesh_textures[target_mesh],
                mesh_rotation,
                &view_state,
            );
            draw_model_wires_and_points(&mut rl3d, &mut ndc_models[target_mesh], mesh_rotation);
            if let Some(center) = hovered_cell_center {
                rl3d.draw_cube_wires(center, 1.0, 1.0, 1.0, NEON_CARROT);

                let hint_model = &mut ndc_models[target_mesh];
                let _color_guard = ColorGuard::hide(&mut hint_model.meshes_mut()[0]);
                let cache_texture_id = hint_model.materials()[0].maps()[MATERIAL_MAP_ALBEDO as usize]
                    .texture()
                    .id;
                hint_model.materials_mut()[0].maps_mut()[MATERIAL_MAP_ALBEDO as usize]
                    .texture
                    .id = unsafe { rlGetTextureIdDefault() };

                let hint_color = if hovered_cell_occupied { RED_DAMASK } else { ANAKIWA };

                unsafe { ffi::rlDisableDepthTest() };

                rl3d.draw_model_wires_ex(
                    &mut *hint_model,
                    center,
                    Y_AXIS,
                    mesh_rotation.to_degrees(),
                    Vector3::new(HINT_SCALE, HINT_SCALE, HINT_SCALE),
                    hint_color,
                );

                hint_model.materials_mut()[0].maps_mut()[MATERIAL_MAP_ALBEDO as usize]
                    .texture
                    .id = cache_texture_id;
            }
            unsafe { ffi::rlEnableDepthTest() };
        });

        let screen_width = draw_handle.get_screen_width();
        let screen_height = draw_handle.get_screen_height();

        const LABEL_COL: i32 = HUD_MARGIN;
        const VALUE_COL: i32 = LABEL_COL + 200;

        let mut line_y = HUD_MARGIN;
        draw_handle.draw_text(
            match target_mesh {
                0 => "GHOST",
                1 => "CUBE",
                2 => "SPHERE",
                _ => "",
            },
            LABEL_COL + 450,
            line_y,
            FONT_SIZE,
            NEON_CARROT,
        );

        draw_handle.draw_text("JUGEMU [ P ]:", LABEL_COL, line_y, FONT_SIZE, SUNFLOWER);
        draw_handle.draw_text(
            if view_state.jugemu_ortho_mode {
                "ORTHOGRAPHIC"
            } else {
                "PERSPECTIVE"
            },
            VALUE_COL,
            line_y,
            FONT_SIZE,
            if view_state.jugemu_ortho_mode {
                BAHAMA_BLUE
            } else {
                ANAKIWA
            },
        );
        line_y += HUD_LINE_HEIGHT;
        draw_handle.draw_text("FOVY[ + - ]:", LABEL_COL, line_y, FONT_SIZE, SUNFLOWER);
        draw_handle.draw_text(&format!("{:.2}", jugemu.fovy), VALUE_COL, line_y, FONT_SIZE, LILAC);
        line_y += HUD_LINE_HEIGHT;
        draw_handle.draw_text("DISTANCE [ W S ]:", LABEL_COL, line_y, FONT_SIZE, SUNFLOWER);
        let jugemu_distance = {
            let offset = Vector3::new(
                jugemu.position.x - jugemu.target.x,
                jugemu.position.y - jugemu.target.y,
                jugemu.position.z - jugemu.target.z,
            );
            (offset.x * offset.x + offset.y * offset.y + offset.z * offset.z).sqrt()
        };
        draw_handle.draw_text(
            &format!("{:.2}", jugemu_distance),
            VALUE_COL,
            line_y,
            FONT_SIZE,
            HOPBUSH,
        );

        const RIGHT_LABEL_COL: i32 = 250;
        const RIGHT_VALUE_COL: i32 = 80;
        line_y = HUD_MARGIN;
        draw_handle.draw_text(
            "TEXTURE [ T ]:",
            screen_width - RIGHT_LABEL_COL,
            line_y,
            FONT_SIZE,
            SUNFLOWER,
        );
        draw_handle.draw_text(
            if view_state.texture_mode { "ON" } else { "OFF" },
            screen_width - RIGHT_VALUE_COL,
            line_y,
            FONT_SIZE,
            if view_state.texture_mode {
                ANAKIWA
            } else {
                CHESTNUT_ROSE
            },
        );
        line_y += HUD_LINE_HEIGHT;
        draw_handle.draw_text(
            "COLORS [ C ]:",
            screen_width - RIGHT_LABEL_COL,
            line_y,
            FONT_SIZE,
            SUNFLOWER,
        );
        draw_handle.draw_text(
            if view_state.color_mode { "ON" } else { "OFF" },
            screen_width - RIGHT_VALUE_COL,
            line_y,
            FONT_SIZE,
            if view_state.color_mode { ANAKIWA } else { CHESTNUT_ROSE },
        );

        line_y = screen_height - HUD_MARGIN - HUD_LINE_HEIGHT * 3;
        draw_handle.draw_text("ASPECT [ Q ]:", LABEL_COL, line_y, FONT_SIZE, SUNFLOWER);
        draw_handle.draw_text(
            if view_state.aspect_correct {
                "CORRECT"
            } else {
                "INCORRECT"
            },
            VALUE_COL,
            line_y,
            FONT_SIZE,
            if view_state.aspect_correct {
                ANAKIWA
            } else {
                CHESTNUT_ROSE
            },
        );
        line_y += HUD_LINE_HEIGHT;
        draw_handle.draw_text("LENS [ O ]:", LABEL_COL, line_y, FONT_SIZE, SUNFLOWER);
        draw_handle.draw_text(
            if view_state.ortho_mode {
                "ORTHOGRAPHIC"
            } else {
                "PERSPECTIVE"
            },
            VALUE_COL,
            line_y,
            FONT_SIZE,
            if view_state.ortho_mode { BAHAMA_BLUE } else { ANAKIWA },
        );
        line_y += HUD_LINE_HEIGHT;
        draw_handle.draw_text("SPACE [ N ]:", LABEL_COL, line_y, FONT_SIZE, SUNFLOWER);
        draw_handle.draw_text(
            if view_state.ndc_space { "NDC" } else { "WORLD" },
            VALUE_COL,
            line_y,
            FONT_SIZE,
            if view_state.ndc_space { BAHAMA_BLUE } else { ANAKIWA },
        );
    }
}

#[inline]
pub fn observed_line_of_sight(observer: &Camera3D) -> Vec3 {
    Vec3::new(
        observer.target.x - observer.position.x,
        observer.target.y - observer.position.y,
        observer.target.z - observer.position.z,
    )
    .normalize_or_zero()
}

fn fill_planar_texcoords(mesh: &mut WeakMesh) {
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

fn fill_vertex_colors(mesh: &mut WeakMesh) {
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

fn update_normals_for_silhouette(mesh: &mut WeakMesh) {
    let vertices = mesh.vertices();
    let mut normals = vec![Vec3::ZERO; vertices.len()];

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
}

fn fade_vertex_colors_silhouette_rim(mesh: &mut WeakMesh, observer: &Camera3D, mesh_rotation: f32) {
    let model_center_to_camera = rotate_point_about_axis(
        -1.0 * observed_line_of_sight(observer),
        (Vec3::ZERO, Vec3::Y),
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
        let alpha = fade_scalar * fade_scalar * fade_scalar * fade_scalar; //powf 4
        alpha_buffer[i] = (alpha * 255.0).round() as u8;
    }

    let colors = mesh.colors_mut().unwrap();
    for i in 0..alpha_buffer.len() {
        colors[i].a = alpha_buffer[i];
    }
}

pub fn collect_deformed_vertex_samples(base_vertices: &[Vector3]) -> Vec<Vec<Vector3>> {
    let vertices = base_vertices;
    let mut mesh_samples = Vec::with_capacity(ROTATIONAL_SAMPLES_FOR_INV_PROJ);
    for i in 0..ROTATIONAL_SAMPLES_FOR_INV_PROJ {
        let sample_time = i as f32 * TIME_BETWEEN_SAMPLES;
        let sample_rotation = -ANGULAR_VELOCITY * sample_time;
        let mut mesh_sample = vertices.to_vec();
        rotate_vertices_in_plane_slice(&mut mesh_sample, sample_rotation);
        let radial_field = generate_silhouette_radial_field(sample_time);
        deform_vertices_with_radial_field(&mut mesh_sample, &radial_field);
        rotate_vertices_in_plane_slice(&mut mesh_sample, -sample_rotation);
        mesh_samples.push(mesh_sample);
    }
    mesh_samples
}

pub fn generate_silhouette_radial_field(i_time: f32) -> Vec<f32> {
    let mut radial_field = Vec::with_capacity(RADIAL_FIELD_SIZE);
    for i in 0..RADIAL_FIELD_SIZE {
        let radial_field_angle = (i as f32) * TAU / (RADIAL_FIELD_SIZE as f32);
        radial_field.push(deformed_silhouette_radius_at_angle(radial_field_angle, i_time));
    }
    let max_radius = radial_field.iter().cloned().fold(1e-6, f32::max);
    for radius in &mut radial_field {
        *radius /= max_radius;
    }
    radial_field
}

pub fn deform_vertices_with_radial_field(vertices: &mut [Vector3], radial_field: &[f32]) {
    if vertices.is_empty() {
        return;
    }
    for vertex in vertices.iter_mut() {
        let interpolated_radial_magnitude = interpolate_between_radial_field_elements(vertex.x, vertex.y, radial_field);
        vertex.x *= interpolated_radial_magnitude;
        vertex.y *= interpolated_radial_magnitude;
    }
}

pub fn interpolate_between_deformed_vertices(model: &mut Model, i_time: f32, vertex_samples: &[Vec<Vector3>]) {
    let target_mesh = &mut model.meshes_mut()[0];
    let duration = vertex_samples.len() as f32 * TIME_BETWEEN_SAMPLES;
    let time = i_time % duration;
    let frame = time / TIME_BETWEEN_SAMPLES;
    let current_frame = frame.floor() as usize % vertex_samples.len();
    let next_frame = (current_frame + 1) % vertex_samples.len();
    let weight = frame.fract();
    let vertices = target_mesh.vertices_mut();
    for ((dst_vertex, src_vertex), next_vertex) in vertices
        .iter_mut()
        .zip(vertex_samples[current_frame].iter())
        .zip(vertex_samples[next_frame].iter())
    {
        dst_vertex.x = src_vertex.x * (1.0 - weight) + next_vertex.x * weight;
        dst_vertex.y = src_vertex.y * (1.0 - weight) + next_vertex.y * weight;
        dst_vertex.z = src_vertex.z * (1.0 - weight) + next_vertex.z * weight;
    }
}

pub fn interpolate_between_radial_field_elements(sample_x: f32, sample_y: f32, radial_field: &[f32]) -> f32 {
    let radial_disk_angle = sample_y.atan2(sample_x).rem_euclid(TAU);
    let radial_index = radial_disk_angle / TAU * RADIAL_FIELD_SIZE as f32;
    let lower_index = radial_index.floor() as usize % RADIAL_FIELD_SIZE;
    let upper_index = (lower_index + 1) % RADIAL_FIELD_SIZE;
    let interpolation_toward_upper = radial_index.fract();
    radial_field[lower_index] * (1.0 - interpolation_toward_upper)
        + radial_field[upper_index] * interpolation_toward_upper
}

#[inline]
pub fn deformed_silhouette_radius_at_angle(radial_field_angle: f32, i_time: f32) -> f32 {
    let direction_vector = Vector2::new(radial_field_angle.cos(), radial_field_angle.sin());
    let phase = LIGHT_WAVE_AMPLITUDE_X.hypot(LIGHT_WAVE_AMPLITUDE_Y) + 2.0;
    let mut lower_phase_radius = 0.0_f32;
    let mut upper_phase_radius = UMBRAL_MASK_OUTER_RADIUS + phase;
    for _ in 0..8 {
        let current_radius = grid_phase_magnitude(
            &mut (UMBRAL_MASK_CENTER + direction_vector * upper_phase_radius),
            i_time,
        );
        if current_radius >= UMBRAL_MASK_OUTER_RADIUS {
            break;
        }
        upper_phase_radius *= 1.5;
    }
    for _ in 0..20 {
        let mid_phase_radius = 0.5 * (lower_phase_radius + upper_phase_radius);
        let current_radius =
            grid_phase_magnitude(&mut (UMBRAL_MASK_CENTER + direction_vector * mid_phase_radius), i_time);
        if current_radius >= UMBRAL_MASK_OUTER_RADIUS {
            upper_phase_radius = mid_phase_radius;
        } else {
            lower_phase_radius = mid_phase_radius;
        }
    }
    upper_phase_radius
}

#[inline]
pub fn grid_phase_magnitude(grid_coord: &mut Vector2, i_time: f32) -> f32 {
    let mut grid_phase = spatial_phase(*grid_coord);
    grid_phase += temporal_phase(i_time);
    *grid_coord += add_phase(grid_phase);
    grid_coord.distance(UMBRAL_MASK_CENTER)
}

#[inline]
pub fn rotate_vertices_in_plane_slice(vertices: &mut [Vector3], rotation: f32) {
    let (rotation_sin, rotation_cos) = rotation.sin_cos();
    for vertex in vertices {
        let (x0, z0) = (vertex.x, vertex.z);
        vertex.x = x0 * rotation_cos + z0 * rotation_sin;
        vertex.z = -x0 * rotation_sin + z0 * rotation_cos;
    }
}

#[inline]
pub fn spatial_phase(grid_coords: Vector2) -> Vector2 {
    Vector2::new(
        grid_coords.y * LIGHT_WAVE_SPATIAL_FREQ_X,
        grid_coords.x * LIGHT_WAVE_SPATIAL_FREQ_Y,
    )
}

#[inline]
pub fn temporal_phase(time: f32) -> Vector2 {
    Vector2::new(time * LIGHT_WAVE_TEMPORAL_FREQ_X, time * LIGHT_WAVE_TEMPORAL_FREQ_Y)
}

#[inline]
pub fn add_phase(phase: Vector2) -> Vector2 {
    Vector2::new(
        LIGHT_WAVE_AMPLITUDE_X * phase.x.cos(),
        LIGHT_WAVE_AMPLITUDE_Y * phase.y.sin(),
    )
}

fn orbit_space(handle: &mut RaylibHandle, camera: &mut Camera3D) {
    let dt = handle.get_frame_time();

    let offset = Vector3::new(
        camera.position.x - camera.target.x,
        camera.position.y - camera.target.y,
        camera.position.z - camera.target.z,
    );

    let mut radius = (offset.x * offset.x + offset.y * offset.y + offset.z * offset.z).sqrt();
    let mut azimuth = offset.z.atan2(offset.x);
    let horizontal_radius = (offset.x * offset.x + offset.z * offset.z).sqrt();
    let mut elevation = offset.y.atan2(horizontal_radius);

    if handle.is_key_down(KeyboardKey::KEY_LEFT) {
        azimuth += 1.0 * dt;
    }
    if handle.is_key_down(KeyboardKey::KEY_RIGHT) {
        azimuth -= 1.0 * dt;
    }
    if handle.is_key_down(KeyboardKey::KEY_UP) {
        elevation += 1.0 * dt;
    }
    if handle.is_key_down(KeyboardKey::KEY_DOWN) {
        elevation -= 1.0 * dt;
    }

    if handle.is_key_down(KeyboardKey::KEY_W) {
        radius -= 1.0 * dt;
    }
    if handle.is_key_down(KeyboardKey::KEY_S) {
        radius += 1.0 * dt;
    }
    radius = radius.clamp(0.25, 10.0);

    if camera.projection == CAMERA_ORTHOGRAPHIC {
        if handle.is_key_down(KeyboardKey::KEY_SEMICOLON) {
            camera.fovy += 6.0 * dt;
        }
        if handle.is_key_down(KeyboardKey::KEY_MINUS) {
            camera.fovy -= 6.0 * dt;
        }
        camera.fovy = camera.fovy.clamp(1.0, 30.0);
    } else {
        if handle.is_key_down(KeyboardKey::KEY_SEMICOLON) {
            camera.fovy += 35.0 * dt;
        }
        if handle.is_key_down(KeyboardKey::KEY_MINUS) {
            camera.fovy -= 35.0 * dt;
        }
        camera.fovy = camera.fovy.clamp(1.0, 130.0);
    }

    elevation = elevation.clamp(-PI / 2.0 + 0.1, PI / 2.0 - 0.1);

    camera.position.x = camera.target.x + radius * elevation.cos() * azimuth.cos();
    camera.position.y = camera.target.y + radius * elevation.sin();
    camera.position.z = camera.target.z + radius * elevation.cos() * azimuth.sin();
}

#[inline]
pub fn triangle_normal(a: Vec3, b: Vec3, c: Vec3) -> Vec3 {
    (b - a).cross(c - a).normalize_or_zero()
}

#[inline]
pub fn rotate_point_about_axis(c: Vec3, axis: (Vec3, Vec3), theta: f32) -> Vec3 {
    let (a, b) = axis;
    let ab = b - a;
    let ab_axis_dir = ab.normalize_or_zero();
    let ac = c - a;
    let ac_z_component = ab_axis_dir.dot(ac) * ab_axis_dir;
    let ac_x_component = ac - ac_z_component;
    let ac_y_component = ab_axis_dir.cross(ac_x_component);
    let origin = a;
    let rotated_x_component = ac_x_component * theta.cos();
    let rotated_y_component = ac_y_component * theta.sin();
    let rotated_c = rotated_x_component + rotated_y_component + ac_z_component;
    origin + rotated_c
}

fn basis_vector(main: &Camera3D) -> (Vector3, Vector3, Vector3) {
    let depth = main.target.sub(main.position).normalize();
    let right = depth.cross(main.up).normalize();
    let up = right.cross(depth).normalize();
    (depth, right, up)
}
fn world_to_ndc_space(
    main: &mut Camera3D,
    aspect: f32,
    near: f32,
    far: f32,
    world: &Model,
    ndc: &mut Model,
    rotation: f32,
    ortho_factor: f32,
    aspect_factor: f32,
) {
    let (depth, right, up) = basis_vector(&main);
    let half_h_near = lerp(
        near * (FOVY_PERSPECTIVE * 0.5).to_radians().tan(),
        0.5 * NEAR_PLANE_HEIGHT_ORTHOGRAPHIC(),
        ortho_factor,
    );
    let half_w_near = lerp(half_h_near, half_h_near * aspect, aspect_factor);
    let half_depth_ndc = lerp(half_h_near, 0.5 * (far - near), lerp(aspect_factor, 0.0, ortho_factor));
    let center_near_plane = main.position.add(depth * near);
    let center_ndc_cube = center_near_plane.add(depth * half_depth_ndc);

    let world_mesh = &world.meshes()[0];
    let ndc_mesh = &mut ndc.meshes_mut()[0];
    let world_vertices = world_mesh.vertices();
    let ndc_vertices = ndc_mesh.vertices_mut();

    for [a, b, c] in world_mesh.triangles() {
        for &i in [a, b, c].iter() {
            let world_vertex = translate_rotate_scale(0, world_vertices[i], MODEL_POS, MODEL_SCALE, rotation);
            let signed_depth = world_vertex.sub(main.position).dot(depth);

            let intersection_coord = intersect(main, near, world_vertex, ortho_factor);
            let clip_plane_vector = intersection_coord.sub(center_near_plane);
            let x_ndc = clip_plane_vector.dot(right) / half_w_near;
            let y_ndc = clip_plane_vector.dot(up) / half_h_near;
            let z_ndc = lerp(
                (far + near - 2.0 * far * near / signed_depth) / (far - near),
                2.0 * (signed_depth - near) / (far - near) - 1.0,
                ortho_factor,
            );
            let scaled_right = right * (x_ndc * half_w_near);
            let scaled_up = up * (y_ndc * half_h_near);
            let scaled_depth = depth * (z_ndc * half_depth_ndc);
            let offset = scaled_right.add(scaled_up).add(scaled_depth);
            let scaled_ndc_coord = center_ndc_cube.add(offset);
            ndc_vertices[i] = translate_rotate_scale(1, scaled_ndc_coord, MODEL_POS, MODEL_SCALE, rotation);
        }
    }
}

fn calculate_average_ndc_z_shift(world_model: &Model, ndc_model: &Model) -> f32 {
    let world_vertices = world_model.meshes()[0].vertices();
    let ndc_vertices = ndc_model.meshes()[0].vertices();

    let mut total_z_shift = 0.0;
    let mut count = 0;

    for (world_v, ndc_v) in world_vertices.iter().zip(ndc_vertices.iter()) {
        total_z_shift += ndc_v.z - world_v.z;
        count += 1;
    }

    if count > 0 {
        total_z_shift / count as f32
    } else {
        0.0
    }
}

fn draw_model_filled(
    rl3d: &mut RaylibMode3D<RaylibDrawHandle>,
    model: &mut Model,
    mesh_texture: &Texture2D,
    rotation: f32,
    view_state: &ViewState,
) {
    if !(view_state.color_mode || view_state.texture_mode) {
        return;
    }
    let _color_guard = if view_state.texture_mode && !view_state.color_mode {
        Some(ColorGuard::hide(&mut model.meshes_mut()[0]))
    } else {
        None
    };

    model.materials_mut()[0].maps_mut()[MATERIAL_MAP_ALBEDO as usize]
        .texture
        .id = if view_state.texture_mode {
        mesh_texture.id
    } else {
        unsafe { rlGetTextureIdDefault() }
    };

    rl3d.draw_model_ex(
        &mut *model,
        MODEL_POS,
        Y_AXIS,
        rotation.to_degrees(),
        MODEL_SCALE,
        Color::WHITE,
    );
    model.materials_mut()[0].maps_mut()[MATERIAL_MAP_ALBEDO as usize]
        .texture
        .id = unsafe { rlGetTextureIdDefault() };
}

fn update_spatial_frame(
    main: &mut Camera3D,
    aspect: f32,
    near: f32,
    far: f32,
    spatial_frame: &mut WeakMesh,
    space_factor: f32,
    aspect_factor: f32,
    ortho_factor: f32,
) {
    let (depth, right, up) = basis_vector(&main);
    let half_h_near = lerp(
        near * (FOVY_PERSPECTIVE * 0.5).to_radians().tan(),
        0.5 * NEAR_PLANE_HEIGHT_ORTHOGRAPHIC(),
        ortho_factor,
    );
    let half_w_near = lerp(half_h_near, half_h_near * aspect, aspect_factor);
    let half_h_far = lerp(
        far * (FOVY_PERSPECTIVE * 0.5).to_radians().tan(),
        0.5 * NEAR_PLANE_HEIGHT_ORTHOGRAPHIC(),
        ortho_factor,
    );
    let half_w_far = lerp(half_h_far, half_h_far * aspect, aspect_factor);
    let half_depth_ndc = lerp(half_h_near, 0.5 * (far - near), lerp(aspect_factor, 0.0, ortho_factor));
    let half_depth = lerp(0.5 * (far - near), half_depth_ndc, space_factor);
    let far_half_w = lerp(half_w_far, half_w_near, space_factor);
    let far_half_h = lerp(half_h_far, half_h_near, space_factor);
    let center_near = main.position.add(depth * near);

    let spatial_frame_triangles = spatial_frame.triangles().collect::<Vec<[usize; 3]>>();
    let spatial_frame_vertices_snapshot = spatial_frame.vertices().to_vec();
    let spatial_frame_vertices = spatial_frame.vertices_mut();

    for [a, b, c] in spatial_frame_triangles {
        for &i in [a, b, c].iter() {
            let offset = spatial_frame_vertices_snapshot[i].sub(center_near);
            let x_sign = if offset.dot(right) >= 0.0 { 1.0 } else { -1.0 };
            let y_sign = if offset.dot(up) >= 0.0 { 1.0 } else { -1.0 };
            let far_mask = if offset.dot(depth) > half_depth { 1.0 } else { 0.0 };
            let final_half_w = half_w_near + far_mask * (far_half_w - half_w_near);
            let final_half_h = half_h_near + far_mask * (far_half_h - half_h_near);
            let center = center_near.add(depth * (far_mask * 2.0 * half_depth));
            spatial_frame_vertices[i] = center
                .add(right * (x_sign * final_half_w))
                .add(up * (y_sign * final_half_h));
        }
    }
}

fn draw_model_wires_and_points(rl3d: &mut RaylibMode3D<RaylibDrawHandle>, model: &mut Model, rotation: f32) {
    let _color_guard = ColorGuard::hide(&mut model.meshes_mut()[0]);
    let cache_texture_id = model.materials()[0].maps()[MATERIAL_MAP_ALBEDO as usize].texture().id;
    model.materials_mut()[0].maps_mut()[MATERIAL_MAP_ALBEDO as usize]
        .texture
        .id = unsafe { rlGetTextureIdDefault() };
    rl3d.draw_model_wires_ex(
        &mut *model,
        MODEL_POS,
        Y_AXIS,
        rotation.to_degrees(),
        MODEL_SCALE,
        MARINER,
    );
    unsafe { rlSetPointSize(4.0) }
    rl3d.draw_model_points_ex(
        &mut *model,
        MODEL_POS,
        Y_AXIS,
        rotation.to_degrees(),
        MODEL_SCALE,
        LILAC,
    );

    model.materials_mut()[0].maps_mut()[MATERIAL_MAP_ALBEDO as usize]
        .texture
        .id = cache_texture_id;
} // NOTE!!! Colors automatically restored when _color_guard drops!!!

fn draw_spatial_frame(rl3d: &mut RaylibMode3D<RaylibDrawHandle>, spatial_frame: &WeakMesh) {
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

fn aspect_correct_and_reflect_near_plane(
    intersect: Vector3,
    center: Vector3,
    right: Vector3,
    up: Vector3,
    x_aspect: f32,
    y_reflect: f32,
) -> Vector3 {
    let center_distance = intersect.sub(center);
    let x = center_distance.dot(right);
    let y = center_distance.dot(up);
    center.add(right * (x * x_aspect)).add(up * (y * y_reflect))
}

fn translate_rotate_scale(inverse: i32, coord: Vector3, pos: Vector3, scale: Vector3, rotation: f32) -> Vector3 {
    let matrix = Mat4::from_scale(scale) * Mat4::from_rotation_y(rotation) * Mat4::from_translation(pos);
    let result = if inverse != 0 { matrix.inverse() } else { matrix };
    result.transform_point3(coord)
}
fn intersect(main: &mut Camera3D, near: f32, world_coord: Vector3, ortho_factor: f32) -> Vector3 {
    let view_dir = main.target.sub(main.position).normalize();
    let main_camera_to_point = world_coord.sub(main.position);
    let depth_along_view = main_camera_to_point.dot(view_dir);
    let center_near_plane = main.position.add(view_dir * near);
    if depth_along_view <= 0.0 {
        return center_near_plane;
    }
    let scale_to_near = near / depth_along_view;
    let result_perspective = main.position.add(main_camera_to_point * scale_to_near);
    let result_ortho = world_coord.add(view_dir * (center_near_plane.sub(world_coord).dot(view_dir)));
    Vector3::new(
        result_perspective.x + (result_ortho.x - result_perspective.x) * ortho_factor,
        result_perspective.y + (result_ortho.y - result_perspective.y) * ortho_factor,
        result_perspective.z + (result_ortho.z - result_perspective.z) * ortho_factor,
    )
}

fn space_blend_factor(dt: f32, ndc_space: bool) -> f32 {
    static mut BLEND: f32 = 0.0;
    unsafe {
        if dt > 0.0 {
            BLEND = (BLEND + (if ndc_space { 1.0 } else { -1.0 }) * BLEND_SCALAR * dt).clamp(0.0, 1.0);
        }
        BLEND
    }
}

fn aspect_blend_factor(dt: f32, aspect_correct: bool) -> f32 {
    static mut BLEND: f32 = 0.0;
    unsafe {
        if dt > 0.0 {
            BLEND = (BLEND + (if aspect_correct { 1.0 } else { -1.0 }) * BLEND_SCALAR * dt).clamp(0.0, 1.0);
        }
        BLEND
    }
}

fn ortho_blend_factor(dt: f32, ortho_mode: bool) -> f32 {
    static mut BLEND: f32 = 0.0;
    unsafe {
        if dt > 0.0 {
            BLEND = (BLEND + (if ortho_mode { 1.0 } else { -1.0 }) * BLEND_SCALAR * dt).clamp(0.0, 1.0);
        }
        BLEND
    }
}

fn draw_room_floor_grid(rl3d: &mut RaylibMode3D<RaylibDrawHandle>) {
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

fn get_hovered_room_floor_cell(handle: &RaylibHandle, camera: &Camera3D) -> Option<(i32, i32, i32)> {
    let mouse = handle.get_mouse_position();
    let ray = handle.get_screen_to_world_ray(mouse, *camera);

    if ray.direction.y.abs() < 1e-5 {
        return None;
    }

    let floor_y = -(ROOM_H as f32) / 2.0;
    let t = (floor_y - ray.position.y) / ray.direction.y;
    if t <= 0.0 {
        return None;
    }

    let hit = Vector3::new(
        ray.position.x + ray.direction.x * t,
        floor_y,
        ray.position.z + ray.direction.z * t,
    );

    let origin = room_origin();
    let local_x = hit.x - origin.x;
    let local_z = hit.z - origin.z;

    if local_x < 0.0 || local_z < 0.0 || local_x >= ROOM_W as f32 || local_z >= ROOM_D as f32 {
        return None;
    }

    let cell_x = local_x.floor() as i32;
    let cell_z = local_z.floor() as i32;
    let cell_y = 0;

    Some((cell_x, cell_y, cell_z))
}

fn cell_center(ix: i32, iy: i32, iz: i32) -> Vector3 {
    let origin = room_origin();
    Vector3::new(
        origin.x + ix as f32 + 0.5,
        origin.y + iy as f32 + 0.5,
        origin.z + iz as f32 + 0.5,
    )
}

fn room_origin() -> Vector3 {
    Vector3::new(-(ROOM_W as f32) / 2.0, -(ROOM_H as f32) / 2.0, -(ROOM_D as f32) / 2.0)
}
