use gltf::Gltf;
use raylib::camera::Camera3D;
use raylib::color::Color;
use raylib::consts::{CameraProjection, KeyboardKey, MaterialMapIndex, PixelFormat, TraceLogLevel};
use raylib::drawing::{RaylibDraw, RaylibDraw3D, RaylibMode3DExt};
use raylib::ffi::{rlSetLineWidth, rlSetPointSize};
use raylib::init;
use raylib::math::Vector3;
use raylib::models::{Model, RaylibMaterial, RaylibMesh, RaylibModel, WeakMesh};
use raylib::texture::Image;
use raylib::RaylibHandle;
use raylib::RaylibThread;
use std::fs;
use std::path::PathBuf;

const SCREEN_WIDTH: i32 = 800;
const SCREEN_HEIGHT: i32 = 600;
const FONT_SIZE: i32 = 20;
const SUNFLOWER: Color = Color::new(255, 204, 153, 255);
const ANAKIWA: Color = Color::new(153, 204, 255, 255);
const MARINER: Color = Color::new(51, 102, 204, 255);
const NEON_CARROT: Color = Color::new(255, 153, 51, 255);
const CHESTNUT_ROSE: Color = Color::new(204, 102, 102, 255);
const LILAC: Color = Color::new(204, 153, 204, 255);

struct ViewerState {
    show_pvc: bool,
    show_wireframe: bool,
    show_points: bool,
    show_texture: bool,
    current_model_index: usize,
    ortho: bool,
}

impl Default for ViewerState {
    fn default() -> Self {
        Self {
            show_pvc: true,
            show_wireframe: true,
            show_points: false,
            show_texture: true,
            current_model_index: 0,
            ortho: false,
        }
    }
}

fn collect_model_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let extensions = ["glb", "gltf", "obj"];
    if let Ok(entries) = fs::read_dir("/Users/adduser/meshdump") {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                let ext_lower = ext.to_string_lossy().to_lowercase();
                if extensions.contains(&ext_lower.as_str()) {
                    paths.push(path);
                }
            }
        }
    }
    paths.sort_by(|a, b| {
        let na = a.file_name().unwrap_or_default().to_string_lossy().to_string();
        let nb = b.file_name().unwrap_or_default().to_string_lossy().to_string();
        let pa = if na.starts_with("panchang_") {
            0
        } else if na.starts_with("charm_") {
            1
        } else {
            2
        };
        let pb = if nb.starts_with("panchang_") {
            0
        } else if nb.starts_with("charm_") {
            1
        } else {
            2
        };
        pa.cmp(&pb).then(na.cmp(&nb))
    });
    paths
}

fn dump_mesh_info(path: &PathBuf) {
    println!("\n=== {} ===", path.file_name().unwrap_or_default().to_string_lossy());
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    match ext.to_lowercase().as_str() {
        "glb" | "gltf" => {
            if let Ok(data) = fs::read(path) {
                if let Ok(gltf) = Gltf::from_slice(&data) {
                    for (i, mesh) in gltf.meshes().enumerate() {
                        println!("  Mesh[{}]: {:?}", i, mesh.name());
                    }
                }
            }
        },
        "obj" => {
            if let Ok(content) = fs::read_to_string(path) {
                let verts = content.lines().filter(|l| l.starts_with("v ")).count();
                let faces = content.lines().filter(|l| l.starts_with("f ")).count();
                println!("  {} verts, {} faces", verts, faces);
            }
        },
        _ => {},
    }
}

struct ColorGuard {
    mesh_colors_entries: Vec<(*mut raylib::ffi::Mesh, *mut u8)>,
}

