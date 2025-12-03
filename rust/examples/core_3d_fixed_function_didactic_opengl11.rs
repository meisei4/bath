extern crate core;

use raylib::core::math::Vector3;
use raylib::models::{Mesh, Model, RaylibMaterial, RaylibMesh, RaylibModel, WeakMesh};
use raylib::{ffi, RaylibHandle, RaylibThread};

use asset_payload::{FUSUMA_MIN_PATH, SPHERE_PATH, WINDOW_MIN_PATH};
use raylib::camera::Camera3D;
use raylib::color::Color;
use raylib::consts::CameraProjection::{CAMERA_ORTHOGRAPHIC, CAMERA_PERSPECTIVE};
use raylib::consts::KeyboardKey;
use raylib::consts::MaterialMapIndex::MATERIAL_MAP_ALBEDO;
use raylib::consts::PixelFormat::{PIXELFORMAT_UNCOMPRESSED_GRAYSCALE, PIXELFORMAT_UNCOMPRESSED_R8G8B8A8};
use raylib::drawing::{RaylibDraw, RaylibDraw3D, RaylibDrawHandle, RaylibMode3D, RaylibMode3DExt};
use raylib::ffi::{
    rlBegin, rlColor4ub, rlDisableTexture, rlDisableWireMode, rlDrawRenderBatchActive, rlEnableTexture,
    rlEnableWireMode, rlEnd, rlGetTextureIdDefault, rlSetPointSize, rlSetTexture, rlTexCoord2f, rlVertex3f,
    IsKeyPressed, RL_TRIANGLES,
};
use raylib::math::glam::Mat4;
use raylib::math::{lerp, Vector2};
use raylib::texture::{Image, RaylibTexture2D, Texture2D};
use std::f32::consts::PI;
use std::mem::replace;
use std::ops::{Add, Sub};

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

const FLAG_NDC: u32 = 1u32 << 0;
const FLAG_REFLECT_Y: u32 = 1u32 << 1;
const FLAG_ASPECT: u32 = 1u32 << 2;
const FLAG_PERSPECTIVE_CORRECT: u32 = 1u32 << 3;
const FLAG_PAUSE: u32 = 1u32 << 4;
const FLAG_COLOR_MODE: u32 = 1u32 << 5;
const FLAG_TEXTURE_MODE: u32 = 1u32 << 6;
const FLAG_JUGEMU: u32 = 1u32 << 7;
const FLAG_ORTHO: u32 = 1u32 << 8;
const FLAG_CLIP: u32 = 1u32 << 9;
const GEN_CUBE: u32 = 1u32 << 10;
const LOAD_CUBE: u32 = 1u32 << 11;
const GEN_SPHERE: u32 = 1u32 << 12;
const LOAD_SPHERE: u32 = 1u32 << 13;
const GEN_KNOT: u32 = 1u32 << 14;

static mut GFLAGS: u32 = FLAG_ASPECT | FLAG_COLOR_MODE | FLAG_JUGEMU | GEN_CUBE;
macro_rules! ndc_space {
    () => {
        (unsafe { GFLAGS } & FLAG_NDC) != 0
    };
}
macro_rules! reflect_y {
    () => {
        (unsafe { GFLAGS } & FLAG_REFLECT_Y) != 0
    };
}
macro_rules! aspect_correct {
    () => {
        (unsafe { GFLAGS } & FLAG_ASPECT) != 0
    };
}
macro_rules! perspective_correct {
    () => {
        (unsafe { GFLAGS } & FLAG_PERSPECTIVE_CORRECT) != 0
    };
}
macro_rules! paused {
    () => {
        (unsafe { GFLAGS } & FLAG_PAUSE) != 0
    };
}
macro_rules! color_mode {
    () => {
        (unsafe { GFLAGS } & FLAG_COLOR_MODE) != 0
    };
}
macro_rules! texture_mode {
    () => {
        (unsafe { GFLAGS } & FLAG_TEXTURE_MODE) != 0
    };
}
macro_rules! jugemu_mode {
    () => {
        (unsafe { GFLAGS } & FLAG_JUGEMU) != 0
    };
}
macro_rules! ortho_mode {
    () => {
        (unsafe { GFLAGS } & FLAG_ORTHO) != 0
    };
}
macro_rules! clip_mode {
    () => {
        (unsafe { GFLAGS } & FLAG_CLIP) != 0
    };
}
macro_rules! toggle {
    ($k:expr, $f:expr) => {{
        unsafe {
            if IsKeyPressed($k as i32) {
                GFLAGS ^= $f;
            }
        }
    }};
}

