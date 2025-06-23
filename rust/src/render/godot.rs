use crate::render::godot_util::create_buffer_viewport;
use crate::render::renderer::Renderer;
use crate::render::renderer::RendererVector2;
use crate::resource_paths::ResourcePaths;
use godot::builtin::{Vector2 as GodotVector2, Vector2i as GodotVector2i};
use godot::classes::{ColorRect, INode2D, Node, Node2D, ResourceLoader, TextureRect};
use godot::classes::{Shader, ShaderMaterial, SubViewport, Texture};
use godot::meta::ToGodot;
use godot::obj::{Base, Gd, NewAlloc, NewGd, WithBaseField};
use godot::prelude::{godot_api, GodotClass};

#[derive(GodotClass)]
#[class(init, base=Node2D)]
pub struct GodotRenderer {
    base: Base<Node2D>,
}

impl Renderer for GodotRenderer {
    type Shader = Gd<ShaderMaterial>;
    type Texture = Gd<Texture>;
    type RenderTarget = Gd<SubViewport>;

    fn init() -> Self {
        unreachable!("Godot instantiates this node; Renderer::init() is never called")
    }

    fn load_shader(&mut self, path: &str) -> Self::Shader {
        let shader = ResourceLoader::singleton().load(path).unwrap().cast::<Shader>();
        let mut shader_material = ShaderMaterial::new_gd();
        shader_material.set_shader(&shader);
        shader_material
    }

    fn load_texture(&mut self, path: &str) -> Self::Texture {
        ResourceLoader::singleton().load(path).unwrap().cast()
    }

    fn set_texture_params(_texture: &mut Self::Texture, _repeat: bool, _nearest: bool) {}

    fn create_render_target(&mut self, size: RendererVector2, hdr: bool) -> Self::RenderTarget {
        let mut subviewport_buffer = create_buffer_viewport(GodotVector2i::new(size.x as i32, size.y as i32));
        subviewport_buffer.set_use_hdr_2d(hdr);
        subviewport_buffer
    }

    fn set_uniform_vec2(shader: &mut Self::Shader, name: &str, vec2: RendererVector2) {
        shader.set_shader_parameter(name, &GodotVector2::new(vec2.x, vec2.y).to_variant());
    }

    fn set_uniform_texture(shader: &mut Self::Shader, name: &str, texture: &Self::Texture) {
        shader.set_shader_parameter(name, &texture.to_variant());
    }

    fn begin_texture(
        &mut self,
        size: RendererVector2,
        render_target: &mut Self::RenderTarget,
        texture: &Self::Texture,
    ) {
        self.base_mut().add_child(&*render_target);
    }

    fn begin_frame(&mut self, render_target: &Self::RenderTarget) -> bool {
        let mut main_image = TextureRect::new_alloc();
        main_image.set_texture(&render_target.get_texture().unwrap());
        main_image.set_flip_v(true);
        self.base_mut().add_child(&main_image);
        true
    }

    fn shader_draw(
        &mut self,
        size: RendererVector2,
        shader: &mut Self::Shader,
        render_target: &mut Self::RenderTarget,
    ) {
        let mut buffer_a_shader_node = ColorRect::new_alloc();
        let i_resolution = GodotVector2::new(size.x, size.y);
        buffer_a_shader_node.set_size(i_resolution);
        buffer_a_shader_node.set_material(&*shader);
        render_target.add_child(&buffer_a_shader_node);
    }
}

#[godot_api]
impl INode2D for GodotRenderer {
    fn ready(&mut self) {
        let scene_tree = self.base().get_tree().unwrap();
        let root_window = scene_tree.get_root().unwrap();
        let resolution_manager = root_window.get_node_as::<Node>("ResolutionManager");
        let godot_resolution = resolution_manager.get("resolution").try_to::<GodotVector2>().unwrap();
        let i_resolution = RendererVector2::new(godot_resolution.x, godot_resolution.y);
        let mut shader_material = <Self as Renderer>::load_shader(self, ResourcePaths::DREKKER_EFFECT);
        let texture = <Self as Renderer>::load_texture(self, ResourcePaths::ICEBERGS_JPG);
        <GodotRenderer as Renderer>::set_uniform_vec2(&mut shader_material, "iResolution", i_resolution);
        <GodotRenderer as Renderer>::set_uniform_texture(&mut shader_material, "iChannel0", &texture);
        let mut sub_viewport = <Self as Renderer>::create_render_target(self, i_resolution, true);
        <GodotRenderer as Renderer>::begin_texture(
            self,
            RendererVector2::new(0.0, 0.0),
            &mut sub_viewport,
            &Texture::new_gd(),
        );
        <GodotRenderer as Renderer>::shader_draw(self, i_resolution, &mut shader_material, &mut sub_viewport);
        <GodotRenderer as Renderer>::begin_frame(self, &sub_viewport);
    }
}