impl ColorGuard {
    fn hide(model: &mut Model) -> Self {
        let mut entries = Vec::new();
        for mesh in model.meshes_mut() {
            let mesh_ptr = mesh.as_mut() as *mut raylib::ffi::Mesh;
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
        for (mesh_ptr, colors_ptr) in &self.mesh_colors_entries {
            unsafe {
                (**mesh_ptr).colors = *colors_ptr;
            }
        }
    }
}

struct TextureGuard {
    cached_ids: Vec<(usize, u32)>,
    model_ptr: *mut Model,
}

impl TextureGuard {
    fn hide(model: &mut Model) -> Self {
        let mut cached_ids = Vec::new();
        for (i, mat) in model.materials_mut().iter_mut().enumerate() {
            let id = mat.maps_mut()[MaterialMapIndex::MATERIAL_MAP_ALBEDO as usize]
                .texture
                .id;
            mat.maps_mut()[MaterialMapIndex::MATERIAL_MAP_ALBEDO as usize]
                .texture
                .id = 0;
            cached_ids.push((i, id));
        }
        Self {
            cached_ids,
            model_ptr: model as *mut Model,
        }
    }
}

impl Drop for TextureGuard {
    fn drop(&mut self) {
        unsafe {
            for (i, id) in &self.cached_ids {
                (*self.model_ptr).materials_mut()[*i].maps_mut()[MaterialMapIndex::MATERIAL_MAP_ALBEDO as usize]
                    .texture
                    .id = *id;
            }
        }
    }
}

fn fill_vertex_colors(mesh: &mut WeakMesh) {
    let bounds = mesh.get_mesh_bounding_box();
    let vertices = mesh.vertices();
    let mut colors = vec![Color::WHITE; vertices.len()];
    for i in 0..vertices.len() {
        let vertex = vertices[i];
        let nx = if bounds.max.x != bounds.min.x {
            (vertex.x - 0.5 * (bounds.min.x + bounds.max.x)) / (0.5 * (bounds.max.x - bounds.min.x))
        } else {
            0.0
        };
        let ny = if bounds.max.y != bounds.min.y {
            (vertex.y - 0.5 * (bounds.min.y + bounds.max.y)) / (0.5 * (bounds.max.y - bounds.min.y))
        } else {
            0.0
        };
        let nz = if bounds.max.z != bounds.min.z {
            (vertex.z - 0.5 * (bounds.min.z + bounds.max.z)) / (0.5 * (bounds.max.z - bounds.min.z))
        } else {
            0.0
        };
        let len = (nx * nx + ny * ny + nz * nz).sqrt().max(0.001);
        colors[i] = Color::new(
            (127.5 * (nx / len + 1.0)).round() as u8,
            (127.5 * (ny / len + 1.0)).round() as u8,
            (127.5 * (nz / len + 1.0)).round() as u8,
            255,
        );
    }
    if let Ok(color_slice) = mesh.init_colors_mut() {
        color_slice.copy_from_slice(&colors);
    }
}

fn load_glb_textures(
    path: &PathBuf,
    handle: &mut RaylibHandle,
    thread: &RaylibThread,
    model: &mut Model,
    texture_storage: &mut Vec<Box<[u8]>>,
) {
    let path_str = path.to_string_lossy();
    if let Ok((doc, _buffers, images)) = gltf::import(path_str.as_ref()) {
        for (mat_idx, material) in doc.materials().enumerate() {
            if let Some(tex_info) = material.pbr_metallic_roughness().base_color_texture() {
                let tex_idx = tex_info.texture().source().index();
                if tex_idx < images.len() {
                    let img = &images[tex_idx];
                    let rgba_pixels: Vec<u8> = match img.format {
                        gltf::image::Format::R8G8B8 => img
                            .pixels
                            .chunks(3)
                            .flat_map(|rgb| [rgb[0], rgb[1], rgb[2], 255])
                            .collect(),
                        gltf::image::Format::R8G8B8A8 => img.pixels.clone(),
                        gltf::image::Format::R8 => img.pixels.iter().flat_map(|&r| [r, r, r, 255]).collect(),
                        gltf::image::Format::R8G8 => {
                            img.pixels.chunks(2).flat_map(|rg| [rg[0], rg[1], 0, 255]).collect()
                        },
                        _ => continue,
                    };
                    texture_storage.push(rgba_pixels.into_boxed_slice());
                    let pixel_data = texture_storage.last_mut().unwrap();
                    let ffi_image = raylib::ffi::Image {
                        data: pixel_data.as_mut_ptr() as *mut std::ffi::c_void,
                        width: img.width as i32,
                        height: img.height as i32,
                        mipmaps: 1,
                        format: PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8 as i32,
                    };
                    let raylib_img = unsafe { Image::from_raw(ffi_image) };
                    if let Ok(texture) = handle.load_texture_from_image(thread, &raylib_img) {
                        let raylib_mat_idx = mat_idx + 1;
                        if raylib_mat_idx < model.materials().len() {
                            model.materials_mut()[raylib_mat_idx]
                                .set_material_texture(MaterialMapIndex::MATERIAL_MAP_ALBEDO, &texture);
                        }
                        std::mem::forget(texture);
                    }
                    std::mem::forget(raylib_img);
                }
            }
        }
    }
}

fn compute_model_bounds(model: &Model) -> (Vector3, f32) {
    let mut min = Vector3::new(f32::MAX, f32::MAX, f32::MAX);
    let mut max = Vector3::new(f32::MIN, f32::MIN, f32::MIN);
    for mesh in model.meshes().iter() {
        for v in mesh.vertices() {
            min.x = min.x.min(v.x);
            min.y = min.y.min(v.y);
            min.z = min.z.min(v.z);
            max.x = max.x.max(v.x);
            max.y = max.y.max(v.y);
            max.z = max.z.max(v.z);
        }
    }
    let centroid = Vector3::new((min.x + max.x) * 0.5, (min.y + max.y) * 0.5, (min.z + max.z) * 0.5);
    let extent = Vector3::new(max.x - min.x, max.y - min.y, max.z - min.z);
    let radius = (extent.x * extent.x + extent.y * extent.y + extent.z * extent.z).sqrt() * 0.5;
    (centroid, radius)
}

fn compute_fit_distance(bounds: f32, fovy_deg: f32, aspect: f32) -> f32 {
    let fovy_rad = fovy_deg.to_radians();
    let fovx_rad = 2.0 * ((fovy_rad / 2.0).tan() * aspect).atan();
    let effective_fov = fovy_rad.min(fovx_rad);
    let half_angle = effective_fov / 2.0;
    (bounds / half_angle.tan()).max(0.5)
}

fn main() {
    let model_paths = collect_model_paths();
    if model_paths.is_empty() {
        eprintln!("No model files found in Downloads!");
        return;
    }
    println!("\n=== MESH VIEWER ===");
    println!("Found {} models", model_paths.len());
    for (i, path) in model_paths.iter().enumerate().take(9) {
        println!(
            "  [{}] {}",
            i + 1,
            path.file_name().unwrap_or_default().to_string_lossy()
        );
    }
    let (mut handle, thread) = init()
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("mesh_viewer")
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();
    handle.set_target_fps(60);
    let aspect = SCREEN_WIDTH as f32 / SCREEN_HEIGHT as f32;
    let mut state = ViewerState::default();
    let mut camera = Camera3D {
        position: Vector3::new(0.0, 0.0, 3.0),
        target: Vector3::ZERO,
        up: Vector3::Y,
        fovy: 45.0,
        projection: CameraProjection::CAMERA_PERSPECTIVE,
    };
    let mut current_model: Option<Model> = None;
    let mut current_name = String::new();
    let mut current_centroid = Vector3::ZERO;
    let mut current_radius: f32 = 1.0;
    let mut loaded_textures: Vec<Box<[u8]>> = Vec::new();
    while !handle.window_should_close() {
        let mut model_changed = current_model.is_none();
        if handle.is_key_pressed(KeyboardKey::KEY_LEFT) && !model_paths.is_empty() {
            state.current_model_index = if state.current_model_index == 0 {
                model_paths.len() - 1
            } else {
                state.current_model_index - 1
            };
            model_changed = true;
        }
        if handle.is_key_pressed(KeyboardKey::KEY_RIGHT) && !model_paths.is_empty() {
            state.current_model_index = (state.current_model_index + 1) % model_paths.len();
            model_changed = true;
        }
        if model_changed {
            drop(current_model.take());
            loaded_textures.clear();

            let path = &model_paths[state.current_model_index];
            dump_mesh_info(path);
            current_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
            match handle.load_model(&thread, &path.to_string_lossy()) {
                Ok(mut model) => {
                    for mesh in model.meshes_mut().iter_mut() {
                        fill_vertex_colors(mesh);
                    }
                    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                    if ext.eq_ignore_ascii_case("glb") || ext.eq_ignore_ascii_case("gltf") {
                        load_glb_textures(path, &mut handle, &thread, &mut model, &mut loaded_textures);
                    }
                    let (centroid, radius) = compute_model_bounds(&model);
                    current_centroid = centroid;
                    current_radius = radius;
                    let dist = compute_fit_distance(radius, camera.fovy, aspect);
                    camera.target = centroid;
                    camera.position = Vector3::new(centroid.x, centroid.y, centroid.z + dist);
                    current_model = Some(model);
                },
                Err(e) => {
                    eprintln!("Failed to load: {}", e);
                    current_model = None;
                },
            }
        }
        if handle.is_key_pressed(KeyboardKey::KEY_C) {
            state.show_pvc = !state.show_pvc;
        }
        if handle.is_key_pressed(KeyboardKey::KEY_T) {
            state.show_texture = !state.show_texture;
        }
        if handle.is_key_pressed(KeyboardKey::KEY_X) {
            state.show_wireframe = !state.show_wireframe;
        }
        if handle.is_key_pressed(KeyboardKey::KEY_P) {
            state.show_points = !state.show_points;
        }
        if handle.is_key_pressed(KeyboardKey::KEY_O) {
            state.ortho = !state.ortho;
            camera.projection = if state.ortho {
                CameraProjection::CAMERA_ORTHOGRAPHIC
            } else {
                CameraProjection::CAMERA_PERSPECTIVE
            };
            if state.ortho {
                camera.fovy = current_radius * 2.5;
            } else {
                camera.fovy = 45.0;
            }
        }
        if handle.is_key_pressed(KeyboardKey::KEY_R) {
            let dist = compute_fit_distance(current_radius, camera.fovy, aspect);
            camera.position = Vector3::new(current_centroid.x, current_centroid.y, current_centroid.z + dist);
        }
        let rot_speed = 2.0 * handle.get_frame_time();
        let mut orbit_yaw = 0.0f32;
        let mut orbit_pitch = 0.0f32;
        if handle.is_key_down(KeyboardKey::KEY_A) {
            orbit_yaw -= rot_speed;
        }
        if handle.is_key_down(KeyboardKey::KEY_D) {
            orbit_yaw += rot_speed;
        }
        if handle.is_key_down(KeyboardKey::KEY_W) {
            orbit_pitch -= rot_speed;
        }
        if handle.is_key_down(KeyboardKey::KEY_S) {
            orbit_pitch += rot_speed;
        }
        if orbit_yaw != 0.0 || orbit_pitch != 0.0 {
            let mut offset = Vector3::new(
                camera.position.x - camera.target.x,
                camera.position.y - camera.target.y,
                camera.position.z - camera.target.z,
            );
            let dist = (offset.x * offset.x + offset.y * offset.y + offset.z * offset.z).sqrt();
            if orbit_yaw != 0.0 {
                let cos_y = orbit_yaw.cos();
                let sin_y = orbit_yaw.sin();
                let new_x = offset.x * cos_y + offset.z * sin_y;
                let new_z = -offset.x * sin_y + offset.z * cos_y;
                offset.x = new_x;
                offset.z = new_z;
            }
            if orbit_pitch != 0.0 {
                let horiz = (offset.x * offset.x + offset.z * offset.z).sqrt();
                let mut elev = offset.y.atan2(horiz);
                elev += orbit_pitch;
                elev = elev.clamp(-1.4, 1.4); // Limit pitch
                let new_horiz = dist * elev.cos();
                offset.y = dist * elev.sin();
                if horiz > 0.001 {
                    let scale = new_horiz / horiz;
                    offset.x *= scale;
                    offset.z *= scale;
                }
            }
            camera.position.x = camera.target.x + offset.x;
            camera.position.y = camera.target.y + offset.y;
            camera.position.z = camera.target.z + offset.z;
        }
        let wheel = handle.get_mouse_wheel_move();
        if wheel != 0.0 {
            if state.ortho {
                camera.fovy = (camera.fovy - wheel * 0.2).clamp(0.1, current_radius * 10.0);
            } else {
                let dir = Vector3::new(
                    camera.position.x - camera.target.x,
                    camera.position.y - camera.target.y,
                    camera.position.z - camera.target.z,
                );
                let len = (dir.x * dir.x + dir.y * dir.y + dir.z * dir.z).sqrt();
                let new_len = (len - wheel * current_radius * 0.3).clamp(current_radius * 0.5, current_radius * 10.0);
                let scale = new_len / len;
                camera.position.x = camera.target.x + dir.x * scale;
                camera.position.y = camera.target.y + dir.y * scale;
                camera.position.z = camera.target.z + dir.z * scale;
            }
        }
        let key_zoom = if handle.is_key_down(KeyboardKey::KEY_ONE) {
            1.0f32
        } else if handle.is_key_down(KeyboardKey::KEY_ZERO) {
            -1.0
        } else {
            0.0
        };
        if key_zoom != 0.0 {
            let zoom_amount = key_zoom * handle.get_frame_time() * 2.0;
            if state.ortho {
                camera.fovy = (camera.fovy - zoom_amount * current_radius).clamp(0.1, current_radius * 10.0);
            } else {
                let dir = Vector3::new(
                    camera.position.x - camera.target.x,
                    camera.position.y - camera.target.y,
                    camera.position.z - camera.target.z,
                );
                let len = (dir.x * dir.x + dir.y * dir.y + dir.z * dir.z).sqrt();
                let new_len = (len - zoom_amount * current_radius).clamp(current_radius * 0.5, current_radius * 10.0);
                let scale = new_len / len;
                camera.position.x = camera.target.x + dir.x * scale;
                camera.position.y = camera.target.y + dir.y * scale;
                camera.position.z = camera.target.z + dir.z * scale;
            }
        }
        let mut draw_handle = handle.begin_drawing(&thread);
        draw_handle.clear_background(Color::BLACK);
        if let Some(ref mut model) = current_model {
            draw_handle.draw_mode3D(camera, |mut rl3d| {
                if state.show_texture || state.show_pvc {
                    let _color_guard = if !state.show_pvc {
                        Some(ColorGuard::hide(model))
                    } else {
                        None
                    };
                    let _texture_guard = if !state.show_texture {
                        Some(TextureGuard::hide(model))
                    } else {
                        None
                    };
                    rl3d.draw_model(&*model, Vector3::ZERO, 1.0, Color::WHITE);
                }
                if state.show_wireframe {
                    unsafe { rlSetLineWidth(2.0) };
                    let _color_guard = ColorGuard::hide(model);
                    let _texture_guard = TextureGuard::hide(model);
                    rl3d.draw_model_wires(&*model, Vector3::ZERO, 1.0, MARINER);
                }
                if state.show_points {
                    unsafe { rlSetPointSize(4.0) };
                    rl3d.draw_model_points(&*model, Vector3::ZERO, 1.0, LILAC);
                }
            });
        }
        draw_handle.draw_text(&format!("[</>]: {}", current_name), 12, 12, FONT_SIZE, NEON_CARROT);
        draw_handle.draw_text("TX [ T ]:", 570, 12, FONT_SIZE, SUNFLOWER);
        draw_handle.draw_text(
            if state.show_texture { "ON" } else { "OFF" },
            740,
            12,
            FONT_SIZE,
            if state.show_texture { ANAKIWA } else { CHESTNUT_ROSE },
        );
        draw_handle.draw_text("CLR [ C ]:", 570, 38, FONT_SIZE, SUNFLOWER);
        draw_handle.draw_text(
            if state.show_pvc { "ON" } else { "OFF" },
            740,
            38,
            FONT_SIZE,
            if state.show_pvc { ANAKIWA } else { CHESTNUT_ROSE },
        );
        draw_handle.draw_text("WR [ X ]:", 12, SCREEN_HEIGHT - 52, FONT_SIZE, SUNFLOWER);
        draw_handle.draw_text(
            if state.show_wireframe { "ON" } else { "OFF" },
            140,
            SCREEN_HEIGHT - 52,
            FONT_SIZE,
            if state.show_wireframe { ANAKIWA } else { CHESTNUT_ROSE },
        );
        draw_handle.draw_text("PT [ P ]:", 12, SCREEN_HEIGHT - 26, FONT_SIZE, SUNFLOWER);
        draw_handle.draw_text(
            if state.show_points { "ON" } else { "OFF" },
            140,
            SCREEN_HEIGHT - 26,
            FONT_SIZE,
            if state.show_points { ANAKIWA } else { CHESTNUT_ROSE },
        );
        draw_handle.draw_text("ORTHO [ O ]:", 570, SCREEN_HEIGHT - 26, FONT_SIZE, SUNFLOWER);
        draw_handle.draw_text(
            if state.ortho { "ON" } else { "OFF" },
            740,
            SCREEN_HEIGHT - 26,
            FONT_SIZE,
            if state.ortho { ANAKIWA } else { CHESTNUT_ROSE },
        );
    }
}
