use asset_payload::payloads::{BUFFER_A, IMAGE};
use bath::render::raylib::RaylibRenderer;
use bath::render::raylib_util::{N64_HEIGHT, N64_WIDTH};
use bath::render::renderer::{Renderer, RendererVector2};

fn main() {
    let mut render = RaylibRenderer::init(N64_WIDTH, N64_HEIGHT);
    let i_resolution = RendererVector2::new(
        render.handle.get_screen_width() as f32,
        render.handle.get_screen_height() as f32,
    );
    let mut context = render.init_feedback_buffer(i_resolution, BUFFER_A(), IMAGE());
    while !render.handle.window_should_close() {
        let i_time = render.handle.get_time() as f32;
        render.set_uniform_float(&mut context.feedback_pass_shader, "iTime", i_time);
        render.render_feedback_pass(&mut context);
    }
}
