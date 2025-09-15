use bath::fixed_func::jugemu::apply_barycentric_palette;
use bath::fixed_func::silhouette::{ANGULAR_VELOCITY, FOVY_PERSPECTIVE, MODEL_POS, MODEL_SCALE};
use bath::render::raylib::RaylibRenderer;
use bath::render::raylib_util::N64_WIDTH;
use bath::render::renderer::Renderer;
use raylib::camera::Camera3D;
use raylib::color::Color;
use raylib::consts::CameraProjection;
use raylib::drawing::{RaylibDraw, RaylibDraw3D, RaylibMode3DExt};
use raylib::ffi::{rlSetLineWidth, rlSetPointSize};
use raylib::math::Vector3;
use raylib::models::{Mesh, RaylibMesh, RaylibModel, WeakMesh};

fn main() {
    let mut i_time = 0.0f32;
    let mut mesh_rotation = 0.0f32;
    let mut render = RaylibRenderer::init(N64_WIDTH, N64_WIDTH);
    // let main_observer = Camera3D {
    //     position: Vector3::new(0.0, 0.0, 2.0),
    //     target: Vector3::ZERO,
    //     up: Vector3::Y,
    //     fovy: FOVY_ORTHOGRAPHIC,
    //     projection: CameraProjection::CAMERA_ORTHOGRAPHIC,
    // };
    let main_observer = Camera3D {
        position: Vector3::new(0.0, 0.0, 2.0),
        target: Vector3::ZERO,
        up: Vector3::Y,
        fovy: FOVY_PERSPECTIVE,
        projection: CameraProjection::CAMERA_PERSPECTIVE,
    };
    // let mut main_model = render.handle.load_model(&render.thread, SPHERE_PATH).unwrap();
    // let mut cube_mesh = Mesh::try_gen_mesh_cube(&render.thread, 1.0, 1.0, 1.0).unwrap();
    //
    let mut cube_mesh = Mesh::try_gen_mesh_cube(&render.thread, 1.0, 1.0, 1.0).unwrap();
    dbg_mesh("gen", &*cube_mesh);
    let vertices = cube_mesh.vertices().to_vec();
    let indices = cube_mesh.indices().unwrap().to_vec();
    let mut rebuilt = Mesh::init_mesh(&vertices)
        .indices(&indices)
        .build(&render.thread)
        .unwrap();
    dbg_mesh("rebuilt", &*rebuilt);
    let mut main_model = render
        .handle
        .load_model_from_mesh(&render.thread, unsafe { rebuilt.make_weak() })
        .unwrap();
    dbg_mesh("model", &*main_model.meshes()[0]);
    apply_barycentric_palette(&mut main_model.meshes_mut()[0]);
    dump_vertices(&main_model.meshes()[0]);
    dump_indices(&main_model.meshes()[0]);
    dump_colors(&main_model.meshes()[0]);
    // let mut silhouette_img = generate_silhouette_texture(N64_WIDTH, N64_WIDTH);
    // dither(&mut silhouette_img);
    // let silhouette_texture = render
    //     .handle
    //     .load_texture_from_image(&render.thread, &silhouette_img)
    //     .unwrap();
    // main_model.materials_mut()[0].maps_mut()[MATERIAL_MAP_ALBEDO as usize].texture = *silhouette_texture;
    while !render.handle.window_should_close() {
        i_time += render.handle.get_frame_time();
        mesh_rotation -= ANGULAR_VELOCITY * render.handle.get_frame_time();
        let mut draw_handle = render.handle.begin_drawing(&render.thread);
        draw_handle.clear_background(Color::BLACK);
        draw_handle.draw_mode3D(main_observer, |mut rl3d| {
            rl3d.draw_model_ex(
                &main_model,
                MODEL_POS,
                Vector3::Y,
                mesh_rotation.to_degrees(),
                MODEL_SCALE,
                Color::WHITE,
            );
            unsafe { rlSetLineWidth(2.0) };
            rl3d.draw_model_wires_ex(
                &main_model,
                MODEL_POS,
                Vector3::Y,
                mesh_rotation.to_degrees(),
                MODEL_SCALE,
                Color::BLACK,
            );
            unsafe { rlSetPointSize(8.0) };
            rl3d.draw_model_points_ex(
                &main_model,
                MODEL_POS,
                Vector3::Y,
                mesh_rotation.to_degrees(),
                MODEL_SCALE,
                Color::GREEN,
            );
        });
    }
}

fn dbg_mesh(tag: &str, m: &raylib::ffi::Mesh) {
    println!(
        "{tag}: vtx={}, tris={}, indexed={}",
        m.vertexCount,
        m.triangleCount,
        (!m.indices.is_null())
    );
    if !m.indices.is_null() {
        let n = (m.triangleCount as usize) * 3;
        let idx: &[u16] = unsafe { std::slice::from_raw_parts(m.indices, n) };
        for (t, tri) in idx.chunks_exact(3).take(6).enumerate() {
            println!("  tri[{t:02}] = ({}, {}, {})", tri[0], tri[1], tri[2]);
        }
    }
}

fn dump_normals(mesh: &WeakMesh) {
    let ptr = mesh.normals;
    if ptr.is_null() {
        println!("(no normals array: GL_NORMAL_ARRAY should be disabled)");
        return;
    }
    let n_floats = mesh.vertexCount as usize * 3;
    let normals_f32: &[f32] = unsafe { std::slice::from_raw_parts(ptr, n_floats) };
    for (i, xyz) in normals_f32.chunks_exact(3).enumerate() {
        println!("#{i:04}: ({:.6}, {:.6}, {:.6})", xyz[0], xyz[1], xyz[2]);
    }
    let bad = normals_f32
        .chunks_exact(3)
        .enumerate()
        .find(|(_, v)| !(v[0].is_finite() && v[1].is_finite() && v[2].is_finite()));
    if let Some((i, _)) = bad {
        eprintln!("Found non-finite normal at vertex #{i}");
    }
}

fn dump_colors(mesh: &WeakMesh) {
    let ptr = mesh.colors;
    if ptr.is_null() {
        println!("(no colors array: GL_COLOR_ARRAY should be disabled)");
        return;
    }
    let n_bytes = mesh.vertexCount as usize * 4;
    let colors_u8: &[u8] = unsafe { std::slice::from_raw_parts(ptr, n_bytes) };
    for (i, rgba) in colors_u8.chunks_exact(4).enumerate() {
        println!(
            "#{i:04}: (R: {}, G: {}, B: {}, A: {})",
            rgba[0], rgba[1], rgba[2], rgba[3]
        );
    }
}

fn dump_vertices(mesh: &WeakMesh) {
    let n = mesh.vertexCount as usize;
    let ptr = mesh.vertices;
    let f: &[f32] = unsafe { std::slice::from_raw_parts(ptr, n * 3) };
    for (i, xyz) in f.chunks_exact(3).enumerate() {
        println!("v[{i:02}] = ({:.3}, {:.3}, {:.3})", xyz[0], xyz[1], xyz[2]);
    }
}

fn dump_indices(mesh: &WeakMesh) {
    if mesh.indices.is_null() {
        println!("(unindexed)");
        return;
    }
    let n = mesh.triangleCount as usize * 3;
    let idx: &[u16] = unsafe { std::slice::from_raw_parts(mesh.indices, n) };
    for t in 0..mesh.triangleCount as usize {
        let i0 = idx[t * 3] as usize;
        let i1 = idx[t * 3 + 1] as usize;
        let i2 = idx[t * 3 + 2] as usize;
        println!("tri[{t:02}] = ({i0}, {i1}, {i2})");
    }
}
