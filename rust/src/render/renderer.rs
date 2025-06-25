#[cfg(all(feature = "raylib", not(feature = "godot")))]
pub use raylib::math::Vector2 as RendererVector2;

#[cfg(all(feature = "godot", not(feature = "raylib")))]
pub use godot::builtin::Vector2 as RendererVector2;

#[cfg(all(feature = "raylib", feature = "godot"))]
pub use raylib::math::Vector2 as RendererVector2;

pub trait Renderer {
    type RenderTarget;
    type Texture;
    type Shader;
    fn init(width: i32, height: i32) -> Self;
    fn init_render_target(&mut self, size: RendererVector2, hdr: bool) -> Self::RenderTarget;
    fn load_texture(&mut self, path: &str) -> Self::Texture;
    fn tweak_texture_parameters(&mut self, texture: &mut Self::Texture, repeat: bool, nearest: bool);
    fn load_shader(&mut self, vert_path: &str, frag_path: &str) -> Self::Shader;
    fn set_uniform_float(&mut self, shader: &mut Self::Shader, name: &str, value: f32);
    fn set_uniform_vec2(&mut self, shader: &mut Self::Shader, name: &str, vec2: RendererVector2);
    fn set_uniform_mat2(&mut self, shader: &mut Self::Shader, name: &str, mat2: &[RendererVector2]);
    fn set_uniform_sampler2d(&mut self, shader: &mut Self::Shader, name: &str, texture: &Self::Texture);
    fn draw_texture(&mut self, texture: &mut Self::Texture, render_target: &mut Self::RenderTarget);
    fn draw_screen(&mut self, render_target: &Self::RenderTarget);
    fn draw_shader_screen(&mut self, shader: &mut Self::Shader, render_target: &mut Self::RenderTarget);
}
