use crate::render::godot::GodotRenderer;
use crate::render::renderer::Renderer;
use asset_payload::ResourcePaths;
use godot::classes::{INode2D, Node, Node2D};
use godot::obj::{Base, Gd, NewAlloc, WithBaseField};
use godot::prelude::{godot_api, GodotClass};

#[derive(GodotClass)]
#[class(init, base=Node2D)]
pub struct DrekkerRenderer {
    base: Base<Node2D>,
    render: Option<Gd<GodotRenderer>>,
}

#[godot_api]
impl INode2D for DrekkerRenderer {
    fn ready(&mut self) {
        let mut render = GodotRenderer::new_alloc();
        self.base_mut().add_child(&render.clone().upcast::<Node>());
        self.render = Some(render.clone());
        let mut render = render.bind_mut();
        let i_resolution = render.init_i_resolution();
        let mut buffer_a = render.init_render_target(i_resolution, true);
        let mut shader = render.load_shader_fragment(ResourcePaths::DREKKER_GDSHADER);
        render.set_uniform_vec2(&mut shader, "iResolution", i_resolution);

        let mut texture = render.load_texture_file_path(ResourcePaths::ICEBERGS_JPG_PATH);
        render.draw_texture(&mut texture, &mut buffer_a);
        render.set_uniform_sampler2d(&mut shader, "iChannel0", &texture);
        render.draw_shader_screen(&mut shader, &mut buffer_a);
    }
}
