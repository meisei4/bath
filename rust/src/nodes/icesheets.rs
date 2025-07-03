use crate::render::godot::GodotRenderer;
use crate::render::renderer::{Renderer, RendererVector2};
use crate::resource_paths::ResourcePaths;
use godot::classes::{INode2D, Node, Node2D, Texture2D};
use godot::meta::ToGodot;
use godot::obj::{Base, Gd, NewAlloc, WithBaseField};
use godot::prelude::{godot_api, GodotClass};
use std::f32::consts::SQRT_2;

#[derive(GodotClass)]
#[class(init, base=Node2D)]
pub struct IceSheetsRenderer {
    base: Base<Node2D>,
    render: Option<Gd<GodotRenderer>>,
    buffer_a_shader: Option<<GodotRenderer as Renderer>::Shader>,
    i_time: f32,
}

#[godot_api]
impl INode2D for IceSheetsRenderer {
    fn process(&mut self, delta: f64) {
        self.i_time += delta as f32;
        let mut render = self.render.as_mut().unwrap().bind_mut();
        let buffer_a_shader = self.buffer_a_shader.as_mut().unwrap();
        render.set_uniform_float(buffer_a_shader, "iTime", self.i_time);
        render.update_mask(self.i_time);
    }

    fn ready(&mut self) {
        let mut render = GodotRenderer::new_alloc();
        self.base_mut().add_child(&render.clone().upcast::<Node>());
        self.render = Some(render.clone());
        let mut render = render.bind_mut();
        let i_resolution = render.init_i_resolution();
        let mut buffer_a = render.init_render_target(i_resolution, true);
        let mut buffer_a_shader = render.load_shader_fragment(ResourcePaths::ICE_SHEETS_SHADER_FULL);
        render.set_uniform_vec2(&mut buffer_a_shader, "iResolution", i_resolution);
        render.set_uniform_float(&mut buffer_a_shader, "parallaxDepth", 6.0);
        render.set_uniform_float(&mut buffer_a_shader, "strideLength", 1.0);
        render.set_uniform_float(&mut buffer_a_shader, "globalCoordinateScale", 180.0);
        render.set_uniform_vec2(
            &mut buffer_a_shader,
            "noiseScrollVelocity",
            RendererVector2 { x: 0.0, y: 0.01 },
        );
        render.set_uniform_float(&mut buffer_a_shader, "uniformStretchCorrection", SQRT_2);
        render.set_uniform_float(&mut buffer_a_shader, "stretchScalarY", 2.0);
        render.set_uniform_vec2(
            &mut buffer_a_shader,
            "noiseCoordinateOffset",
            RendererVector2 { x: 2.0, y: 0.0 },
        );
        render.set_uniform_float(&mut buffer_a_shader, "parallaxNearScale", 0.025);
        render.draw_shader_texture(&mut buffer_a_shader, &mut buffer_a);
        let buffer_a_texture = buffer_a.get_texture().unwrap().upcast::<Texture2D>();

        let mut buffer_b = render.init_render_target(i_resolution, false);
        let mut buffer_b_shader = render.load_shader_fragment(ResourcePaths::FREE_ALPHA_CHANNEL);
        render.set_uniform_sampler2d(&mut buffer_b_shader, "iChannel0", &buffer_a_texture);
        render.draw_shader_texture(&mut buffer_b_shader, &mut buffer_b);
        render.draw_screen(&mut buffer_b);

        self.buffer_a_shader = Some(buffer_a_shader);
        self.i_time = 0.0;
        render.update_mask(self.i_time);
        //TODO: get this the fuck out of here, no more singleton collision bulllshit move to rust
        let scene_tree = self.base().get_tree().unwrap();
        let root_window = scene_tree.get_root().unwrap();
        let mut mask_manager = root_window.get_node_as::<Node>("MaskManager");
        mask_manager.call("register_ice_sheets", &[self.to_gd().to_variant()]);
    }
}
