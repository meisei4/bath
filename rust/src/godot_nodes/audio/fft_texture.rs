use crate::sound_render::godot::GodotFFTTexture;
use crate::sound_render::sound_renderer::FFTTexture;
use godot::builtin::PackedFloat32Array;
use godot::classes::{AudioEffectSpectrumAnalyzerInstance, INode2D, Image, ImageTexture, Node, Node2D};
use godot::obj::{Base, Gd, NewAlloc, WithBaseField};
use godot::register::{godot_api, GodotClass};

#[derive(GodotClass)]
#[class(init, base=Node2D)]
pub struct FFTTextureNode {
    base: Base<Node2D>,
    render: Option<Gd<GodotFFTTexture>>,
    fft_data: Option<PackedFloat32Array>,
    audio_image: Option<Gd<Image>>,
    spectrum: Option<Gd<AudioEffectSpectrumAnalyzerInstance>>,
    #[var]
    audio_texture: Option<Gd<ImageTexture>>,
}

#[godot_api]
impl INode2D for FFTTextureNode {
    fn process(&mut self, _delta: f64) {
        let mut render = self.render.as_mut().unwrap().bind_mut();
        let fft_data = self.fft_data.as_mut().unwrap();
        let audio_image = self.audio_image.as_mut().unwrap();
        let audio_texture = self.audio_texture.as_mut().unwrap();

        render.update_audio_texture(fft_data, audio_image);
        audio_texture.update(&audio_image.clone());
    }

    fn ready(&mut self) {
        let mut render = GodotFFTTexture::new_alloc();
        self.base_mut().add_child(&render.clone().upcast::<Node>());
        self.render = Some(render.clone());
        let mut render = render.bind_mut();
        let mut fft_data = PackedFloat32Array::new();
        render.resize_buffer(&mut fft_data);
        let image = render.init_audio_texture();
        self.spectrum = Some(render.fetch_spectrum_analyzer());
        self.fft_data = Some(fft_data);
        self.audio_image = Some(image.clone());
        self.audio_texture = ImageTexture::create_from_image(&image.clone());
    }
}
