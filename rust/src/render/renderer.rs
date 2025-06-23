use raylib::math::Vector2;

pub trait Renderer {
    type Shader;
    type Texture;
    type RenderTarget;
    fn init() -> Self;
    fn load_shader(&mut self, path: &str) -> Self::Shader;
    fn load_texture(&mut self, path: &str) -> Self::Texture;
    fn set_texture_params(texture: &mut Self::Texture, repeat: bool, nearest: bool);
    fn create_render_target(&mut self, size: Vector2, hdr: bool) -> Self::RenderTarget;
    fn target_texture_for_render(render_target: &Self::RenderTarget) -> &Self::Texture;
    fn set_uniform_vec2(shader: &mut Self::Shader, name: &str, vec2: Vector2);
    fn set_uniform_texture(shader: &mut Self::Shader, name: &str, texture: &Self::Texture);
    fn begin_render(&mut self, render_target: &mut Self::RenderTarget);
    fn end_render(&mut self);
    fn begin_frame(&mut self) -> bool;
    fn draw(&mut self, shader: &mut Self::Shader, texture: &Self::Texture);
    fn end_frame(&mut self);
}
