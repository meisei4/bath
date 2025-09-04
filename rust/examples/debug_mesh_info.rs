use asset_payload::SPHERE_PATH;
use bath::fixed_func::silhouette::{MODEL_POS, MODEL_SCALE};
use bath::render::raylib::RaylibRenderer;
use bath::render::raylib_util::N64_WIDTH;
use bath::render::renderer::Renderer;
use raylib::camera::Camera3D;
use raylib::color::Color;
use raylib::consts::CameraProjection;
use raylib::drawing::{RaylibDraw, RaylibDraw3D, RaylibMode3DExt};
use raylib::math::Vector3;
use raylib::models::RaylibModel;

fn main() {
    let mut render = RaylibRenderer::init(N64_WIDTH, N64_WIDTH);
    let main_observer = Camera3D {
        position: Vector3::new(0.0, 0.0, 2.0),
        target: Vector3::ZERO,
        up: Vector3::Y,
        fovy: 2.0,
        projection: CameraProjection::CAMERA_ORTHOGRAPHIC,
    };

    let mut main_model = render.handle.load_model(&render.thread, SPHERE_PATH).unwrap();
    let mut wire_model = render.handle.load_model(&render.thread, SPHERE_PATH).unwrap();

    for (i, m) in main_model.meshes_mut().iter_mut().enumerate() {
        println!(
            "mesh[{i}] vc={} tc={} ptrs: v={:?} n={:?} t={:?} idx={:?} animV={:?} animN={:?}",
            m.vertexCount,
            m.triangleCount,
            m.vertices,
            m.normals,
            m.texcoords,
            m.indices,
            m.animVertices,
            m.animNormals
        );
    }
    let normals = unsafe {
        std::slice::from_raw_parts(
            main_model.meshes_mut()[0].normals as *const Vector3,
            main_model.meshes_mut()[0].vertexCount as usize,
        )
    };
    for norm in normals {
        println!("norm triplet at norm={}", norm);
    }
    while !render.handle.window_should_close() {
        let mut draw_handle = render.handle.begin_drawing(&render.thread);
        draw_handle.clear_background(Color::BLACK);
        draw_handle.draw_mode3D(main_observer, |mut rl3d| {
            rl3d.draw_model_ex(&main_model, MODEL_POS, Vector3::Y, 0.0, MODEL_SCALE, Color::WHITE);
            rl3d.draw_model_wires_ex(&wire_model, MODEL_POS, Vector3::Y, 0.0, MODEL_SCALE, Color::BLACK);
        });
    }
}

//
// fn dump_normals(mesh: &WeakMesh) {
//     let ptr = mesh.normals;
//     if ptr.is_null() {
//         println!("(no normals array: GL_NORMAL_ARRAY should be disabled)");
//         return;
//     }
//     let n_floats = mesh.vertexCount as usize * 3;
//     let normals_f32: &[f32] = unsafe { std::slice::from_raw_parts(ptr, n_floats) };
//     for (i, xyz) in normals_f32.chunks_exact(3).enumerate() {
//         println!("#{i:04}: ({:.6}, {:.6}, {:.6})", xyz[0], xyz[1], xyz[2]);
//     }
//     let bad = normals_f32.chunks_exact(3).enumerate().find(|(_, v)| {
//         !(v[0].is_finite() && v[1].is_finite() && v[2].is_finite())
//     });
//     if let Some((i, _)) = bad {
//         eprintln!("Found non-finite normal at vertex #{i}");
//     }
// }
