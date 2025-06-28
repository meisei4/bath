use bath::render::raylib::RaylibRenderer;
use bath::render::raylib_util::{BATH_HEIGHT, BATH_WIDTH};
use bath::render::{renderer::Renderer, renderer::RendererVector2};
use bath_resources::glsl::{DEBUG_FRAG_PATH, DEBUG_VERT_PATH};

fn main() {
    let mut render = RaylibRenderer::init(BATH_WIDTH, BATH_HEIGHT);
    let i_resolution = RendererVector2::new(
        render.handle.get_screen_width() as f32,
        render.handle.get_screen_height() as f32,
    );
    let mut buffer = render.init_render_target(i_resolution, true);
    let mut shader = render.load_shader(DEBUG_VERT_PATH, DEBUG_FRAG_PATH);
    render.set_uniform_vec2(&mut shader, "iResolution", i_resolution);
    render.set_uniform_float(&mut shader, "depthScalar", 6.0);
    render.set_uniform_int(&mut shader, "tileSize", 4);
    render.set_uniform_float(&mut shader, "zigzagAmplitude", 1.25);

    while !render.handle.window_should_close() {
        let i_time = render.handle.get_time() as f32;
        render.set_uniform_float(&mut shader, "iTime", i_time);
        render.draw_shader_screen(&mut shader, &mut buffer);
    }
}
