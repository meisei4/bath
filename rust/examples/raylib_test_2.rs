use bath::render::raylib::RaylibRenderer;
use bath::render::{renderer::Renderer, renderer::RendererVector2};
use bath_resources::glsl::DREKKER_PATH;
use bath_resources::textures::ICEBERGS_JPG;

fn main() {
    let mut render = RaylibRenderer::init();
    let i_resolution = RendererVector2::new(
        render.handle.get_screen_width() as f32,
        render.handle.get_screen_height() as f32,
    );
    let mut buffer_a = render.init_render_target(i_resolution, true);
    let mut texture = render.load_texture(ICEBERGS_JPG);
    let mut shader = render.load_shader(DREKKER_PATH);
    render.set_uniform_vec2(&mut shader, "iResolution", i_resolution);
    render.set_uniform_sampler2d(&mut shader, "iChannel0", &texture);
    //TODO: some how the image is already getting repeated? im so confused
    //render.tweak_texture_parameters(&mut texture, true, true);
    render.draw_texture(&mut texture, &mut buffer_a);
    while !render.handle.window_should_close() {
        render.draw_shader_screen(&mut shader, &mut buffer_a);
    }
}
