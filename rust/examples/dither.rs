use asset_payload::payloads::{BAYER_PNG, GHOST_FRAG};
use bath::render::raylib::RaylibRenderer;
use bath::render::raylib_util::{EXPERIMENTAL_WINDOW_HEIGHT, EXPERIMENTAL_WINDOW_WIDTH};
use bath::render::{renderer::Renderer, renderer::RendererVector2};

fn main() {
    let mut render = RaylibRenderer::init(EXPERIMENTAL_WINDOW_WIDTH, EXPERIMENTAL_WINDOW_HEIGHT);
    let i_resolution = RendererVector2::new(
        render.handle.get_screen_width() as f32,
        render.handle.get_screen_height() as f32,
    );
    let mut buffer_a = render.init_render_target(i_resolution, true);
    let mut shader = render.load_shader_fragment(GHOST_FRAG());
    render.set_uniform_vec2(&mut shader, "iResolution", i_resolution);

    let mut i_channel0 = render.load_texture(BAYER_PNG(), "png");
    render.tweak_texture_parameters(&mut i_channel0, true, true);
    render.set_uniform_sampler2d(&mut shader, "iChannel0", &i_channel0);

    let mut i_time = 0.0_f32;
    while !render.handle.window_should_close() {
        let delta_time = render.handle.get_frame_time();
        i_time += delta_time;
        render.set_uniform_float(&mut shader, "iTime", i_time);
        render.draw_shader_screen(&mut shader, &mut buffer_a);
    }
}
