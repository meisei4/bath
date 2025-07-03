use crate::render::godot::GodotRenderer;
use crate::render::renderer::Renderer;
use crate::render::renderer::RendererVector2;
use crate::resource_paths::ResourcePaths;
use godot::builtin::Vector2;
use godot::classes::{INode2D, Node, Node2D};
use godot::obj::{Base, Gd, NewAlloc, WithBaseField};
use godot::prelude::{godot_api, GodotClass};

const EMPTY: &str = "";

#[derive(GodotClass)]
#[class(init, base=Node2D)]
pub struct DrekkerRenderer {
    base: Base<Node2D>,
    render: Option<Gd<GodotRenderer>>,
}

#[godot_api]
impl INode2D for DrekkerRenderer {
    fn ready(&mut self) {
        let mut render_smart_pointer = GodotRenderer::new_alloc();
        self.base_mut()
            .add_child(&render_smart_pointer.clone().upcast::<Node>());
        self.render = Some(render_smart_pointer.clone());
        let mut render = render_smart_pointer.bind_mut();
        let scene_tree = self.base().get_tree().unwrap();
        let root_window = scene_tree.get_root().unwrap();
        let resolution_manager = root_window.get_node_as::<Node>("ResolutionManager");
        let godot_resolution = resolution_manager.get("resolution").try_to::<Vector2>().unwrap();
        let i_resolution = RendererVector2::new(godot_resolution.x, godot_resolution.y);
        let mut buffer_a = render.init_render_target(i_resolution, true);
        let mut shader = render.load_shader(EMPTY, ResourcePaths::DREKKER_EFFECT);
        let mut texture = render.load_texture(ResourcePaths::ICEBERGS_JPG);
        render.set_uniform_vec2(&mut shader, "iResolution", i_resolution);
        render.set_uniform_sampler2d(&mut shader, "iChannel0", &texture);
        render.draw_texture(&mut texture, &mut buffer_a);
        render.draw_shader_screen(&mut shader, &mut buffer_a);
    }
}
