use bath::audio_analysis::fftw::fftw_complex;
use bath::render::raylib::RaylibRenderer;
use bath::render::raylib_util::{EXPERIMENTAL_WINDOW_HEIGHT, EXPERIMENTAL_WINDOW_WIDTH};
use bath::render::{renderer::Renderer, renderer::RendererVector2};
use bath::sound_render::raylib::RaylibFFTTexture;
use bath::sound_render::sound_renderer::{FFTTexture, BUFFER_SIZE};
use bath_resources::audio_godot::WAV_TEST;
use bath_resources::glsl::FFT_FRAG_PATH;
use raylib::core::audio::RaylibAudio;

fn main() {
    let mut render = RaylibRenderer::init(EXPERIMENTAL_WINDOW_WIDTH, EXPERIMENTAL_WINDOW_HEIGHT);
    let i_resolution = RendererVector2::new(
        render.handle.get_screen_width() as f32,
        render.handle.get_screen_height() as f32,
    );
    let mut buffer_a = render.init_render_target(i_resolution, true);
    let mut shader = render.load_shader_fragment(FFT_FRAG_PATH);
    render.set_uniform_vec2(&mut shader, "iResolution", i_resolution);
    let mut fft = RaylibFFTTexture {
        plan: None,
        spectrum: vec![fftw_complex { re: 0.0, im: 0.0 }; BUFFER_SIZE],
    };
    let mut fft_data = vec![0.0_f32; BUFFER_SIZE];
    let mut fft_image = fft.init_audio_texture();
    let mut fft_texture = render
        .handle
        .load_texture_from_image(&render.thread, &fft_image)
        .unwrap();
    render.set_uniform_sampler2d(&mut shader, "iChannel0", &fft_texture);

    let raylib_audio = RaylibAudio::init_audio_device().unwrap();
    let sample_byte_size = size_of::<f32>() as u32;
    let sample_bit_size = sample_byte_size * u8::BITS;
    let raw_audio_stream = raylib_audio.new_audio_stream(44_100_u32, sample_bit_size, 1_u32);
    // TODO: somehow get the raw audio data from the wav file but such that its in chunks alligned with the audiostream update rate
    //  which means same sample rate, same sample size, and same channel count??
    let wav_reader = hound::WavReader::open(WAV_TEST).unwrap();
    //TODO: can raylib do it? what?
    //let wav_raylib = WaveSamples;
    raw_audio_stream.play();
    //TODO: how do we initialize the buffer? do we need to? how do we read and fill everything correctly
    let raw_audio_data = ();
    while !render.handle.window_should_close() {
        // raw_audio_stream.update(raw_audio_data);
        fft.update_audio_texture(&mut fft_data, &mut fft_image);
        // [...]
        render.draw_shader_screen(&mut shader, &mut buffer_a);
    }
}
