pub trait FFTTexture {
    type Image;
    type FFTData;
    type AudioEffect;
    fn resize_buffer(&mut self, fft_data: &mut Self::FFTData);
    fn init_audio_texture(&mut self) -> Self::Image;
    fn fetch_spectrum_analyzer(&mut self) -> Self::AudioEffect;
    fn update_audio_texture(
        &mut self,
        spectrum: &Self::AudioEffect,
        fft_data: &mut Self::FFTData,
        audio_texture: &mut Self::Image,
    );
}

pub trait WaveformTexture {
    type Image;
    type WaveformData;
    type AudioEffect;
    fn resize_buffer(&mut self, waveform_data: &mut Self::WaveformData);
    fn init_audio_texture(&mut self) -> Self::Image;
    fn fetch_waveform_capture(&mut self) -> Self::AudioEffect;
    fn update_audio_texture(
        &mut self,
        waveform_capture: &mut Self::AudioEffect,
        waveform_data: &mut Self::WaveformData,
        audio_texture: &mut Self::Image,
    );
}
