use asset_payload::payloads::{ICESHEETS_FRAG, ICESHEETS_VERT};
use asset_payload::{ICESHEETS_FRAG_PATH, ICESHEETS_VERT_PATH};
use bath::render::raylib::RaylibRenderer;
use bath::render::raylib_util::{BATH_HEIGHT, BATH_WIDTH};
use bath::render::{renderer::Renderer, renderer::RendererVector2};
use raylib::math::{Matrix, Vector2};
use std::f32::consts::{PI, SQRT_2};
use std::fs;
use std::time::SystemTime;

fn main() {
    let mut render = RaylibRenderer::init(BATH_WIDTH, BATH_HEIGHT);
    let i_resolution = RendererVector2::new(
        render.handle.get_screen_width() as f32,
        render.handle.get_screen_height() as f32,
    );
    let mut buffer = render.init_render_target(i_resolution, true);
    let mut shader = render.load_shader_full(ICESHEETS_VERT(), ICESHEETS_FRAG());
    let parallax_depth: f32 = 6.0;
    let global_coordinate_scale = 180.0;
    let uniform_stretch_correction = SQRT_2;
    let stretch_scalar_y = 2.0;
    let parallax_near_scale = 0.025;
    let inv_resolution_y = 1.0 / i_resolution.y;
    let a = 0.5 * ((parallax_depth + 1.0) / (parallax_depth - 1.0)).ln();
    let b = 1.5 * (parallax_depth * ((parallax_depth + 1.0) / (parallax_depth - 1.0)).ln() - 2.0);
    let rot = Matrix::rotate_z(-PI * 0.25);
    let near_scale = Matrix::scale(
        global_coordinate_scale * parallax_near_scale,
        global_coordinate_scale * parallax_near_scale,
        1.0,
    );
    let combined_linear_part_matrix = rot * near_scale;
    render.set_uniform_vec2(&mut shader, "iResolution", i_resolution);
    render.set_uniform_float(&mut shader, "globalCoordinateScale", global_coordinate_scale);
    render.set_uniform_vec2(&mut shader, "noiseScrollVelocity", Vector2::new(0.0, 0.05));
    render.set_uniform_float(&mut shader, "uniformStretchCorrection", uniform_stretch_correction);
    render.set_uniform_float(&mut shader, "stretchScalarY", stretch_scalar_y);
    render.set_uniform_vec2(&mut shader, "noiseCoordinateOffset", Vector2::new(2.0, 0.0));
    render.set_uniform_float(&mut shader, "invResolutionY", inv_resolution_y);
    render.set_uniform_float(&mut shader, "a", a);
    render.set_uniform_float(&mut shader, "b", b);
    render.set_uniform_mat4(
        &mut shader,
        "combinedLinearPart",
        combined_linear_part_matrix.transposed(),
    );
    #[cfg(not(feature = "nasa-embed"))]
    let mut vert_mod_time = get_file_mod_time(ICESHEETS_VERT_PATH);
    #[cfg(not(feature = "nasa-embed"))]
    let mut frag_mod_time = get_file_mod_time(ICESHEETS_FRAG_PATH);

    while !render.handle.window_should_close() {
        let t = render.handle.get_time() as f32;
        render.set_uniform_float(&mut shader, "iTime", t);
        render.draw_shader_screen(&mut shader, &mut buffer); //TODO: update to allow stretching render targets to the screen...
        #[cfg(not(feature = "nasa-embed"))]
        let new_vert_mod_time = get_file_mod_time(ICESHEETS_VERT_PATH);
        #[cfg(not(feature = "nasa-embed"))]
        let new_frag_mod_time = get_file_mod_time(ICESHEETS_FRAG_PATH);
        #[cfg(not(feature = "nasa-embed"))]
        if new_vert_mod_time != vert_mod_time || new_frag_mod_time != frag_mod_time {
            println!("Shader modified, reloading...");
            let vert_src = fs::read_to_string(ICESHEETS_VERT_PATH).unwrap();
            let frag_src = fs::read_to_string(ICESHEETS_FRAG_PATH).unwrap();
            let hot_vert_leaked = Box::leak(vert_src.into_boxed_str());
            let hot_frag_leaked = Box::leak(frag_src.into_boxed_str());
            shader = render.load_shader_full(hot_vert_leaked, hot_frag_leaked);
            render.set_uniform_vec2(&mut shader, "iResolution", i_resolution);
            render.set_uniform_float(&mut shader, "parallaxDepth", parallax_depth);
            render.set_uniform_float(&mut shader, "globalCoordinateScale", global_coordinate_scale);
            render.set_uniform_vec2(&mut shader, "noiseScrollVelocity", Vector2::new(0.0, 0.05));
            render.set_uniform_float(&mut shader, "uniformStretchCorrection", uniform_stretch_correction);
            render.set_uniform_float(&mut shader, "stretchScalarY", stretch_scalar_y);
            render.set_uniform_vec2(&mut shader, "noiseCoordinateOffset", Vector2::new(2.0, 0.0));
            render.set_uniform_float(&mut shader, "parallaxNearScale", parallax_near_scale);
            render.set_uniform_float(&mut shader, "invResolutionY", inv_resolution_y);
            render.set_uniform_float(&mut shader, "a", a);
            render.set_uniform_float(&mut shader, "b", b);
            render.set_uniform_mat2(&mut shader, "combinedLinearPart", combined_linear_part_matrix);
            vert_mod_time = new_vert_mod_time;
            frag_mod_time = new_frag_mod_time;
        }
    }
}

fn get_file_mod_time(path: &str) -> SystemTime {
    fs::metadata(path)
        .and_then(|m| m.modified())
        .unwrap_or(SystemTime::UNIX_EPOCH)
}
