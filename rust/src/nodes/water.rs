use crate::render::godot::GodotRenderer;
use crate::render::renderer::{Renderer, RendererVector2};
use crate::resource_paths::ResourcePaths;
use godot::builtin::Vector2;
use godot::classes::{INode2D, Node, Node2D, Texture2D};
use godot::obj::{Base, Gd, NewAlloc, WithBaseField};
use godot::prelude::{godot_api, GodotClass};

const EMPTY: &str = "";

#[derive(GodotClass)]
#[class(init, base=Node2D)]
pub struct WaterProjectedRenderer {
    base: Base<Node2D>,
    render_smart_pointer: Option<Gd<GodotRenderer>>,
    buffer_a: Option<<GodotRenderer as Renderer>::RenderTarget>,
    buffer_b: Option<<GodotRenderer as Renderer>::RenderTarget>,
    buffer_a_shader: Option<<GodotRenderer as Renderer>::Shader>,
    buffer_b_shader: Option<<GodotRenderer as Renderer>::Shader>,
}

#[godot_api]
impl INode2D for WaterProjectedRenderer {
    fn ready(&mut self) {
        let mut render_smart_pointer = GodotRenderer::new_alloc();
        self.base_mut()
            .add_child(&render_smart_pointer.clone().upcast::<Node>());
        self.render_smart_pointer = Some(render_smart_pointer.clone());
        let mut render = render_smart_pointer.bind_mut();
        let scene_tree = self.base().get_tree().unwrap();
        let root_window = scene_tree.get_root().unwrap();
        let resolution_manager = root_window.get_node_as::<Node>("ResolutionManager");
        let godot_resolution = resolution_manager.get("resolution").try_to::<Vector2>().unwrap();
        let i_resolution = RendererVector2::new(godot_resolution.x, godot_resolution.y);
        let mut buffer_a = render.init_render_target(i_resolution, true);
        let mut buffer_b = render.init_render_target(i_resolution, false);
        self.buffer_a = Some(buffer_a.clone());
        self.buffer_b = Some(buffer_b.clone());

        let mut buffer_a_shader = render.load_shader(EMPTY, ResourcePaths::FINITE_APPROX_RIPPLE);
        let mut buffer_b_shader = render.load_shader(EMPTY, ResourcePaths::WATER_PROJECTED_SHADER);
        self.buffer_a_shader = Some(buffer_a_shader.clone());
        self.buffer_b_shader = Some(buffer_b_shader.clone());
        render.set_uniform_vec2(&mut buffer_a_shader, "iResolution", i_resolution);
        render.set_uniform_vec2(&mut buffer_b_shader, "iResolution", i_resolution);

        let i_channel0 = render.load_texture(ResourcePaths::GRAY_NOISE_SMALL_PNG);
        let i_channel1 = render.load_texture(ResourcePaths::MOON_WATER_PNG);
        let i_channel2 = render.load_texture(ResourcePaths::PEBBLES_PNG);
        render.set_uniform_sampler2d(&mut buffer_b_shader, "iChannel0", &i_channel0);
        render.set_uniform_sampler2d(&mut buffer_b_shader, "iChannel1", &i_channel1);
        render.set_uniform_sampler2d(&mut buffer_b_shader, "iChannel2", &i_channel2);

        render.draw_shader_texture(&mut buffer_a_shader, &mut buffer_a);
        let buffer_a_texture = buffer_a.get_texture().unwrap().upcast::<Texture2D>();
        render.set_uniform_sampler2d(&mut buffer_b_shader, "iChannel3", &buffer_a_texture);
        render.draw_shader_texture(&mut buffer_b_shader, &mut buffer_b);
        render.draw_screen(&mut buffer_b);
    }

    fn process(&mut self, _delta: f64) {
        if let (Some(render_smart_pointer), Some(buffer_a), Some(buffer_b_shader)) = (
            self.render_smart_pointer.as_mut(),
            self.buffer_a.as_ref(),
            self.buffer_b_shader.as_mut(),
        ) {
            let mut render = render_smart_pointer.bind_mut();
            let i_channel3 = buffer_a.get_texture().unwrap().upcast::<Texture2D>();
            render.set_uniform_sampler2d(buffer_b_shader, "iChannel3", &i_channel3);
        }
    }
}
