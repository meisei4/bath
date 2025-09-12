use asset_payload::SPHERE_PATH;
use bath::fixed_func::jugemu::{
    apply_barycentric_palette, draw_frustum, draw_near_plane_intersectional_disk_mesh, draw_near_plane_software_raster,
    draw_observed_axes,
};
use bath::fixed_func::silhouette::{ANGULAR_VELOCITY, FOVY_PERSPECTIVE, MODEL_SCALE};
use bath::fixed_func::topology::Topology;
use bath::render::raylib::RaylibRenderer;
use bath::render::raylib_util::N64_WIDTH;
use bath::render::renderer::Renderer;
use raylib::camera::Camera3D;
use raylib::color::Color;
use raylib::consts::CameraProjection;
use raylib::drawing::{RaylibDraw, RaylibDraw3D, RaylibMode3DExt};
use raylib::ffi::{rlSetLineWidth, rlSetPointSize};
use raylib::math::Vector3;
use raylib::models::{Mesh, RaylibMesh, RaylibModel};

pub const MODEL_POS_BACK: Vector3 = Vector3::new(0.0, 0.0, -2.0);
pub const OBSERVER_POS: Vector3 = Vector3::new(0.0, 0.0, 2.0);
pub const JUGEMU_POS_ISO: Vector3 = Vector3::new(3.0, 1.0, 3.0);

fn main() {
    let mut mesh_rotation = 0.0f32;
    let mut render = RaylibRenderer::init(N64_WIDTH, N64_WIDTH);

    let near_clip_plane: f32 = 1.0;
    let far_clip_plane: f32 = 3.0;

    let main_observer = Camera3D {
        position: OBSERVER_POS,
        target: Vector3::ZERO,
        up: Vector3::Y,
        fovy: FOVY_PERSPECTIVE,
        projection: CameraProjection::CAMERA_PERSPECTIVE,
    };

    let screen_w = render.handle.get_screen_width();
    let screen_h = render.handle.get_screen_height();
    let main_observer_aspect = screen_w as f32 / screen_h as f32;

    let jugemu = Camera3D {
        // position: OBSERVER_POS,
        position: JUGEMU_POS_ISO,
        target: Vector3::ZERO,
        up: Vector3::Y,
        fovy: FOVY_PERSPECTIVE,
        projection: CameraProjection::CAMERA_PERSPECTIVE,
    };

    let mut main_model = render.handle.load_model(&render.thread, SPHERE_PATH).unwrap();
    //TODO: this will automatically color the wire mesh... not sure a way around that in opengl,
    apply_barycentric_palette(&mut main_model.meshes_mut()[0]);

    let max_points = main_model.meshes()[0].vertexCount as usize;
    let initial = vec![Vector3::ZERO; max_points];
    let mut marker_mesh = Mesh::init_mesh(&initial).build(&render.thread).unwrap();
    let mut marker_model = render
        .handle
        .load_model_from_mesh(&render.thread, unsafe { marker_mesh.make_weak() })
        .unwrap();

    while !render.handle.window_should_close() {
        mesh_rotation -= ANGULAR_VELOCITY * render.handle.get_frame_time();

        let mut draw_handle = render.handle.begin_drawing(&render.thread);
        draw_handle.clear_background(Color::BLACK);

        draw_handle.draw_mode3D(jugemu, |mut rl3d| {
            unsafe { rlSetLineWidth(2.0) };
            rl3d.draw_model_wires_ex(
                &main_model,
                MODEL_POS_BACK,
                Vector3::Y,
                mesh_rotation.to_degrees(),
                MODEL_SCALE,
                Color::RED,
            );
            unsafe { rlSetPointSize(6.0) };
            rl3d.draw_model_points_ex(
                &main_model,
                MODEL_POS_BACK,
                Vector3::Y,
                mesh_rotation.to_degrees(),
                MODEL_SCALE,
                Color::GREEN,
            );

            draw_observed_axes(&mut rl3d, &main_observer);
            draw_frustum(
                &mut rl3d,
                &main_observer,
                main_observer_aspect,
                near_clip_plane,
                far_clip_plane,
            );
            let topology = Topology::build_topology(&main_model.meshes()[0])
                .triangles()
                .vertices_per_triangle()
                .vertex_normals()
                .welded_vertices()
                .welded_vertices_per_triangle()
                .neighbors_per_triangle()
                .front_triangles(mesh_rotation, &main_observer)
                .silhouette_triangles()
                .build();

            draw_near_plane_intersectional_disk_mesh(
                &mut rl3d,
                &main_observer,
                near_clip_plane,
                &mut marker_model,
                MODEL_POS_BACK,
                MODEL_SCALE,
                mesh_rotation,
                &topology,
            );

            draw_near_plane_software_raster(
                &mut rl3d,
                &main_observer,
                main_observer_aspect,
                near_clip_plane,
                screen_w,
                screen_h,
                &main_model.meshes()[0],
                MODEL_POS_BACK,
                MODEL_SCALE,
                mesh_rotation,
                1,
            );
        });
    }
}
