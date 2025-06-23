use crate::render::godot_util::create_buffer_viewport;
use crate::render::renderer::Renderer;
use godot::builtin::{Vector2 as GodotVector2, Vector2i as GodotVector2i};
use godot::classes::ResourceLoader;
use godot::classes::{Shader, ShaderMaterial, SubViewport, Texture};
use godot::meta::ToGodot;
use godot::obj::{Gd, NewGd};
use raylib::math::Vector2;

pub struct GodotRenderer;

impl Renderer for GodotRenderer {
    type Shader = Gd<ShaderMaterial>;
    type Texture = Gd<Texture>;
    type RenderTarget = Gd<SubViewport>;

    fn init() -> Self {
        GodotRenderer
    }

    fn load_shader(&mut self, path: &str) -> Self::Shader {
        let shader: Gd<Shader> = ResourceLoader::singleton().load(path).unwrap().cast();
        let mut shader_material = ShaderMaterial::new_gd();
        shader_material.set_shader(&shader);
        shader_material
    }

    fn load_texture(&mut self, path: &str) -> Self::Texture {
        ResourceLoader::singleton().load(path).unwrap().cast()
    }

    fn set_texture_params(texture: &mut Self::Texture, _epeat: bool, nearest: bool) {}

    fn create_render_target(&mut self, size: Vector2, hdr: bool) -> Self::RenderTarget {
        let mut subviewport_buffer = create_buffer_viewport(GodotVector2i::new(size.x as i32, size.y as i32));
        subviewport_buffer.set_use_hdr_2d(hdr);
        subviewport_buffer
    }

    fn target_texture_for_render(render_target: &Self::RenderTarget) -> &Self::Texture {
        todo!()
        //render_target.get_texture().unwrap()
    }

    fn set_uniform_vec2(shader: &mut Self::Shader, name: &str, vec2: Vector2) {
        shader.set_shader_parameter(name, &GodotVector2::new(vec2.x, vec2.y).to_variant());
    }

    fn set_uniform_texture(shader: &mut Self::Shader, name: &str, texture: &Self::Texture) {
        shader.set_shader_parameter(name, &texture.to_variant());
    }

    fn begin_render(&mut self, render_target: &mut Self::RenderTarget) {}
    fn end_render(&mut self) {}
    fn begin_frame(&mut self) -> bool {
        true
    }
    fn draw(&mut self, shader: &mut Self::Shader, texture: &Self::Texture) {}
    fn end_frame(&mut self) {}
}
