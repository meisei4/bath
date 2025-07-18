#[cfg(all(feature = "raylib", not(feature = "godot")))]
pub use raylib::math::Matrix as RendererMatrix;
#[cfg(all(feature = "raylib", not(feature = "godot")))]
pub use raylib::math::Vector2 as RendererVector2;
#[cfg(all(feature = "raylib", not(feature = "godot")))]
pub use raylib::math::Vector3 as RendererVector3;
#[cfg(all(feature = "raylib", not(feature = "godot")))]
pub use raylib::math::Vector4 as RendererVector4;

#[cfg(all(feature = "godot", not(feature = "raylib")))]
pub use godot::builtin::Variant as RendererMatrix;
#[cfg(all(feature = "godot", not(feature = "raylib")))]
pub use godot::builtin::Vector2 as RendererVector2;
#[cfg(all(feature = "godot", not(feature = "raylib")))]
pub use godot::builtin::Vector3 as RendererVector3;
#[cfg(all(feature = "godot", not(feature = "raylib")))]
pub use godot::builtin::Vector4 as RendererVector4;

#[cfg(all(feature = "raylib", feature = "godot"))]
pub use raylib::math::Matrix as RendererMatrix;
#[cfg(all(feature = "raylib", feature = "godot"))]
pub use raylib::math::Vector2 as RendererVector2;
#[cfg(all(feature = "raylib", feature = "godot"))]
pub use raylib::math::Vector3 as RendererVector3;
#[cfg(all(feature = "raylib", feature = "godot"))]
pub use raylib::math::Vector4 as RendererVector4;

pub trait Renderer {
    type RenderTarget;
    type Texture;
    type Shader;
    fn init(width: i32, height: i32) -> Self;
    fn init_i_resolution(&mut self) -> RendererVector2;
    fn update_mask(&mut self, i_time: f32);
    fn init_render_target(&mut self, size: RendererVector2, hdr: bool) -> Self::RenderTarget;
    fn load_texture_file_path(&mut self, path: &str) -> Self::Texture;
    fn load_texture(&mut self, data: &[u8], file_ext: &str) -> Self::Texture;
    fn tweak_texture_parameters(&mut self, texture: &mut Self::Texture, repeat: bool, nearest: bool);
    fn load_shader_fragment(&mut self, frag_path: &str) -> Self::Shader;
    fn load_shader_vertex(&mut self, vert_path: &str) -> Self::Shader;
    fn load_shader_full(&mut self, vert_src: &str, frag_src: &str) -> Self::Shader;
    fn set_uniform_float(&mut self, shader: &mut Self::Shader, uniform_name: &str, value: f32);
    fn set_uniform_int(&mut self, shader: &mut Self::Shader, uniform_name: &str, value: i32);
    fn set_uniform_vec2(&mut self, shader: &mut Self::Shader, uniform_name: &str, vec2: RendererVector2);
    fn set_uniform_vec3(&mut self, shader: &mut Self::Shader, uniform_name: &str, vec3: RendererVector3);
    fn set_uniform_vec4(&mut self, shader: &mut Self::Shader, uniform_name: &str, vec4: RendererVector4);
    fn set_uniform_vec3_array(&mut self, shader: &mut Self::Shader, uniform_name: &str, vec3_array: &[RendererVector3]);
    fn set_uniform_mat2(&mut self, shader: &mut Self::Shader, uniform_name: &str, mat2: RendererMatrix);
    fn set_uniform_mat4(&mut self, shader: &mut Self::Shader, uniform_name: &str, mat4: RendererMatrix);
    fn set_uniform_sampler2d(&mut self, shader: &mut Self::Shader, uniform_name: &str, texture: &Self::Texture);
    fn draw_texture(&mut self, texture: &mut Self::Texture, render_target: &mut Self::RenderTarget);
    fn draw_shader_texture(&mut self, shader: &mut Self::Shader, render_target: &mut Self::RenderTarget);
    fn draw_screen(&mut self, render_target: &Self::RenderTarget);
    fn draw_shader_screen(&mut self, shader: &mut Self::Shader, render_target: &mut Self::RenderTarget);
    fn draw_shader_screen_pseudo_ortho_geom(
        &mut self,
        shader: &mut Self::Shader,
        render_target: &mut Self::RenderTarget,
    );
    fn draw_screen_pseudo_ortho_geom(&mut self, render_target: &mut Self::RenderTarget);
    fn draw_shader_screen_tilted_geom(
        &mut self,
        shader: &mut Self::Shader,
        render_target: &mut Self::RenderTarget,
        tilt: f32,
    );
    fn draw_fixedfunc_screen_pseudo_ortho_geom(&mut self, texture: &Self::Texture);

    fn init_feedback_buffer(
        &mut self,
        resolution: RendererVector2,
        feedback_pass_shader_src: &str,
        main_pass_shader_src: &str,
    ) -> FeedbackBufferContext<Self>
    where
        Self: Sized;

    fn render_feedback_pass(&mut self, context: &mut FeedbackBufferContext<Self>)
    where
        Self: Sized;
}

// TODO: ok be careful dont let this shit get out of hand with generics
pub struct FeedbackBufferContext<R: Renderer> {
    pub buffer_a: R::RenderTarget,
    pub buffer_b: R::RenderTarget,
    pub feedback_pass_shader: R::Shader,
    pub main_pass_shader: R::Shader,
}

impl<R: Renderer> FeedbackBufferContext<R> {
    pub fn swap(&mut self) {
        std::mem::swap(&mut self.buffer_a, &mut self.buffer_b);
    }
}
