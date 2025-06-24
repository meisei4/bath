use bath::render::raylib::RaylibRenderer;
use bath::render::{renderer::Renderer, renderer::RendererVector2};
use bath_resources::glsl::DREKKER_PATH;
use bath_resources::textures::ICEBERGS_JPG;

fn main() {
    let mut render = RaylibRenderer::init();
    let i_resolution = RendererVector2::new(
        render.handle.get_render_width() as f32,
        render.handle.get_render_height() as f32,
    );
    let mut buffer_a = render.init_render_target(i_resolution, true);
    let mut shader = render.load_shader(DREKKER_PATH);
    let texture = render.load_texture(ICEBERGS_JPG);
    render.set_uniform_vec2(&mut shader, "iResolution", i_resolution);
    render.set_uniform_sampler2d(&mut shader, "iChannel0", &texture);
    render.draw_texture(&texture, &mut buffer_a);
    while !render.handle.window_should_close() {
        render.draw_shader_screen(&mut shader, &mut buffer_a);
    }
    render.draw_screen(&buffer_a);
}
