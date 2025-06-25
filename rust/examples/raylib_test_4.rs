use bath::render::raylib::RaylibRenderer;
use bath::render::raylib_util::{BATH_HEIGHT, BATH_WIDTH};
use bath::render::{renderer::Renderer, renderer::RendererVector2};
use bath_resources::glsl::{ICESHEETS_FRAG_PATH, ICESHEETS_VERT_PATH};
use raylib::math::Vector2;
use std::f32::consts::SQRT_2;

fn main() {
    let mut render = RaylibRenderer::init(BATH_WIDTH, BATH_HEIGHT);
    let screen_size = RendererVector2::new(
        render.handle.get_screen_width() as f32,
        render.handle.get_screen_height() as f32,
    );
    let mut buffer = render.init_render_target(screen_size, true);
    let mut shader = render.load_shader(ICESHEETS_FRAG_PATH, ICESHEETS_VERT_PATH);
    render.set_uniform_vec2(&mut shader, "iResolution", screen_size);
    render.set_uniform_float(&mut shader, "parallaxDepth", 6.0);
    render.set_uniform_float(&mut shader, "strideLength", 1.0);
    render.set_uniform_float(&mut shader, "globalCoordinateScale", 180.0);
    render.set_uniform_vec2(&mut shader, "noiseScrollVelocity", Vector2::new(0.0, 0.1));
    render.set_uniform_float(&mut shader, "uniformStretchCorrection", SQRT_2);
    render.set_uniform_float(&mut shader, "stretchScalarY", 2.0);
    render.set_uniform_vec2(&mut shader, "noiseCoordinateOffset", Vector2::new(2.0, 0.0));
    render.set_uniform_float(&mut shader, "parallaxNearScale", 0.025);
    while !render.handle.window_should_close() {
        let t = render.handle.get_time() as f32;
        render.set_uniform_float(&mut shader, "iTime", t);
        render.draw_shader_screen(&mut shader, &mut buffer);
    }
}
