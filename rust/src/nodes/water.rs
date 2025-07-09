use crate::render::godot::GodotRenderer;
use crate::render::renderer::Renderer;
use asset_loader::runtime_io::{FINITE_APPROX_RIPPLE_GDSHADER, WATER_PROJECTED_GDSHADER};
use asset_loader::ResourcePaths;
use godot::classes::{INode2D, Node, Node2D, Texture2D};
use godot::obj::{Base, Gd, NewAlloc, WithBaseField};
use godot::prelude::{godot_api, GodotClass};

#[derive(GodotClass)]
#[class(init, base=Node2D)]
pub struct WaterProjectedRenderer {
    base: Base<Node2D>,
    render: Option<Gd<GodotRenderer>>,
    buffer_a: Option<<GodotRenderer as Renderer>::RenderTarget>,
    buffer_b_shader: Option<<GodotRenderer as Renderer>::Shader>,
}

#[godot_api]
impl INode2D for WaterProjectedRenderer {
    fn process(&mut self, _delta: f64) {
        let mut render = self.render.as_mut().unwrap().bind_mut();
        let buffer_a = self.buffer_a.as_ref().unwrap();
        let buffer_b_shader = self.buffer_b_shader.as_mut().unwrap();
        let i_channel3 = buffer_a.get_texture().unwrap().upcast::<Texture2D>();
        render.set_uniform_sampler2d(buffer_b_shader, "iChannel3", &i_channel3);
    }

    fn ready(&mut self) {
        let mut render = GodotRenderer::new_alloc();
        self.base_mut().add_child(&render.clone().upcast::<Node>());
        self.render = Some(render.clone());

        let mut render = render.bind_mut();
        let i_resolution = render.init_i_resolution();
        let mut buffer_a = render.init_render_target(i_resolution, true);
        let mut buffer_a_shader = render.load_shader_fragment(FINITE_APPROX_RIPPLE_GDSHADER);
        render.set_uniform_vec2(&mut buffer_a_shader, "iResolution", i_resolution);
        render.draw_shader_texture(&mut buffer_a_shader, &mut buffer_a);
        let buffer_a_texture = buffer_a.get_texture().unwrap().upcast::<Texture2D>();

        let mut buffer_b = render.init_render_target(i_resolution, false);
        let mut buffer_b_shader = render.load_shader_fragment(WATER_PROJECTED_GDSHADER);
        let i_channel0 = render.load_texture(ResourcePaths::GRAY_NOISE_SMALL_PNG_PATH);
        let i_channel1 = render.load_texture(ResourcePaths::MOON_WATER_PNG_PATH);
        let i_channel2 = render.load_texture(ResourcePaths::PEBBLES_PNG_PATH);
        render.set_uniform_vec2(&mut buffer_b_shader, "iResolution", i_resolution);
        render.set_uniform_sampler2d(&mut buffer_b_shader, "iChannel0", &i_channel0);
        render.set_uniform_sampler2d(&mut buffer_b_shader, "iChannel1", &i_channel1);
        render.set_uniform_sampler2d(&mut buffer_b_shader, "iChannel2", &i_channel2);
        render.set_uniform_sampler2d(&mut buffer_b_shader, "iChannel3", &buffer_a_texture);
        render.draw_shader_texture(&mut buffer_b_shader, &mut buffer_b);
        render.draw_screen(&mut buffer_b);

        self.buffer_a = Some(buffer_a.clone());
        self.buffer_b_shader = Some(buffer_b_shader.clone());
    }
}
