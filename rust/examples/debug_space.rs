use bath::render::raylib::RaylibRenderer;
use bath::render::raylib_util::{BATH_HEIGHT, BATH_WIDTH};
use bath::render::{renderer::Renderer, renderer::RendererVector2};
use bath_resources::glsl::{DEBUG_FRAG_PATH, DEBUG_VERT_PATH};
use std::fs;
use std::time::SystemTime;

fn main() {
    let mut render = RaylibRenderer::init(BATH_WIDTH, BATH_HEIGHT);
    let width = render.handle.get_screen_width() as f32;
    let height = render.handle.get_screen_height() as f32;
    let i_resolution = RendererVector2::new(width, height);
    let mut buffer = render.init_render_target(i_resolution, true);
    let mut shader = render.load_shader_full(DEBUG_VERT_PATH, DEBUG_FRAG_PATH);
    render.set_uniform_vec2(&mut shader, "iResolution", i_resolution);
    //render.handle.set_matrix_modelview(&render.thread, mat_view);
    let mut vert_mod_time = get_file_mod_time(DEBUG_VERT_PATH);
    let mut frag_mod_time = get_file_mod_time(DEBUG_FRAG_PATH);
    while !render.handle.window_should_close() {
        let i_time = render.handle.get_time() as f32;
        render.set_uniform_float(&mut shader, "iTime", i_time);
        let new_vert_mod_time = get_file_mod_time(DEBUG_VERT_PATH);
        let new_frag_mod_time = get_file_mod_time(DEBUG_FRAG_PATH);
        if new_vert_mod_time != vert_mod_time || new_frag_mod_time != frag_mod_time {
            println!("Shader modified, reloading...");
            shader = render.load_shader_full(DEBUG_VERT_PATH, DEBUG_FRAG_PATH);
            render.set_uniform_vec2(&mut shader, "iResolution", i_resolution);
            vert_mod_time = new_vert_mod_time;
            frag_mod_time = new_frag_mod_time;
        }
        //render.draw_shader_screen(&mut shader, &mut buffer);
        //render.draw_shader_screen_pseudo_ortho_geom(&mut shader, &mut buffer);
        render.draw_shader_screen_tilted_geom(&mut shader, &mut buffer, 45.0_f32);
    }
}

fn get_file_mod_time(path: &str) -> SystemTime {
    fs::metadata(path)
        .and_then(|m| m.modified())
        .unwrap_or(SystemTime::UNIX_EPOCH)
}
