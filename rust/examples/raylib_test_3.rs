use bath::render::raylib::RaylibRenderer;
use bath::render::raylib_util::{BATH_HEIGHT, BATH_WIDTH};
use bath::render::{renderer::Renderer, renderer::RendererVector2};
use bath_resources::glsl::{ICE_FRAG_2_PATH, ICE_VERT_2_PATH};
use raylib::math::Vector2;

fn main() {
    let mut render = RaylibRenderer::init(BATH_WIDTH, BATH_HEIGHT);
    let screen_size = RendererVector2::new(
        render.handle.get_screen_width() as f32,
        render.handle.get_screen_height() as f32,
    );
    let mut buffer = render.init_render_target(screen_size, true);
    let mut shader = render.load_shader(ICE_FRAG_2_PATH, ICE_VERT_2_PATH);
    render.set_uniform_vec2(&mut shader, "iResolution", screen_size);
    render.set_uniform_vec2(&mut shader, "uNoiseScrollVel", Vector2::new(0.0, 0.1));
    render.set_uniform_float(&mut shader, "uGlobalCoordScalar", 180.0);
    render.set_uniform_float(&mut shader, "uStretchY", 2.0);
    render.set_uniform_vec2(&mut shader, "uNoiseCoordOffset", Vector2::new(2.0, 0.0));
    render.set_uniform_float(&mut shader, "uUnifStretchCorr", (2.0f32).sqrt());
    render.set_uniform_float(&mut shader, "uRotCos", (-std::f32::consts::FRAC_PI_4).cos());
    render.set_uniform_float(&mut shader, "uRotSin", (-std::f32::consts::FRAC_PI_4).sin());
    render.set_uniform_float(&mut shader, "uPerlinSolidTh", -0.03);
    render.set_uniform_float(&mut shader, "uWaterColR", 0.1);
    render.set_uniform_float(&mut shader, "uWaterColG", 0.7);
    render.set_uniform_float(&mut shader, "uWaterColB", 0.8);
    render.set_uniform_float(&mut shader, "uWaterDarkenMult", 0.5);
    render.set_uniform_float(&mut shader, "uWaterDepthDiv", 9.0);
    render.set_uniform_float(&mut shader, "uWaterStaticTh", 12.0);
    render.set_uniform_float(&mut shader, "uSolidBrightness", 0.9);
    render.set_uniform_float(&mut shader, "uParDepth", 6.0);
    render.set_uniform_float(&mut shader, "uParNearScale", 0.025);
    render.set_uniform_float(&mut shader, "uStrideLen", 1.0);
    while !render.handle.window_should_close() {
        let t = render.handle.get_time() as f32;
        render.set_uniform_float(&mut shader, "iTime", t);
        render.draw_shader_screen(&mut shader, &mut buffer);
    }
}