const NUM_MODELS: usize = 5;
static mut TARGET_MESH_INDEX: usize = 0;
macro_rules! cycle_mesh {
    ($k:expr, $i:expr, $f:expr) => {{
        unsafe {
            if IsKeyPressed($k as i32) {
                TARGET_MESH_INDEX = $i;
                GFLAGS = (GFLAGS & !(GEN_CUBE | LOAD_CUBE | GEN_SPHERE | LOAD_SPHERE | GEN_KNOT)) | ($f);
            }
        }
    }};
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

const FOVY_PERSPECTIVE: f32 = 60.0;
fn NEAR_PLANE_HEIGHT_ORTHOGRAPHIC() -> f32 {
    2.0 * (FOVY_PERSPECTIVE * 0.5).to_radians().tan()
}
const BLEND_SCALAR: f32 = 5.0;

const Y_AXIS: Vector3 = Vector3::new(0.0, 1.0, 0.0);
const MODEL_POS: Vector3 = Vector3::new(0.0, 0.0, 0.0);
const MODEL_SCALE: Vector3 = Vector3::new(1.0, 1.0, 1.0);
const MAIN_POS: Vector3 = Vector3::new(0.0, 0.0, 2.0);
const JUGEMU_POS_ISO: Vector3 = Vector3::new(3.0, 1.0, 3.0);

//------------------------------------------------------------------------------------
// Program main entry point
//------------------------------------------------------------------------------------
fn main() {
    // Initialization
    //--------------------------------------------------------------------------------------
    const FONT_SIZE: i32 = 20;
    const ANGULAR_VELOCITY: f32 = 1.25;

    let (mut handle, thread) = raylib::init()
        .size(800, 450)
        .title("raylib [core] example - fixed function didactic")
        .build();
    let mut perspective_correct_texture = Texture2D::default();

    let near = 1.0;
    let far = 3.0;
    let mut aspect = handle.get_screen_width() as f32 / handle.get_screen_height() as f32;
    let mut mesh_rotation = 0.0;

    let mut main = Camera3D {
        position: MAIN_POS,
        target: MODEL_POS,
        up: Y_AXIS,
        fovy: if ortho_mode!() {
            NEAR_PLANE_HEIGHT_ORTHOGRAPHIC()
        } else {
            FOVY_PERSPECTIVE
        },
        projection: if ortho_mode!() {
            CAMERA_ORTHOGRAPHIC
        } else {
            CAMERA_PERSPECTIVE
        },
    };

    let mut jugemu = Camera3D {
        position: JUGEMU_POS_ISO,
        target: MODEL_POS,
        up: Y_AXIS,
        fovy: FOVY_PERSPECTIVE,
        projection: CAMERA_PERSPECTIVE,
    };

    let mut world_models: Vec<Model> = Vec::new();
    let mut ndc_models: Vec<Model> = Vec::new();
    let mut near_plane_points_models: Vec<Model> = Vec::new();
    let mut mesh_textures = Vec::new();
    let texture_config: [i32; NUM_MODELS] = [4, 4, 16, 16, 32];

    for i in 0..NUM_MODELS {
        let mut world_model = match i {
            0 => handle
                .load_model_from_mesh(&thread, Mesh::try_gen_mesh_cube(&thread, 1.0, 1.0, 1.0).unwrap())
                .unwrap(),
            // 1 => handle.load_model(&thread, CUBE_PATH).expect("load cube obj"), //TODO: REQUIRES LATEST RAYLIB_SYS PIN!!
            1 => handle.load_model(&thread, FUSUMA_MIN_PATH).expect("load fusuma obj"), //TODO: REQUIRES LATEST RAYLIB_SYS PIN!!
            2 => handle.load_model(&thread, WINDOW_MIN_PATH).expect("load window obj"), //TODO: REQUIRES LATEST RAYLIB_SYS PIN!!
            // 2 => handle
            //     .load_model_from_mesh(&thread, Mesh::try_gen_mesh_sphere(&thread, 0.5, 8, 8).unwrap())
            //     .expect("load model sphere gen"),
            3 => handle.load_model(&thread, SPHERE_PATH).expect("load sphere obj"),
            _ => handle
                .load_model_from_mesh(&thread, Mesh::try_gen_mesh_knot(&thread, 1.0, 1.0, 16, 128).unwrap())
                .expect("load model knot gen"),
        };

        let world_mesh = &mut world_model.meshes_mut()[0];
        fill_planar_texcoords(world_mesh);
        fill_vertex_colors(world_mesh);

        let checked_img =
            Image::gen_image_checked(texture_config[i], texture_config[i], 1, 1, Color::BLACK, Color::WHITE);
        let mesh_texture = handle.load_texture_from_image(&thread, &checked_img).unwrap();
        world_model.materials_mut()[0].set_material_texture(MATERIAL_MAP_ALBEDO, &mesh_texture);

        let ndc_mesh = {
            let world_mesh = &world_model.meshes()[0]; //NOTE: this is an example of immutable borrows simple in these scopes
            Mesh::init_mesh(world_mesh.vertices())
                .texcoords_opt(world_mesh.texcoords())
                .colors_opt(world_mesh.colors())
                .indices_opt(world_mesh.indices())
                .build_dynamic(&thread)
                .unwrap()
        };
        let mut ndc_model = handle.load_model_from_mesh(&thread, ndc_mesh).unwrap();
        ndc_model.materials_mut()[0].set_material_texture(MATERIAL_MAP_ALBEDO, &mesh_texture);
        let near_plane_points_mesh =
            Mesh::init_mesh(&vec![Vector3::default(); &world_model.meshes()[0].triangle_count() * 3]) // NOTE: again, with the cooorrrrnneerrsss
                .build_dynamic(&thread)
                .unwrap();

        let near_plane_points_model = handle.load_model_from_mesh(&thread, near_plane_points_mesh).unwrap();
        world_models.push(world_model);
        ndc_models.push(ndc_model);
        near_plane_points_models.push(near_plane_points_model);
        mesh_textures.push(mesh_texture);
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

        let mut model = handle.load_model_from_mesh(&thread, spatial_frame_mesh).unwrap();
        model
    }; // NOTE: see! `temp_cube` gets Unloaded here for real! wow, Weak vs non Weak is really clicking
    handle.set_target_fps(60);
    //--------------------------------------------------------------------------------------

    while !handle.window_should_close() {
        // Update
        //----------------------------------------------------------------------------------
        aspect = handle.get_screen_width() as f32 / handle.get_screen_height() as f32;

        toggle!(KeyboardKey::KEY_N, FLAG_NDC);
        if ndc_space!() {
            toggle!(KeyboardKey::KEY_F, FLAG_REFLECT_Y);
        }
        toggle!(KeyboardKey::KEY_Q, FLAG_ASPECT);
        toggle!(KeyboardKey::KEY_P, FLAG_PERSPECTIVE_CORRECT);
        toggle!(KeyboardKey::KEY_SPACE, FLAG_PAUSE);
        toggle!(KeyboardKey::KEY_C, FLAG_COLOR_MODE);
        toggle!(KeyboardKey::KEY_T, FLAG_TEXTURE_MODE);
        toggle!(KeyboardKey::KEY_J, FLAG_JUGEMU);
        toggle!(KeyboardKey::KEY_O, FLAG_ORTHO);
        toggle!(KeyboardKey::KEY_X, FLAG_CLIP);

        cycle_mesh!(KeyboardKey::KEY_ONE as i32, 0, GEN_CUBE);
        cycle_mesh!(KeyboardKey::KEY_TWO as i32, 1, LOAD_CUBE);
        cycle_mesh!(KeyboardKey::KEY_THREE as i32, 2, GEN_SPHERE);
        cycle_mesh!(KeyboardKey::KEY_FOUR as i32, 3, LOAD_SPHERE);
        cycle_mesh!(KeyboardKey::KEY_FIVE as i32, 4, GEN_KNOT);

        let s_blend = space_blend_factor(handle.get_frame_time());
        aspect_blend_factor(handle.get_frame_time());
        reflect_blend_factor(handle.get_frame_time());
        ortho_blend_factor(handle.get_frame_time());

        if !paused!() {
            mesh_rotation -= ANGULAR_VELOCITY * handle.get_frame_time();
        }

        orbit_space(&mut handle, &mut jugemu);

        main.projection = if ortho_mode!() {
            CAMERA_ORTHOGRAPHIC
        } else {
            CAMERA_PERSPECTIVE
        };
        main.fovy = if ortho_mode!() {
            NEAR_PLANE_HEIGHT_ORTHOGRAPHIC()
        } else {
            FOVY_PERSPECTIVE
        };
        let target_mesh = unsafe { TARGET_MESH_INDEX };
        world_to_ndc_space(
            &mut main,
            aspect,
            near,
            far,
            &mut world_models[target_mesh],
            &mut ndc_models[target_mesh],
            mesh_rotation,
        );
        {
            let world_mesh = &world_models[target_mesh].meshes()[0];
            let ndc_mesh = &mut ndc_models[target_mesh].meshes_mut()[0];
            let world_vertices = world_mesh.vertices();
            let ndc_vertices = ndc_mesh.vertices_mut();
            //NOTE: investigate funky animations during indexed meshes vs unindexed. Indexed meshes are like not all moved together, where as unindexed everything moves together
            for [a, b, c] in world_mesh.triangles() {
                for &i in [a, b, c].iter() {
                    ndc_vertices[i].x = lerp(world_vertices[i].x, ndc_vertices[i].x, s_blend);
                    ndc_vertices[i].y = lerp(world_vertices[i].y, ndc_vertices[i].y, s_blend);
                    ndc_vertices[i].z = lerp(world_vertices[i].z, ndc_vertices[i].z, s_blend);
                }
            }
        }
        unsafe {
            ndc_models[target_mesh].meshes_mut()[0].update_position_buffer(&thread);
        }

        let display_model = &mut ndc_models[target_mesh];

        update_spatial_frame(&mut main, aspect, near, far, &mut spatial_frame_model.meshes_mut()[0]);
        unsafe {
            spatial_frame_model.meshes_mut()[0].update_position_buffer(&thread);
        }

        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let (depth, right, up) = basis_vector(&main);
        let mut draw_handle = handle.begin_drawing(&thread);
        if perspective_correct!() && texture_mode!() {
            perspective_correct_capture(
                &thread,
                &mut draw_handle,
                &mut main,
                display_model,
                &mesh_textures[target_mesh],
                &mut perspective_correct_texture,
                mesh_rotation,
            );
        }
        draw_handle.clear_background(Color::BLACK);
        draw_handle.draw_mode3D(if jugemu_mode!() { jugemu } else { main }, |mut rl3d| {
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

            if jugemu_mode!() {
                draw_spatial_frame(&mut rl3d, &spatial_frame_model.meshes_mut()[0]);
            }
            draw_model_filled(
                &thread,
                &mut rl3d,
                display_model,
                &mesh_textures[target_mesh],
                mesh_rotation,
            );
            draw_model_wires_and_points(&thread, &mut rl3d, display_model, mesh_rotation);

            if jugemu_mode!() {
                draw_near_plane_points(
                    &thread,
                    &mut rl3d,
                    &mut main,
                    aspect,
                    near,
                    &mut near_plane_points_models[target_mesh],
                    &display_model,
                    mesh_rotation,
                );
            }

            if perspective_correct!() && texture_mode!() {
                {
                    spatial_frame_model.materials_mut()[0]
                        .set_material_texture(MATERIAL_MAP_ALBEDO, &perspective_correct_texture);
                    if jugemu_mode!() {
                        rl3d.draw_model(&spatial_frame_model, MODEL_POS, 1.0, Color::WHITE);
                    }
                }
            } else {
                if jugemu_mode!() {
                    perspective_incorrect_capture(
                        &mut main,
                        aspect,
                        near,
                        &display_model,
                        &mesh_textures[target_mesh],
                        mesh_rotation,
                    );
                }
            }
        });

        draw_handle.draw_text("[1-2]: CUBE [3-4]: SPHERE [5]: KNOT", 12, 12, FONT_SIZE, NEON_CARROT);
        draw_handle.draw_text("ARROWS: MOVE | SPACEBAR: PAUSE", 12, 38, FONT_SIZE, NEON_CARROT);
        draw_handle.draw_text("W A : ZOOM", 12, 64, FONT_SIZE, NEON_CARROT);
        draw_handle.draw_text("CLIP [ X ]:", 12, 94, FONT_SIZE, SUNFLOWER);
        draw_handle.draw_text(
            if clip_mode!() { "ON" } else { "OFF" },
            120,
            94,
            FONT_SIZE,
            if clip_mode!() { BAHAMA_BLUE } else { ANAKIWA },
        );
        draw_handle.draw_text(
            match target_mesh {
                0 => "GEN_CUBE",
                1 => "LOAD_CUBE",
                2 => "GEN_SPHERE",
                3 => "LOAD_SPHERE",
                _ => "GEN_KNOT",
            },
            12,
            205,
            FONT_SIZE,
            NEON_CARROT,
        );
        draw_handle.draw_text("TEXTURE [ T ]:", 570, 12, FONT_SIZE, SUNFLOWER);
        draw_handle.draw_text(
            if texture_mode!() { "ON" } else { "OFF" },
            740,
            12,
            FONT_SIZE,
            if texture_mode!() { ANAKIWA } else { CHESTNUT_ROSE },
        );
        draw_handle.draw_text("COLORS [ C ]:", 570, 38, FONT_SIZE, SUNFLOWER);
        draw_handle.draw_text(
            if color_mode!() { "ON" } else { "OFF" },
            740,
            38,
            FONT_SIZE,
            if color_mode!() { ANAKIWA } else { CHESTNUT_ROSE },
        );
        draw_handle.draw_text("ASPECT [ Q ]:", 12, 392, FONT_SIZE, SUNFLOWER);
        draw_handle.draw_text(
            if aspect_correct!() { "CORRECT" } else { "INCORRECT" },
            230,
            392,
            FONT_SIZE,
            if aspect_correct!() { ANAKIWA } else { CHESTNUT_ROSE },
        );
        draw_handle.draw_text("PERSPECTIVE [ P ]:", 12, 418, FONT_SIZE, SUNFLOWER);
        draw_handle.draw_text(
            if perspective_correct!() { "CORRECT" } else { "INCORRECT" },
            230,
            418,
            FONT_SIZE,
            if perspective_correct!() { ANAKIWA } else { CHESTNUT_ROSE },
        );
        draw_handle.draw_text("LENS [ O ]:", 510, 366, FONT_SIZE, SUNFLOWER);
        draw_handle.draw_text(
            if ortho_mode!() { "ORTHOGRAPHIC" } else { "PERSPECTIVE" },
            630,
            366,
            FONT_SIZE,
            if ortho_mode!() { BAHAMA_BLUE } else { ANAKIWA },
        );
        draw_handle.draw_text("SPACE [ N ]:", 520, 392, FONT_SIZE, SUNFLOWER);
        draw_handle.draw_text(
            if ndc_space!() { "NDC" } else { "WORLD" },
            655,
            392,
            FONT_SIZE,
            if ndc_space!() { BAHAMA_BLUE } else { ANAKIWA },
        );
        if ndc_space!() {
            draw_handle.draw_text("REFLECT [ F ]:", 530, 418, FONT_SIZE, SUNFLOWER);
            draw_handle.draw_text(
                if reflect_y!() { "Y_DOWN" } else { "Y_UP" },
                695,
                418,
                FONT_SIZE,
                if reflect_y!() { ANAKIWA } else { CHESTNUT_ROSE },
            );
        }
        //----------------------------------------------------------------------------------
    }
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
) {
    let (depth, right, up) = basis_vector(&main);
    let half_h_near = lerp(
        near * (FOVY_PERSPECTIVE * 0.5).to_radians().tan(),
        0.5 * NEAR_PLANE_HEIGHT_ORTHOGRAPHIC(),
        ortho_blend_factor(0.0),
    );
    let half_w_near = lerp(half_h_near, half_h_near * aspect, aspect_blend_factor(0.0));
    let half_depth_ndc = lerp(
        half_h_near,
        0.5 * (far - near),
        lerp(aspect_blend_factor(0.0), 0.0, ortho_blend_factor(0.0)),
    );
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

            let intersection_coord = intersect(main, near, world_vertex);
            let clip_plane_vector = intersection_coord.sub(center_near_plane);
            let x_ndc = clip_plane_vector.dot(right) / half_w_near;
            let y_ndc = clip_plane_vector.dot(up) / half_h_near;
            let z_ndc = lerp(
                (far + near - 2.0 * far * near / signed_depth) / (far - near),
                2.0 * (signed_depth - near) / (far - near) - 1.0,
                ortho_blend_factor(0.0),
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

fn draw_model_filled(
    thread: &RaylibThread,
    rl3d: &mut RaylibMode3D<RaylibDrawHandle>,
    model: &mut Model,
    mesh_texture: &Texture2D,
    rotation: f32,
) {
    if !(color_mode!() || texture_mode!()) {
        return;
    }
    let _color_guard = if texture_mode!() && !color_mode!() {
        Some(ColorGuard::hide(&mut model.meshes_mut()[0]))
    } else {
        None
    };

    model.materials_mut()[0].maps_mut()[MATERIAL_MAP_ALBEDO as usize]
        .texture
        .id = if texture_mode!() {
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
} // NOTE!!!!!! Colors automatically restored when _color_guard drops

fn draw_model_wires_and_points(
    thread: &RaylibThread,
    rl3d: &mut RaylibMode3D<RaylibDrawHandle>,
    model: &mut Model,
    rotation: f32,
) {
    let _color_guard = if !clip_mode!() {
        Some(ColorGuard::hide(&mut model.meshes_mut()[0]))
    } else {
        None
    };

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

fn update_spatial_frame(main: &mut Camera3D, aspect: f32, near: f32, far: f32, spatial_frame: &mut WeakMesh) {
    let (depth, right, up) = basis_vector(&main);
    let half_h_near = lerp(
        near * (FOVY_PERSPECTIVE * 0.5).to_radians().tan(),
        0.5 * NEAR_PLANE_HEIGHT_ORTHOGRAPHIC(),
        ortho_blend_factor(0.0),
    );
    let half_w_near = lerp(half_h_near, half_h_near * aspect, aspect_blend_factor(0.0));
    let half_h_far = lerp(
        far * (FOVY_PERSPECTIVE * 0.5).to_radians().tan(),
        0.5 * NEAR_PLANE_HEIGHT_ORTHOGRAPHIC(),
        ortho_blend_factor(0.0),
    );
    let half_w_far = lerp(half_h_far, half_h_far * aspect, aspect_blend_factor(0.0));
    let half_depth_ndc = lerp(
        half_h_near,
        0.5 * (far - near),
        lerp(aspect_blend_factor(0.0), 0.0, ortho_blend_factor(0.0)),
    );
    let half_depth = lerp(0.5 * (far - near), half_depth_ndc, space_blend_factor(0.0));
    let far_half_w = lerp(half_w_far, half_w_near, space_blend_factor(0.0));
    let far_half_h = lerp(half_h_far, half_h_near, space_blend_factor(0.0));
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

fn draw_near_plane_points(
    thread: &RaylibThread,
    rl3d: &mut RaylibMode3D<RaylibDrawHandle>,
    main: &mut Camera3D,
    aspect: f32,
    near: f32,
    near_plane_points_model: &mut Model,
    display_model: &Model,
    rotation: f32,
) {
    let display_mesh = &display_model.meshes()[0];
    let (depth, right, up) = basis_vector(&main);
    let mut count = 0usize;
    let capacity = display_mesh.triangle_count() * 3;
    let near_plane_points_mesh = &mut near_plane_points_model.meshes_mut()[0];
    // let capacity = near_plane_points_mesh.vertex_count(); //FIXME NOTE: I dare you to spend another 5 hours learning the difference between indexed nd unindexed and what a corner position on a triangle is vs an actual vertex
    near_plane_points_mesh.resize(capacity).expect("vertices resized");

    let center_near_plane = main.position.add(depth * near);
    let x_aspect = lerp(1.0 / aspect, 1.0, aspect_blend_factor(0.0));
    let y_reflect = lerp(1.0, -1.0, reflect_blend_factor(0.0));

    let vertices = display_mesh.vertices();
    let near_plane_points_mesh_vertices = near_plane_points_mesh.vertices_mut();

    'tri_loop: for [a, b, c] in display_mesh.triangles() {
        let vertex_a = translate_rotate_scale(0, vertices[a], MODEL_POS, MODEL_SCALE, rotation);
        let vertex_b = translate_rotate_scale(0, vertices[b], MODEL_POS, MODEL_SCALE, rotation);
        let vertex_c = translate_rotate_scale(0, vertices[c], MODEL_POS, MODEL_SCALE, rotation);
        if vertex_b
            .sub(vertex_a)
            .cross(vertex_c.sub(vertex_a))
            .normalize_or_zero()
            .dot(depth)
            > 0.0
        {
            continue 'tri_loop;
        }
        for &world_coord in [vertex_a, vertex_b, vertex_c].iter() {
            if count >= capacity {
                break;
            }
            let intersection_points = intersect(main, near, world_coord);
            let corrected = aspect_correct_and_reflect_near_plane(
                intersection_points,
                center_near_plane,
                right,
                up,
                x_aspect,
                y_reflect,
            );
            rl3d.draw_line3D(
                world_coord,
                corrected,
                Color::new(RED_DAMASK.r, RED_DAMASK.g, RED_DAMASK.b, 20),
            );
            near_plane_points_mesh_vertices[count] = corrected;
            count += 1;
        }
    }

    // near_plane_points_mesh.resize(count).expect("hope it worked");
    near_plane_points_mesh
        .resize_sync(thread, count)
        .expect("hope it worked"); //TODO: test resize_sync on stuff actually visible with opengl33... points are tough to see
                                   // unsafe { near_plane_points_mesh.update_position_buffer(thread); }
    unsafe { rlSetPointSize(3.0) }
    rl3d.draw_model_points(near_plane_points_model, MODEL_POS, 1.0, LILAC);
}

fn perspective_incorrect_capture(
    main: &mut Camera3D,
    aspect: f32,
    near: f32,
    display_model: &Model,
    mesh_texture: &Texture2D,
    rotation: f32,
) {
    let (depth, right, up) = basis_vector(&main);
    let center_near_plane = main.position + depth * near;
    let x_aspect = lerp(1.0 / aspect, 1.0, aspect_blend_factor(0.0));
    let y_reflect = lerp(1.0, -1.0, reflect_blend_factor(0.0));

    let display_mesh = &display_model.meshes()[0];
    let vertices = display_mesh.vertices();
    let colors = display_mesh.colors();
    let texcoords = display_mesh.texcoords();

    unsafe {
        rlColor4ub(Color::WHITE.r, Color::WHITE.g, Color::WHITE.b, Color::WHITE.a);
    }
    if texture_mode!() {
        unsafe {
            rlSetTexture(mesh_texture.id);
        }
        unsafe {
            rlEnableTexture(mesh_texture.id);
        }
    } else {
        unsafe {
            rlDisableTexture();
        }
    }
    if !texture_mode!() && !color_mode!() {
        unsafe {
            rlEnableWireMode();
        }
        unsafe {
            rlColor4ub(MARINER.r, MARINER.g, MARINER.b, MARINER.a);
        }
    }
    unsafe {
        rlBegin(RL_TRIANGLES as i32);
    }

    for [ia, ib, ic] in display_mesh.triangles() {
        let mut a = translate_rotate_scale(0, vertices[ia], MODEL_POS, MODEL_SCALE, rotation);
        let mut b = translate_rotate_scale(0, vertices[ib], MODEL_POS, MODEL_SCALE, rotation);
        let mut c = translate_rotate_scale(0, vertices[ic], MODEL_POS, MODEL_SCALE, rotation);

        a = aspect_correct_and_reflect_near_plane(
            intersect(main, near, a),
            center_near_plane,
            right,
            up,
            x_aspect,
            y_reflect,
        );
        b = aspect_correct_and_reflect_near_plane(
            intersect(main, near, b),
            center_near_plane,
            right,
            up,
            x_aspect,
            y_reflect,
        );
        c = aspect_correct_and_reflect_near_plane(
            intersect(main, near, c),
            center_near_plane,
            right,
            up,
            x_aspect,
            y_reflect,
        );

        //TODO: I hate all these "Some" checks, but perhaps its truly the best most emphatic way to demonstrate rust options?
        if let Some(rgba) = colors {
            if color_mode!() {
                unsafe {
                    rlColor4ub(rgba[ia].r, rgba[ia].g, rgba[ia].b, rgba[ia].a);
                }
            }
        }
        if let Some(st) = texcoords {
            if texture_mode!() {
                unsafe {
                    rlTexCoord2f(st[ia].x, st[ia].y);
                }
            }
        }
        unsafe {
            rlVertex3f(a.x, a.y, a.z);
        }

        let (second_index, second_vertex) = if ndc_space!() && reflect_y!() { (ic, c) } else { (ib, b) };
        if let Some(rgba) = colors {
            if color_mode!() {
                unsafe {
                    rlColor4ub(
                        rgba[second_index].r,
                        rgba[second_index].g,
                        rgba[second_index].b,
                        rgba[second_index].a,
                    );
                }
            }
        }
        if let Some(st) = texcoords {
            if texture_mode!() {
                unsafe {
                    rlTexCoord2f(st[second_index].x, st[second_index].y);
                }
            }
        }
        unsafe {
            rlVertex3f(second_vertex.x, second_vertex.y, second_vertex.z);
        }

        let (third_index, third_vertex) = if ndc_space!() && reflect_y!() { (ib, b) } else { (ic, c) };
        if let Some(rgba) = colors {
            if color_mode!() {
                unsafe {
                    rlColor4ub(
                        rgba[third_index].r,
                        rgba[third_index].g,
                        rgba[third_index].b,
                        rgba[third_index].a,
                    );
                }
            }
        }
        if let Some(st) = texcoords {
            if texture_mode!() {
                unsafe {
                    rlTexCoord2f(st[third_index].x, st[third_index].y);
                }
            }
        }
        unsafe {
            rlVertex3f(third_vertex.x, third_vertex.y, third_vertex.z);
        }
    }

    unsafe {
        rlEnd();
    }
    unsafe {
        rlDrawRenderBatchActive();
    } //NOTE: WHOA THIS ALLOWS TRIANGLES IN WIRE MODE, OTHERWISE WILL DRAW FILLED TRIANGLES
    unsafe {
        rlSetTexture(rlGetTextureIdDefault());
    }
    unsafe {
        rlDisableTexture();
    }
    unsafe {
        rlDisableWireMode();
    }
}

fn perspective_correct_capture(
    thread: &RaylibThread,
    draw_handle: &mut RaylibDrawHandle,
    main: &mut Camera3D,
    model: &mut Model,
    mesh_texture: &Texture2D,
    perspective_correct_texture: &mut Texture2D,
    rotation: f32,
) {
    let _color_guard = if texture_mode!() && !color_mode!() {
        Some(ColorGuard::hide(&mut model.meshes_mut()[0]))
    } else {
        None
    };

    let cache_texture_id = {
        let material_map = &mut model.materials_mut()[0].maps_mut()[MATERIAL_MAP_ALBEDO as usize];
        let id = material_map.texture.id;
        material_map.texture.id = mesh_texture.id;
        id
    };

    draw_handle.draw_mode3D(*main, |mut rl3d| {
        rl3d.clear_background(Color::BLACK);
        rl3d.draw_model_ex(
            &mut *model,
            MODEL_POS,
            Y_AXIS,
            rotation.to_degrees(),
            MODEL_SCALE,
            Color::WHITE,
        );
    });

    model.materials_mut()[0].maps_mut()[MATERIAL_MAP_ALBEDO as usize]
        .texture
        .id = cache_texture_id;

    let mut rgba = draw_handle.load_image_from_screen(thread);
    Image::set_format(&mut rgba, PIXELFORMAT_UNCOMPRESSED_R8G8B8A8);
    // model.meshes_mut()[0].set_colors(cache_colors.as_deref()).unwrap();
    // Colors restored here automatically????????????????????????? HOW>?????

    let (cache_material_texture_id, cache_material_color) = {
        let material_map = &mut model.materials_mut()[0].maps_mut()[MATERIAL_MAP_ALBEDO as usize];
        let material_texture_id = material_map.texture.id;
        let material_color = material_map.color;
        material_map.texture.id = 0;
        material_map.color = Color::WHITE;
        (material_texture_id, material_color)
    };

    draw_handle.draw_mode3D(*main, |mut rl3d| {
        rl3d.clear_background(Color::BLACK);
        rl3d.draw_model_ex(
            &mut *model,
            MODEL_POS,
            Y_AXIS,
            rotation.to_degrees(),
            MODEL_SCALE,
            Color::WHITE,
        );
    });

    let material_map = &mut model.materials_mut()[0].maps_mut()[MATERIAL_MAP_ALBEDO as usize];
    material_map.texture.id = cache_material_texture_id;
    material_map.color = cache_material_color;

    let mut mask = draw_handle.load_image_from_screen(thread);
    alpha_mask_punch_out(&mut rgba, &mut mask, 1);
    Image::flip_vertical(&mut rgba);
    if ndc_space!() && reflect_y!() {
        Image::flip_vertical(&mut rgba);
    }

    let pixels = rgba.get_image_data_u8(false);
    if perspective_correct_texture.id != 0 {
        perspective_correct_texture.update_texture(&pixels).unwrap();
    } else {
        let _ = replace(
            perspective_correct_texture,
            draw_handle.load_texture_from_image(thread, &rgba).unwrap(),
        );
        // *perspective_correct_texture = draw_handle.load_texture_from_image(thread, &rgba).unwrap();
    }
}

fn alpha_mask_punch_out(rgba: &mut Image, mask: &mut Image, threshold: u8) {
    Image::set_format(mask, PIXELFORMAT_UNCOMPRESSED_GRAYSCALE);
    Image::set_format(rgba, PIXELFORMAT_UNCOMPRESSED_R8G8B8A8);
    for y in 0..rgba.height {
        for x in 0..rgba.width {
            let gray_scale_mask = mask.get_color(x, y).r;
            let mut color = rgba.get_color(x, y);
            color.a = if gray_scale_mask > threshold { 255 } else { 0 };
            rgba.draw_pixel(x, y, color);
        }
    }
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
    mesh.init_colors_mut().unwrap().copy_from_slice(&colors); //TODO: ahh jeez.. wtf is the point of having added set_colors?.. or do we fill?
}

fn orbit_space(handle: &mut RaylibHandle, jugemu: &mut Camera3D) {
    let dt = handle.get_frame_time();
    let mut radius = (jugemu.position.x * jugemu.position.x
        + jugemu.position.y * jugemu.position.y
        + jugemu.position.z * jugemu.position.z)
        .sqrt();
    let mut azimuth = jugemu.position.z.atan2(jugemu.position.x);
    let horizontal_radius = (jugemu.position.x * jugemu.position.x + jugemu.position.z * jugemu.position.z).sqrt();
    let mut elevation = jugemu.position.y.atan2(horizontal_radius);
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
    elevation = elevation.clamp(-PI / 2.0 + 0.1, PI / 2.0 - 0.1);
    radius = radius.clamp(0.25, 10.0);
    jugemu.position.x = radius * elevation.cos() * azimuth.cos();
    jugemu.position.y = radius * elevation.sin();
    jugemu.position.z = radius * elevation.cos() * azimuth.sin();
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

fn intersect(main: &mut Camera3D, near: f32, world_coord: Vector3) -> Vector3 {
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
        result_perspective.x + (result_ortho.x - result_perspective.x) * ortho_blend_factor(0.0),
        result_perspective.y + (result_ortho.y - result_perspective.y) * ortho_blend_factor(0.0),
        result_perspective.z + (result_ortho.z - result_perspective.z) * ortho_blend_factor(0.0),
    )
}

fn space_blend_factor(dt: f32) -> f32 {
    static mut BLEND: f32 = 0.0;
    unsafe {
        if dt > 0.0 {
            BLEND = (BLEND + (if ndc_space!() { 1.0 } else { -1.0 }) * BLEND_SCALAR * dt).clamp(0.0, 1.0);
        }
        BLEND
    }
}

fn aspect_blend_factor(dt: f32) -> f32 {
    static mut BLEND: f32 = 0.0;
    unsafe {
        if dt > 0.0 {
            BLEND = (BLEND + (if aspect_correct!() { 1.0 } else { -1.0 }) * BLEND_SCALAR * dt).clamp(0.0, 1.0);
        }
        BLEND
    }
}

fn reflect_blend_factor(dt: f32) -> f32 {
    static mut BLEND: f32 = 0.0;
    unsafe {
        if dt > 0.0 {
            let target = if ndc_space!() && reflect_y!() { 1.0 } else { 0.0 };
            let direction = if BLEND < target {
                1.0
            } else if BLEND > target {
                -1.0
            } else {
                0.0
            };
            BLEND = (BLEND + direction * BLEND_SCALAR * dt).clamp(0.0, 1.0);
        }
        BLEND
    }
}

fn ortho_blend_factor(dt: f32) -> f32 {
    static mut BLEND: f32 = 0.0;
    unsafe {
        if dt > 0.0 {
            BLEND = (BLEND + (if ortho_mode!() { 1.0 } else { -1.0 }) * BLEND_SCALAR * dt).clamp(0.0, 1.0);
        }
        BLEND
    }
}
