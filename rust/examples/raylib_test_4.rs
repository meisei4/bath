use bath::render::raylib::RaylibRenderer;
use bath::render::raylib_util::{BATH_HEIGHT, BATH_WIDTH};
use bath::render::{renderer::Renderer, renderer::RendererVector2};
use bath_resources::glsl::{ICE_SHEETS_PATH, RAYLIB_DEFAULT_VERT_PATH};

fn main() {
    let mut render = RaylibRenderer::init(BATH_WIDTH, BATH_HEIGHT);
    let screen_size = RendererVector2::new(
        render.handle.get_screen_width() as f32,
        render.handle.get_screen_height() as f32,
    );
    let mut buffer = render.init_render_target(screen_size, true);
    let mut shader = render.load_shader(ICE_SHEETS_PATH, RAYLIB_DEFAULT_VERT_PATH);
    render.set_uniform_vec2(&mut shader, "iResolution", screen_size);
    while !render.handle.window_should_close() {
        let t = render.handle.get_time() as f32;
        render.set_uniform_float(&mut shader, "iTime", t);
        render.draw_shader_screen(&mut shader, &mut buffer);
    }
}
