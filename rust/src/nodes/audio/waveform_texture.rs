use crate::sound_render::godot::GodotWaveformTexture;
use crate::sound_render::sound_renderer::WaveformTexture;
use godot::builtin::PackedFloat32Array;
use godot::classes::{AudioEffectCapture, INode2D, Image, ImageTexture, Node, Node2D};
use godot::obj::{Base, Gd, NewAlloc, WithBaseField};
use godot::register::{godot_api, GodotClass};

#[derive(GodotClass)]
#[class(init, base=Node2D)]
pub struct WaveformTextureNode {
    base: Base<Node2D>,
    render: Option<Gd<GodotWaveformTexture>>,
    waveform_data: Option<PackedFloat32Array>,
    audio_image: Option<Gd<Image>>,
    waveform_capture: Option<Gd<AudioEffectCapture>>,

    #[var]
    audio_texture: Option<Gd<ImageTexture>>,
}

#[godot_api]
impl INode2D for WaveformTextureNode {
    fn process(&mut self, _delta: f64) {
        let mut render = self.render.as_mut().unwrap().bind_mut();
        let mut waveform_capture = self.waveform_capture.as_mut().unwrap();
        let waveform_data = self.waveform_data.as_mut().unwrap();
        let audio_image = self.audio_image.as_mut().unwrap();
        let audio_texture = self.audio_texture.as_mut().unwrap();

        render.update_audio_texture(&mut waveform_capture, waveform_data, audio_image);
        audio_texture.update(&audio_image.clone());
    }

    fn ready(&mut self) {
        let mut render = GodotWaveformTexture::new_alloc();
        self.base_mut().add_child(&render.clone().upcast::<Node>());
        self.render = Some(render.clone());
        let mut render = render.bind_mut();
        let mut waveform_data = PackedFloat32Array::new();
        render.resize_buffer(&mut waveform_data);
        let image = render.init_audio_texture();

        self.waveform_capture = Some(render.fetch_waveform_capture());
        self.waveform_data = Some(waveform_data);
        self.audio_image = Some(image.clone());
        self.audio_texture = ImageTexture::create_from_image(&image.clone());
    }
}
