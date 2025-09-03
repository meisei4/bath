use asset_payload::SPHERE_PATH;
// use bath::fixed_func::papercraft::unfold;
use bath::fixed_func::papercraft::{billow_unfolded, fold, unfold, BillowWaveParameters, PeriodicWhipParameters, WhipPulseParameters};
use bath::fixed_func::silhouette::{collect_deformed_mesh_samples, FOVY};
use bath::fixed_func::silhouette::interpolate_between_deformed_meshes;
use bath::fixed_func::silhouette::{ANGULAR_VELOCITY, MODEL_POS, MODEL_SCALE, TIME_BETWEEN_SAMPLES};
use bath::fixed_func::topology::{debug_draw_faces, ensure_drawable};
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
    let mut i_time = 0.0f32;
    let mut mesh_rotation = 0.0f32;
    let mut render = RaylibRenderer::init(N64_WIDTH, N64_WIDTH);

    // let observer = Camera3D {
    //     position: Vector3::new(1.8, 1.1, 2.4), // <- not head-on
    //     target: Vector3::ZERO,
    //     up: Vector3::Y,
    //     fovy: 45.0, // <- perspective FoV
    //     projection: CameraProjection::CAMERA_PERSPECTIVE,
    // };

    let observer = Camera3D {
        position: Vector3::new(0.0, 0.0, 2.0),
        target: Vector3::ZERO,
        up: Vector3::Y,
        fovy: FOVY,
        projection: CameraProjection::CAMERA_ORTHOGRAPHIC,
    };
    let mut wire_model = render.handle.load_model(&render.thread, SPHERE_PATH).unwrap();
    ensure_drawable(&mut wire_model.meshes_mut()[0]);
    let mesh_samples = collect_deformed_mesh_samples(&mut render);
    interpolate_between_deformed_meshes(&mut wire_model, i_time, &mesh_samples);
    let billow_params = BillowWaveParameters {
        amplitude_radians: 0.15,
        wavelength_in_unfolded_space: 2.0,
        speed_cycles_per_second: 0.5,
        phase_offset_radians: 0.0,
        depth_attenuation_per_breadth_first_step: 0.05,
        hinge_length_weight: 0.0,
    };
    let whip_params = WhipPulseParameters {
        maximum_rotation_radians: 0.28,
        pulse_front_speed_units_per_second: 1.4,
        pulse_width_sigma_in_unfolded_space: 0.30,
        depth_attenuation_per_breadth_first_step: 0.04,
        hinge_length_weight: 0.15,
        launch_delay_seconds: 0.0,
        ..Default::default()
    };

    let params = PeriodicWhipParameters {
        amplitude_radians: 0.40,
        wavelength_in_unfolded_space: 1.0,
        speed_cycles_per_second: 1.2,
        phase_offset_radians: 0.0,
        front_sharpness_exponent: 2.2,
        use_half_wave_rectification: true,
        depth_attenuation_per_breadth_first_step: 0.03,
        hinge_length_weight: 0.15,
        launch_delay_seconds: 0.0,
        ..Default::default()
    };

    while !render.handle.window_should_close() {
        i_time += render.handle.get_frame_time();
        mesh_rotation -= ANGULAR_VELOCITY * render.handle.get_frame_time();
        let duration = mesh_samples.len() as f32 * TIME_BETWEEN_SAMPLES;
        let time = i_time % duration;
        let frame = time / TIME_BETWEEN_SAMPLES;
        let current_frame = frame.floor() as usize % mesh_samples.len();
        interpolate_between_deformed_meshes(
            &mut wire_model,
            i_time,
            // (current_frame as f32 * TIME_BETWEEN_SAMPLES).floor(),
            &mesh_samples,
        );
        let unfolded_mesh = unsafe { fold(&mut wire_model.meshes_mut()[0], i_time, true).make_weak() };
        // let unfolded_mesh = unsafe { unfold(&mut wire_model.meshes_mut()[0]).make_weak() };
        // let unfolded_mesh =
        //     unsafe { billow_unfolded(&mut wire_model.meshes_mut()[0], i_time, true, &billow_params).make_weak() };
        // let unfolded_mesh =
        //     unsafe { whip_pulse_unfolded(&mut wire_model.meshes_mut()[0], i_time, true, &whip_params).make_weak() };
        // let unfolded_mesh =
        //     unsafe { periodic_whip_unfolded(&mut wire_model.meshes_mut()[0], i_time, true, &params).make_weak() };
        let unfolded_model = render
            .handle
            .load_model_from_mesh(&render.thread, unfolded_mesh.clone())
            .unwrap();
        let mut draw_handle = render.handle.begin_drawing(&render.thread);
        draw_handle.clear_background(Color::BLACK);
        {
            let mut rl3d = draw_handle.begin_mode3D(observer);
            // rl3d.draw_model_wires_ex(&unfolded_model, MODEL_POS, Vector3::Y, 0.0, MODEL_SCALE, Color::WHITE);
        }
        let all_faces: Vec<usize> = (0..unfolded_mesh.triangleCount as usize).collect();
        debug_draw_faces(
            observer,
            &mut draw_handle,
            &unfolded_mesh,
            mesh_rotation,
            &*all_faces,
            None,
            false,
        );
    }
}
