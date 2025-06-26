use bath::render::raylib::RaylibRenderer;
use bath::render::raylib_util::{BATH_HEIGHT, BATH_WIDTH};
use bath::render::{renderer::Renderer, renderer::RendererVector2};
use bath_resources::glsl::{RAYLIB_DEBUG_FRAG_PATH, RAYLIB_DEBUG_VERT_PATH};

fn main() {
    let mut render = RaylibRenderer::init(BATH_WIDTH, BATH_HEIGHT);
    let i_resolution = RendererVector2::new(
        render.handle.get_screen_width() as f32,
        render.handle.get_screen_height() as f32,
    );
    let mut buffer = render.init_render_target(i_resolution, true);
    let mut shader = render.load_shader(RAYLIB_DEBUG_VERT_PATH, RAYLIB_DEBUG_FRAG_PATH);
    while !render.handle.window_should_close() {
        render.draw_shader_screen(&mut shader, &mut buffer);
    }
}
