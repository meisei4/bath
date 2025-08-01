use asset_payload::payloads::{DREKKER, ICEBERGS_JPG};
use bath::render::raylib::RaylibRenderer;
use bath::render::raylib_util::{EXPERIMENTAL_WINDOW_HEIGHT, EXPERIMENTAL_WINDOW_WIDTH};
use bath::render::{renderer::Renderer, renderer::RendererVector2};

fn main() {
    let mut render = RaylibRenderer::init(EXPERIMENTAL_WINDOW_WIDTH, EXPERIMENTAL_WINDOW_HEIGHT);
    let width = render.handle.get_screen_width() as f32;
    let height = render.handle.get_screen_height() as f32;
    let i_resolution = RendererVector2::new(width, height);
    let mut buffer_a = render.init_render_target(i_resolution, true);
    //TODO:WARNING: IMAGE: Data format not supported
    // WARNING: IMAGE: Failed to load image data
    // thread 'main' panicked at src/render/raylib.rs:92:64:
    // called `Result::unwrap()` on an `Err` value: NullDataFromMemory

    //no idea what is happening here...
    let mut texture = render.load_texture(ICEBERGS_JPG(), ".jpg");
    let mut shader = render.load_shader_fragment(DREKKER());
    render.set_uniform_vec2(&mut shader, "iResolution", i_resolution);
    render.set_uniform_sampler2d(&mut shader, "iChannel1", &texture);
    render.draw_texture(&mut texture, &mut buffer_a);
    while !render.handle.window_should_close() {
        render.draw_shader_screen(&mut shader, &mut buffer_a);
    }
}
