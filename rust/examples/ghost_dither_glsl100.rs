use asset_payload::payloads::{BAYER_PNG, GHOST_DEBUG_FRAG};
use asset_payload::GHOST_DEBUG_FRAG_PATH;
use bath::render::raylib::RaylibRenderer;
use bath::render::raylib_util::N64_WIDTH;
use bath::render::{renderer::Renderer, renderer::RendererVector2};
use std::fs;
use std::time::SystemTime;

fn main() {
    let mut render = RaylibRenderer::init(N64_WIDTH, N64_WIDTH);
    let i_resolution = RendererVector2::new(
        render.handle.get_screen_width() as f32,
        render.handle.get_screen_height() as f32,
    );
    let mut buffer_a = render.init_render_target(i_resolution, false);
    let mut shader = render.load_shader_fragment(GHOST_DEBUG_FRAG());

    render.set_uniform_vec2(&mut shader, "iResolution", i_resolution);
    let i_channel0 = render.load_texture(BAYER_PNG(), ".png");
    let mut i_time = 0.0_f32;
    let mut frag_mod_time = get_file_mod_time(GHOST_DEBUG_FRAG_PATH);

    while !render.handle.window_should_close() {
        i_time += render.handle.get_frame_time();
        render.set_uniform_float(&mut shader, "iTime", i_time);
        render.draw_shader_screen(&mut shader, &mut buffer_a);
        let new_frag_mod_time = get_file_mod_time(GHOST_DEBUG_FRAG_PATH);
        if new_frag_mod_time != frag_mod_time {
            println!("Shader modified, reloading...");
            let frag_src = fs::read_to_string(GHOST_DEBUG_FRAG_PATH).unwrap();
            let hot_frag_leaked = Box::leak(frag_src.into_boxed_str());
            shader = render.load_shader_fragment(hot_frag_leaked);
            render.set_uniform_vec2(&mut shader, "iResolution", i_resolution);
            render.set_uniform_sampler2d(&mut shader, "iChannel0", &i_channel0);
            frag_mod_time = new_frag_mod_time;
        }
    }
}

fn get_file_mod_time(path: &str) -> SystemTime {
    fs::metadata(path)
        .and_then(|m| m.modified())
        .unwrap_or(SystemTime::UNIX_EPOCH)
}
