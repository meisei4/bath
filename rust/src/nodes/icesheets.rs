use crate::render::godot::GodotRenderer;
use crate::render::renderer::Renderer;
use crate::render::renderer::RendererVector2;
use bath_resources::glsl::{ICESHEETS_FRAG_PATH, ICESHEETS_VERT_PATH};
use godot::builtin::Vector2;
use godot::classes::{INode2D, Node, Node2D};
use godot::obj::{Base, Gd, NewAlloc, WithBaseField};
use godot::prelude::{godot_api, GodotClass};
use std::f32::consts::SQRT_2;

#[derive(GodotClass)]
#[class(init, base=Node2D)]
pub struct RustyIceSheets {
    base: Base<Node2D>,
    render: Option<Gd<GodotRenderer>>,
}

#[godot_api]
impl INode2D for RustyIceSheets {
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
        //TODO: look into how vertex shaders work in godot
        let mut shader = render.load_shader(ICESHEETS_VERT_PATH, ICESHEETS_FRAG_PATH);
        render.set_uniform_vec2(&mut shader, "iResolution", i_resolution);
        render.set_uniform_float(&mut shader, "parallaxDepth", 6.0);
        render.set_uniform_float(&mut shader, "strideLength", 1.0);
        render.set_uniform_float(&mut shader, "globalCoordinateScale", 180.0);
        render.set_uniform_vec2(&mut shader, "noiseScrollVelocity", RendererVector2::new(0.0, 0.1));
        render.set_uniform_float(&mut shader, "uniformStretchCorrection", SQRT_2);
        render.set_uniform_float(&mut shader, "stretchScalarY", 2.0);
        render.set_uniform_vec2(&mut shader, "noiseCoordinateOffset", RendererVector2::new(2.0, 0.0));
        render.set_uniform_float(&mut shader, "parallaxNearScale", 0.025);
        //TODO: Figure out how to actually achieve HDR and WHEN to apply it
        //render.draw_texture(&mut Texture2D::new_gd(), &mut buffer_a);
        //render.draw_screen(&buffer_a);
        render.draw_shader_screen(&mut shader, &mut buffer_a);
    }
}
