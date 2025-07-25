use crate::godot_nodes::audio::waveform_texture::WaveformTextureNode;
use crate::render::godot::GodotRenderer;
use crate::render::renderer::Renderer;
use asset_payload::WAVEFORM_GDSHADER_PATH_GD;
use godot::classes::{INode2D, Node, Node2D, Texture2D};
use godot::obj::{Base, Gd, NewAlloc, WithBaseField};
use godot::prelude::{godot_api, GodotClass};

#[derive(GodotClass)]
#[class(init, base=Node2D)]
pub struct WaveformVisualizerRenderer {
    base: Base<Node2D>,
    render: Option<Gd<GodotRenderer>>,
}

#[godot_api]
impl INode2D for WaveformVisualizerRenderer {
    fn ready(&mut self) {
        let mut render = GodotRenderer::new_alloc();
        self.base_mut().add_child(&render.clone().upcast::<Node>());
        self.render = Some(render.clone());
        let mut render = render.bind_mut();
        let i_resolution = render.init_i_resolution();
        let mut buffer_a = render.init_render_target(i_resolution, true);
        let mut shader = render.load_shader_fragment(WAVEFORM_GDSHADER_PATH_GD);
        render.set_uniform_vec2(&mut shader, "iResolution", i_resolution);
        let waveform_texture = WaveformTextureNode::new_alloc();
        self.base_mut().add_child(&waveform_texture.clone().upcast::<Node>());
        let audio_texture = waveform_texture
            .bind()
            .get_audio_texture()
            .unwrap()
            .upcast::<Texture2D>();
        render.set_uniform_sampler2d(&mut shader, "iChannel0", &audio_texture);
        render.draw_shader_texture(&mut shader, &mut buffer_a);
        render.draw_screen(&mut buffer_a);
    }
}
