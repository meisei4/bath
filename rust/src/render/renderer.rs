#[cfg(all(feature = "raylib-render", not(feature = "godot")))]
pub use raylib::math::Vector2 as RendererVector2;

#[cfg(all(feature = "godot", not(feature = "raylib-render")))]
pub use godot::builtin::Vector2 as RendererVector2;

#[cfg(all(feature = "raylib-render", feature = "godot"))]
pub use raylib::math::Vector2 as RendererVector2;

pub trait Renderer {
    type Shader;
    type Texture;
    type RenderTarget;
    fn init() -> Self;
    fn load_shader(&mut self, path: &str) -> Self::Shader;
    fn load_texture(&mut self, path: &str) -> Self::Texture;
    fn set_texture_params(texture: &mut Self::Texture, repeat: bool, nearest: bool);
    fn create_render_target(&mut self, size: RendererVector2, hdr: bool) -> Self::RenderTarget;
    fn set_uniform_vec2(shader: &mut Self::Shader, name: &str, vec2: RendererVector2);
    fn set_uniform_texture(shader: &mut Self::Shader, name: &str, texture: &Self::Texture);
    fn begin_texture(&mut self, size: RendererVector2, render_target: &mut Self::RenderTarget, texture: &Self::Texture);
    fn begin_frame(&mut self, render_target: &Self::RenderTarget) -> bool;
    fn shader_draw(&mut self, size: RendererVector2, shader: &mut Self::Shader, render_target: &mut Self::RenderTarget);
}
