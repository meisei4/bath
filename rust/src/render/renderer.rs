#[cfg(all(feature = "raylib-render", not(feature = "godot")))]
pub use raylib::math::Vector2 as RendererVector2;

#[cfg(all(feature = "godot", not(feature = "raylib-render")))]
pub use godot::builtin::Vector2 as RendererVector2;

#[cfg(all(feature = "raylib-render", feature = "godot"))]
pub use raylib::math::Vector2 as RendererVector2;

pub trait Renderer {
    type RenderTarget;
    type Texture;
    type Shader;
    fn init() -> Self;
    fn init_render_target(&mut self, size: RendererVector2, hdr: bool) -> Self::RenderTarget;
    fn load_texture(&mut self, path: &str) -> Self::Texture;
    fn tweak_texture_parameters(&mut self, texture: &mut Self::Texture, repeat: bool, nearest: bool);
    fn load_shader(&mut self, path: &str) -> Self::Shader;
    fn set_uniform_vec2(&mut self, shader: &mut Self::Shader, name: &str, vec2: RendererVector2);
    fn set_uniform_sampler2d(&mut self, shader: &mut Self::Shader, name: &str, texture: &Self::Texture);
    fn draw_texture(&mut self, texture: &mut Self::Texture, render_target: &mut Self::RenderTarget);
    fn draw_screen(&mut self, render_target: &Self::RenderTarget);
    fn draw_shader_screen(&mut self, shader: &mut Self::Shader, render_target: &mut Self::RenderTarget);
}
