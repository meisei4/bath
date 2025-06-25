use crate::render::godot_util::create_buffer_viewport;
use crate::render::renderer::Renderer;
use crate::render::renderer::RendererVector2;
use crate::resource_paths::ResourcePaths;
use godot::builtin::{real, Vector2, Vector2i};
use godot::classes::canvas_item::{TextureFilter, TextureRepeat};
use godot::classes::texture_rect::StretchMode;
use godot::classes::{ColorRect, INode2D, Node, Node2D, ResourceLoader, Texture2D, TextureRect};
use godot::classes::{Shader, ShaderMaterial, SubViewport};
use godot::meta::ToGodot;
use godot::obj::{Base, Gd, NewAlloc, NewGd, WithBaseField};
use godot::prelude::{godot_api, GodotClass};
use raylib::math::Matrix;

#[derive(GodotClass)]
#[class(init, base=Node2D)]
pub struct GodotRenderer {
    base: Base<Node2D>,
}

impl Renderer for GodotRenderer {
    type RenderTarget = Gd<SubViewport>;
    type Texture = Gd<Texture2D>;
    type Shader = Gd<ShaderMaterial>;

    fn init(_width: i32, _height: i32) -> Self {
        unreachable!("Godot instantiates this node; Renderer::init() is never called")
    }

    fn init_render_target(&mut self, size: RendererVector2, hdr: bool) -> Self::RenderTarget {
        let mut subviewport = create_buffer_viewport(Vector2i::new(size.x as i32, size.y as i32));
        subviewport.set_use_hdr_2d(hdr);
        subviewport
    }

    fn load_texture(&mut self, path: &str) -> Self::Texture {
        ResourceLoader::singleton().load(path).unwrap().cast()
    }

    fn tweak_texture_parameters(&mut self, _texture: &mut Self::Texture, _repeat: bool, _nearest: bool) {
        todo!()
        //TODO: Texture for GodotRenderer we some how need to get the CanvasItem
        // I think we have to use a TextureRect with its filter and repeat somehow
        // Otherwise use viewport but somehow make the rendertarget filter and repeat (but that breaks from how raylib does it)
    }

    fn load_shader(&mut self, frag_path: &str, _vert_path: &str) -> Self::Shader {
        let shader = ResourceLoader::singleton().load(frag_path).unwrap().cast::<Shader>();
        let mut shader_material = ShaderMaterial::new_gd();
        shader_material.set_shader(&shader);
        shader_material
    }

    fn set_uniform_float(&mut self, shader: &mut Self::Shader, name: &str, value: f32) {
        shader.set_shader_parameter(name, &value.to_variant());
    }

    fn set_uniform_vec2(&mut self, shader: &mut Self::Shader, name: &str, vec2: RendererVector2) {
        shader.set_shader_parameter(name, &Vector2::new(vec2.x, vec2.y).to_variant());
    }

    fn set_uniform_mat2(&mut self, _shader: &mut Self::Shader, _name: &str, _mat2: Matrix) {
        todo!()
    }

    fn set_uniform_sampler2d(&mut self, shader: &mut Self::Shader, name: &str, texture: &Self::Texture) {
        shader.set_shader_parameter(name, &texture.to_variant());
    }

    fn draw_texture(&mut self, texture: &mut Self::Texture, render_target: &mut Self::RenderTarget) {
        let mut buffer_a_node = TextureRect::new_alloc();
        buffer_a_node.set_stretch_mode(StretchMode::TILE);
        buffer_a_node.set_texture_filter(TextureFilter::NEAREST);
        buffer_a_node.set_texture_repeat(TextureRepeat::ENABLED);
        buffer_a_node.set_texture(&*texture);
        let i_resolution = Vector2::new(render_target.get_size().x as real, render_target.get_size().y as real);
        buffer_a_node.set_size(i_resolution);
        render_target.add_child(&buffer_a_node);
        self.base_mut().add_child(&*render_target);
    }

    fn draw_screen(&mut self, render_target: &Self::RenderTarget) {
        let mut main_image = TextureRect::new_alloc();
        main_image.set_texture_filter(TextureFilter::NEAREST);
        main_image.set_texture(&render_target.get_texture().unwrap());
        main_image.set_flip_v(true);
        self.base_mut().add_child(&main_image);
    }

    fn draw_shader_screen(&mut self, shader: &mut Self::Shader, render_target: &mut Self::RenderTarget) {
        let mut buffer_a_shader_node = ColorRect::new_alloc();
        let i_resolution = Vector2::new(render_target.get_size().x as real, render_target.get_size().y as real);
        buffer_a_shader_node.set_size(i_resolution);
        buffer_a_shader_node.set_texture_filter(TextureFilter::NEAREST);
        buffer_a_shader_node.set_material(&*shader);
        render_target.add_child(&buffer_a_shader_node);
        self.draw_screen(render_target);
    }
}

#[godot_api]
impl INode2D for GodotRenderer {
    fn ready(&mut self) {
        let scene_tree = self.base().get_tree().unwrap();
        let root_window = scene_tree.get_root().unwrap();
        let resolution_manager = root_window.get_node_as::<Node>("ResolutionManager");
        let godot_resolution = resolution_manager.get("resolution").try_to::<Vector2>().unwrap();
        let i_resolution = RendererVector2::new(godot_resolution.x, godot_resolution.y);
        let mut buffer_a = self.init_render_target(i_resolution, true);
        let mut shader = self.load_shader(ResourcePaths::DREKKER_EFFECT, "");
        let mut texture = self.load_texture(ResourcePaths::ICEBERGS_JPG);
        self.set_uniform_vec2(&mut shader, "iResolution", i_resolution);
        self.set_uniform_sampler2d(&mut shader, "iChannel0", &texture);
        self.draw_texture(&mut texture, &mut buffer_a);
        //self.draw_screen(&buffer_a);
        self.draw_shader_screen(&mut shader, &mut buffer_a);
    }
}
