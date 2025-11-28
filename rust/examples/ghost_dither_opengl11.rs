use asset_payload::SPHERE_PATH;
use bath::render::raylib::RaylibRenderer;
use bath::render::renderer::Renderer;
use raylib::math::glam::Vec3;
use raylib::prelude::*;
use std::f32::consts::TAU;

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
pub const LIGHT_WAVE_TEMPORAL_FREQ_X: f32 = 80.0;
pub const LIGHT_WAVE_TEMPORAL_FREQ_Y: f32 = 2.3;
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

pub const FOVY_ORTHOGRAPHIC: f32 = 2.0;
pub const MODEL_POS: Vector3 = Vector3::ZERO;
pub const SCALE_ELEMENT: f32 = 1.5;
pub const MODEL_SCALE: Vector3 = Vector3::new(SCALE_ELEMENT, SCALE_ELEMENT, SCALE_ELEMENT);

fn main() {
    let mut i_time = 0.0f32;
    let mut mesh_rotation = 0.0f32;
    let mut render = RaylibRenderer::init(640, 640);
    let mut main_observer = Camera3D {
        position: Vector3::new(0.0, 0.0, 2.0),
        target: Vector3::ZERO,
        up: Vector3::Y,
        fovy: FOVY_ORTHOGRAPHIC,
        projection: CameraProjection::CAMERA_ORTHOGRAPHIC,
    };
    let mut main_model = render.handle.load_model(&render.thread, SPHERE_PATH).unwrap();
    // let mut main_model = render.handle.load_model(&render.thread, SPHERE_MIN_PATH).unwrap();
    debug_iter_vertices(&mut main_model.meshes_mut()[0]);
    let indexed_mesh = indexed_mesh(&main_model.meshes_mut()[0]);
    let mut main_model = render
        .handle
        .load_model_from_mesh(&render.thread, indexed_mesh)
        .unwrap();
    debug_iter_vertices(&mut main_model.meshes_mut()[0]);

    let checked_img = Image::gen_image_checked(16, 16, 1, 1, Color::BLACK, Color::WHITE);
    let mesh_texture = render
        .handle
        .load_texture_from_image(&render.thread, &checked_img)
        .unwrap();
    // main_model.materials_mut()[0].set_material_texture(MATERIAL_MAP_ALBEDO, &mesh_texture);

    fill_vertex_normals(&mut main_model.meshes_mut()[0]);
    fill_vertex_colors(&mut main_model.meshes_mut()[0]);

    let mesh_samples = collect_deformed_vertex_samples(main_model.meshes()[0].vertices());
    interpolate_between_deformed_vertices(&mut main_model, i_time, &mesh_samples);
    while !render.handle.window_should_close() {
        orbit_space(&mut render.handle, &mut main_observer);
        i_time += render.handle.get_frame_time();
        mesh_rotation -= ANGULAR_VELOCITY * render.handle.get_frame_time();
        interpolate_between_deformed_vertices(&mut main_model, i_time, &mesh_samples);

        update_normals_for_silhouette(&mut main_model.meshes_mut()[0]);
        fade_vertex_colors_silhouette_rim(&mut main_model.meshes_mut()[0], &main_observer, mesh_rotation);

        let mut draw_handle = render.handle.begin_drawing(&render.thread);
        draw_handle.clear_background(Color::BLACK);
        draw_handle.draw_mode3D(main_observer, |mut rl3d| {
            rl3d.draw_model_ex(
                &main_model,
                // &indexed_model,
                MODEL_POS,
                Vector3::Y,
                mesh_rotation.to_degrees(),
                MODEL_SCALE,
                Color::WHITE,
            );
            {
                let _color_guard = ColorGuard::hide(&mut main_model.meshes_mut()[0]);
                rl3d.draw_model_wires_ex(
                    &main_model,
                    // &indexed_model,
                    MODEL_POS,
                    Vector3::Y,
                    mesh_rotation.to_degrees(),
                    MODEL_SCALE,
                    Color { a: 80, ..Color::WHITE },
                );
            } /*NOTE: end of closure -> `colors` automatically restored because _color_guard drops */
        });
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

fn fill_vertex_normals(mesh: &mut WeakMesh) {
    let vertices = mesh.vertices();
    let mut normals = vec![Vec3::ZERO; vertices.len()];
    for [a, b, c] in mesh.triangles() {
        let vertex_a = vertices[a];
        let vertex_b = vertices[b];
        let vertex_c = vertices[c];
        let face_normal = triangle_normal(vertex_a, vertex_b, vertex_c);
        normals[a] += face_normal;
        normals[b] += face_normal;
        normals[c] += face_normal;
    }
    //NOTE this is the second pass to actually smooth out the normals based on surrounding faces
    for i in mesh.triangles().iter_vertices() {
        normals[i] = normals[i].normalize_or_zero();
    }
    mesh.init_normals_mut().unwrap().copy_from_slice(&normals);
}

fn fill_vertex_colors(mesh: &mut WeakMesh) {
    let bounds = mesh.get_mesh_bounding_box();
    let vertices = mesh.vertices().to_vec();
    let mut colors = mesh.init_colors_mut().unwrap().to_vec();

    for i in mesh.triangles().iter_vertices() {
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
    mesh.colors_mut().unwrap().copy_from_slice(&colors);
}

fn indexed_mesh(mesh: &WeakMesh) -> Mesh {
    let vertices = mesh.vertices();
    let texcoords = mesh.texcoords();
    let vertex_count = vertices.len();

    let mut indexed_vertices: Vec<Vector3> = Vec::new();
    let mut indexed_texcoords: Option<Vec<Vector2>> = texcoords.map(|_| Vec::new());
    let mut indices: Vec<u16> = Vec::with_capacity(vertex_count);

    for i in 0..vertex_count {
        let identical = |a: usize, b: usize| -> bool {
            if vertices[a] != indexed_vertices[b] {
                return false;
            }
            match (texcoords, &indexed_texcoords) {
                (Some(src), Some(dst)) => src[a] == dst[b],
                _ => true,
            }
        };

        let mut existing = None;

        for j in 0..indexed_vertices.len() {
            if identical(i, j) {
                existing = Some(j);
                break;
            }
        }

        let index = match existing {
            Some(j) => j,
            None => {
                let j = indexed_vertices.len();
                indexed_vertices.push(vertices[i]);
                if let (Some(src), Some(dst)) = (texcoords, indexed_texcoords.as_mut()) {
                    dst.push(src[i]);
                }
                j
            },
        };

        indices.push(index as u16);
    }

    let mesh = Mesh::init_mesh(&indexed_vertices)
        .texcoords_opt(indexed_texcoords.as_deref())
        .indices(&indices)
        .build_cpu();

    mesh.unwrap()
}

fn debug_iter_vertices(mesh: &WeakMesh) {
    let vertices = mesh.vertices();
    let vertex_count = vertices.len();
    let is_indexed = mesh.is_indexed();
    let triangle_count = mesh.triangle_count();

    let mut visits = vec![0u32; vertex_count];
    let mut iter_vertex_count = 0usize;

    for i in mesh.triangles().iter_vertices() {
        iter_vertex_count += 1;
        visits[i] += 1;
    }

    let mut unique_visited = 0usize;
    let mut duplicates = 0usize;
    let mut never_visited = 0usize;

    for (i, count) in visits.iter().enumerate() {
        if *count == 0 {
            never_visited += 1;
        } else {
            unique_visited += 1;
            if *count > 1 {
                duplicates += 1;
                println!("debug_iter_vertices: vertex {} visited {} times", i, count);
            }
        }
    }
    println!("debug_iter_vertices:");
    println!("  is_indexed          = {}", is_indexed);
    println!("  vertex_count        = {}", vertex_count);
    println!("  triangle_count      = {}", triangle_count);
    println!("  total_index_slots   = {}", triangle_count * 3);
    println!("  iter_vertices_count = {}", iter_vertex_count);
    println!("  unique_visited      = {}", unique_visited);
    println!("  never_visited       = {}", never_visited);
    println!("  vertices_with_dupes = {}", duplicates);
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
    const FADE_RATE: f32 = 4.0; // UNUSED, BUT KEPT FOR REFERENCE AS TO HOW MANY mutiplications to do
                                // 0° straight toward camera ~ 90° orthogonal to camera (i.e. silhouette rim)
    const OUTER_FADE_ANGLE: f32 = 70.0_f32.to_radians(); // 70°~90° becomes the silhouette fade area
    let cos_fade_angle: f32 = OUTER_FADE_ANGLE.cos();

    let vertices = mesh.vertices();
    let mut alpha_buffer = vec![0u8; vertices.len()];
    // let mut silhouette_colors = mesh.colors().unwrap().to_vec();
    // let mut silhouette_colors = mesh.init_colors_mut().unwrap(); //TODO: all sorts of collecting and veccing stuff
    // let mut silhouette_colors = vec![Color::WHITE; vertices.len()];

    for i in mesh.triangles().iter_vertices() {
        // let model_center_to_vertex = normals[i]; // this is for like a cellshading feel? idk how it works
        let model_center_to_vertex = vertices[i].normalize_or_zero();
        let cos_theta = model_center_to_vertex.dot(model_center_to_camera);
        if cos_theta <= 0.0 {
            // silhouette_colors[i].a = 0; // BACKFACING VERTICES
            alpha_buffer[i] = 0;
            continue;
        }
        let fade_scalar = (cos_theta / cos_fade_angle).clamp(0.0, 1.0);
        // let alpha = fade_scalar.powf(FADE_RATE);
        let alpha = fade_scalar * fade_scalar * fade_scalar * fade_scalar; //powf 4
                                                                           // silhouette_colors[i].a = (alpha * 255.0).round() as u8;
        alpha_buffer[i] = (alpha * 255.0).round() as u8;
    }

    // mesh.colors_mut().unwrap().copy_from_slice(&silhouette_colors);
    let colors = mesh.colors_mut().unwrap();
    for i in 0..alpha_buffer.len() {
        colors[i].a = alpha_buffer[i];
    }
}

#[inline]
pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
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
    let mut radius = (camera.position.x * camera.position.x
        + camera.position.y * camera.position.y
        + camera.position.z * camera.position.z)
        .sqrt();
    let mut azimuth = camera.position.z.atan2(camera.position.x);
    let horizontal_radius = (camera.position.x * camera.position.x + camera.position.z * camera.position.z).sqrt();
    let mut elevation = camera.position.y.atan2(horizontal_radius);
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
    elevation = elevation.clamp(-std::f32::consts::PI / 2.0 + 0.1, std::f32::consts::PI / 2.0 - 0.1);
    radius = radius.clamp(0.25, 10.0);
    camera.position.x = radius * elevation.cos() * azimuth.cos();
    camera.position.y = radius * elevation.sin();
    camera.position.z = radius * elevation.cos() * azimuth.sin();
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
    //rotate in the xy plane
    let rotated_c = rotated_x_component + rotated_y_component + ac_z_component;
    origin + rotated_c
}
