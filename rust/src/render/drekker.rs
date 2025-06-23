use crate::render::godot_util::create_buffer_viewport;
use crate::resource_paths::ResourcePaths;
use godot::builtin::Vector2;
use godot::classes::{
    ColorRect, INode2D, Node, Node2D, ResourceLoader, SceneTree, Shader, ShaderMaterial, SubViewport, Texture,
    TextureRect, Window,
};
use godot::meta::ToGodot;
use godot::obj::{Base, Gd, NewAlloc, NewGd, WithBaseField};
use godot::register::{godot_api, GodotClass};

#[derive(GodotClass)]
#[class(base=Node2D)]
pub struct DrekkerColorRust {
    base: Base<Node2D>,
    pub buffer_a_shader: Gd<Shader>,
    pub i_channel_0: Gd<Texture>,
}

#[godot_api]
impl INode2D for DrekkerColorRust {
    fn init(base: Base<Node2D>) -> Self {
        let buffer_a_shader = ResourceLoader::singleton()
            .load(ResourcePaths::DREKKER_EFFECT)
            .unwrap()
            .cast::<Shader>();
        let i_channel_0 = ResourceLoader::singleton()
            .load(ResourcePaths::ICEBERGS_JPG)
            .unwrap()
            .cast::<Texture>();
        Self {
            base,
            buffer_a_shader,
            i_channel_0,
        }
    }

    fn ready(&mut self) {
        let tree: Gd<SceneTree> = self.base().get_tree().unwrap();
        let root_window: Gd<Window> = tree.get_root().unwrap();
        let resolution_manager = root_window.get_node_as::<Node>("ResolutionManager");
        let i_resolution: Vector2 = resolution_manager.get("resolution").try_to::<Vector2>().unwrap();

        let mut buffer_a: Gd<SubViewport> = create_buffer_viewport(i_resolution.cast_int());
        let mut buffer_a_shader_material = ShaderMaterial::new_gd();
        buffer_a_shader_material.set_shader(&self.buffer_a_shader);
        let mut buffer_a_shader_node = ColorRect::new_alloc();
        buffer_a_shader_node.set_size(i_resolution);
        buffer_a_shader_node.set_material(&buffer_a_shader_material);
        buffer_a_shader_material.set_shader_parameter("iResolution", &i_resolution.to_variant());
        buffer_a_shader_material.set_shader_parameter("iChannel0", &self.i_channel_0.to_variant());
        let mut main_image = TextureRect::new_alloc();
        main_image.set_texture(&buffer_a.get_texture().unwrap());
        main_image.set_flip_v(true);
        buffer_a.add_child(&buffer_a_shader_node);
        self.base_mut().add_child(&buffer_a);
        self.base_mut().add_child(&main_image);
    }
}
