use crate::render::godot::GodotRenderer;
use crate::render::renderer::{FeedbackBufferContext, Renderer};
use asset_payload::ResourcePaths;
use godot::classes::{INode2D, Node, Node2D};
use godot::obj::{Base, Gd, NewAlloc, WithBaseField};
use godot::prelude::{godot_api, GodotClass};

#[derive(GodotClass)]
#[class(init, base=Node2D)]
pub struct FeedbackBuffer {
    base: Base<Node2D>,
    render: Option<Gd<GodotRenderer>>,
    context: Option<FeedbackBufferContext<GodotRenderer>>,
    i_time: f32,
}

#[godot_api]
impl INode2D for FeedbackBuffer {
    fn process(&mut self, delta: f64) {
        self.i_time += delta as f32;
        let mut render = self.render.as_mut().unwrap().bind_mut();
        let context = self.context.as_mut().unwrap();
        render.set_uniform_float(&mut context.feedback_pass_shader, "iTime", self.i_time);
        render.render_feedback_pass(context);
    }

    fn ready(&mut self) {
        let mut render = GodotRenderer::new_alloc();
        self.base_mut().add_child(&render.clone().upcast::<Node>());
        self.render = Some(render.clone());

        let mut render = render.bind_mut();
        let i_resolution = render.init_i_resolution();
        let context = render.init_feedback_buffer(
            i_resolution,
            ResourcePaths::BUFFER_A_GDSHADER,
            ResourcePaths::MAIN_GDSHADER,
        );

        self.context = Some(context);
        self.i_time = 0.0;
    }
}
