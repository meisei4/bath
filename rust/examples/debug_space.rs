use bath::render::raylib::RaylibRenderer;
use bath::render::raylib_util::{BATH_HEIGHT, BATH_WIDTH};
use bath::render::{renderer::Renderer, renderer::RendererVector2};
use bath_resources::glsl::{DEBUG_FRAG_PATH, DEBUG_VERT_PATH};
use std::f32::consts::SQRT_2;
use std::fs;
use std::time::SystemTime;

fn main() {
    let mut render = RaylibRenderer::init(BATH_WIDTH, BATH_HEIGHT);
    let width = render.handle.get_screen_width() as f32;
    let height = render.handle.get_screen_height() as f32;
    let i_resolution = RendererVector2::new(width, height);
    let mut buffer = render.init_render_target(i_resolution, true);
    let mut shader = render.load_shader(DEBUG_VERT_PATH, DEBUG_FRAG_PATH);
    render.set_uniform_vec2(&mut shader, "iResolution", i_resolution);
    //_set_static_uniforms(&mut render, &mut shader);
    let mut vert_mod_time = get_file_mod_time(DEBUG_VERT_PATH);
    let mut frag_mod_time = get_file_mod_time(DEBUG_FRAG_PATH);
    while !render.handle.window_should_close() {
        let i_time = render.handle.get_time() as f32;
        render.set_uniform_float(&mut shader, "iTime", i_time);
        let new_vert_mod_time = get_file_mod_time(DEBUG_VERT_PATH);
        let new_frag_mod_time = get_file_mod_time(DEBUG_FRAG_PATH);
        if new_vert_mod_time != vert_mod_time || new_frag_mod_time != frag_mod_time {
            println!("Shader modified, reloading...");
            shader = render.load_shader(DEBUG_VERT_PATH, DEBUG_FRAG_PATH);
            render.set_uniform_vec2(&mut shader, "iResolution", i_resolution);
            //_set_static_uniforms(&mut render, &mut shader);
            vert_mod_time = new_vert_mod_time;
            frag_mod_time = new_frag_mod_time;
        }
        //render.draw_shader_screen(&mut shader, &mut buffer);
        render.draw_shader_screen_pseudo_ortho_geom(&mut shader, &mut buffer);
        //render.draw_shader_screen_alt_geom(&mut shader, &mut buffer);
    }
}

fn _set_static_uniforms(render: &mut RaylibRenderer, shader: &mut <RaylibRenderer as Renderer>::Shader) {
    render.set_uniform_float(shader, "depthScalar", 6.0);
    //render.set_uniform_int(shader, "tileSize", 128);
    render.set_uniform_int(shader, "tileSize", 32);
    //render.set_uniform_int(shader, "tileSize", 8);
    render.set_uniform_float(shader, "zigzagAmplitude", 1.25);
    render.set_uniform_vec2(shader, "scrollVelocity", RendererVector2::new(0.0, 0.05));
    render.set_uniform_float(shader, "macroScale", 180.0);
    render.set_uniform_float(shader, "microScale", 0.025);
    render.set_uniform_float(shader, "uniformStretch", SQRT_2);
    render.set_uniform_float(shader, "stretchY", 2.0);
    render.set_uniform_vec2(shader, "staticOffset", RendererVector2::new(2.0, 0.0));
}

fn get_file_mod_time(path: &str) -> SystemTime {
    fs::metadata(path)
        .and_then(|m| m.modified())
        .unwrap_or(SystemTime::UNIX_EPOCH)
}
