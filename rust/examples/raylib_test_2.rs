use bath::render::raylib::RaylibRenderer;
use bath::render::raylib_util::{EXPERIMENTAL_WINDOW_HEIGHT, EXPERIMENTAL_WINDOW_WIDTH};
use bath::render::{renderer::Renderer, renderer::RendererVector2};
use bath_resources::glsl::{DREKKER_PATH, RAYLIB_DEFAULT_VERT_PATH};
use bath_resources::textures::ICEBERGS_JPG;

fn main() {
    let mut render = RaylibRenderer::init(EXPERIMENTAL_WINDOW_WIDTH, EXPERIMENTAL_WINDOW_HEIGHT);
    let width = render.handle.get_screen_width() as f32;
    let height = render.handle.get_screen_height() as f32;
    let i_resolution = RendererVector2::new(width, height);
    let mut buffer_a = render.init_render_target(i_resolution, true);
    let mut texture = render.load_texture(ICEBERGS_JPG);
    let mut shader = render.load_shader(RAYLIB_DEFAULT_VERT_PATH, DREKKER_PATH);
    render.set_uniform_vec2(&mut shader, "iResolution", i_resolution);
    render.set_uniform_sampler2d(&mut shader, "iChannel0", &texture);
    render.draw_texture(&mut texture, &mut buffer_a);
    while !render.handle.window_should_close() {
        render.draw_shader_screen(&mut shader, &mut buffer_a);
    }
}
