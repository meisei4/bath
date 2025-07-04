pub trait FFTTexture {
    type Image;
    type FFTData;
    type SpectrumAnalyzer;
    fn resize_buffer(&mut self, fft_data: &mut Self::FFTData);
    fn init_audio_texture(&mut self) -> Self::Image;
    fn fetch_spectrum_analyzer(&mut self) -> Self::SpectrumAnalyzer;
    fn update_audio_texture(
        &mut self,
        spectrum: &Self::SpectrumAnalyzer,
        fft_data: &mut Self::FFTData,
        audio_texture: &mut Self::Image,
    );
}

pub trait WaveformTexture {
    type Image;
    type WaveformData;
    type SpectrumAnalyzer;
    fn resize_buffer(&mut self, waveform_data: &mut Self::WaveformData);
    fn init_audio_texture(&mut self) -> Self::Image;
    fn fetch_spectrum_analyzer(&mut self) -> Self::SpectrumAnalyzer;
    fn update_audio_texture(
        &mut self,
        spectrum: &Self::SpectrumAnalyzer,
        waveform_data: &mut Self::WaveformData,
        audio_texture: &mut Self::Image,
    );
}
