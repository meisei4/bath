use bath::render::raylib::RaylibRenderer;
use bath::render::{renderer::Renderer, renderer::RendererVector2};

fn main() {
    let mut renderer = <RaylibRenderer as Renderer>::init();
    let project_root_dir = env!("CARGO_MANIFEST_DIR");
    let glsl_dir = format!("{}/resources/glsl", project_root_dir);
    let path = &format!("{}/color/drekker_effect.glsl", glsl_dir);
    let mut shader = renderer.load_shader(path);
    let image_path = &"/../godot/Resources/textures/icebergs.jpg".to_string();
    let texture = renderer.load_texture(image_path);

    let resolution = RendererVector2::new(
        renderer.handle.get_render_width() as f32,
        renderer.handle.get_render_height() as f32,
    );
    <RaylibRenderer as Renderer>::set_uniform_vec2(&mut shader, "iResolution", resolution);
    <RaylibRenderer as Renderer>::set_uniform_texture(&mut shader, "iChannel0", &texture);

    let mut buffer_a = renderer.create_render_target(resolution, true);
    renderer.begin_texture(resolution, &mut buffer_a, &texture);
    while !renderer.handle.window_should_close() {
        renderer.shader_draw(resolution, &mut shader, &mut buffer_a);
    }
}
