use bath::audio_analysis::fftw::fftw_complex;
use bath::render::raylib::RaylibRenderer;
use bath::render::raylib_util::{EXPERIMENTAL_WINDOW_HEIGHT, EXPERIMENTAL_WINDOW_WIDTH};
use bath::render::{renderer::Renderer, renderer::RendererVector2};
use bath::sound_render::raylib::RaylibFFTTexture;
use bath::sound_render::sound_renderer::{FFTTexture, BUFFER_SIZE, SAMPLE_RATE};
use bath_resources::audio_godot::{SHADERTOY_MUSIC_EXPERIMENT_OGG, WAV_TEST};
use bath_resources::glsl::FFT_FRAG_PATH;

use raylib::core::audio::{RaylibAudio, Sound};
use raylib::ffi::{rlActiveTextureSlot, rlEnableTexture, LoadWaveSamples, UnloadWaveSamples};
use raylib::shaders::RaylibShader;
use raylib::texture::{RaylibRenderTexture2D, RaylibTexture2D};

fn main() {
    let mut render = RaylibRenderer::init(EXPERIMENTAL_WINDOW_WIDTH, EXPERIMENTAL_WINDOW_HEIGHT);
    let i_resolution = RendererVector2::new(
        render.handle.get_screen_width() as f32,
        render.handle.get_screen_height() as f32,
    );
    let mut buffer_a = render.init_render_target(i_resolution, true);
    let mut shader = render.load_shader_fragment(FFT_FRAG_PATH);
    render.set_uniform_vec2(&mut shader, "iResolution", i_resolution);
    let i_channel_0_location = shader.get_shader_location("iChannel0");
    let audio = RaylibAudio::init_audio_device().unwrap();
    audio.set_audio_stream_buffer_size_default((BUFFER_SIZE * 2_usize) as i32);
    let music = audio.new_music(WAV_TEST).unwrap();
    music.play_stream();
    let wave = audio.new_wave(WAV_TEST).unwrap();
    let channels = wave.channels() as usize;
    let frames = wave.frame_count() as usize;
    let total_f32 = frames * channels;
    let samples_ptr = unsafe { LoadWaveSamples(*wave) };
    let samples = unsafe { std::slice::from_raw_parts(samples_ptr, total_f32) }.to_vec();
    unsafe { UnloadWaveSamples(samples_ptr) };
    let mut fft = RaylibFFTTexture {
        plan: None,
        spectrum: vec![fftw_complex { re: 0.0, im: 0.0 }; BUFFER_SIZE],
    };
    let mut fft_data = vec![0.0_f32; BUFFER_SIZE];
    let mut fft_image = fft.init_audio_texture();
    let mut cursor = 0usize;
    let hop = BUFFER_SIZE / 2;
    let mut fft_texture = render
        .handle
        .load_texture_from_image(&render.thread, &fft_image)
        .unwrap();
    while !render.handle.window_should_close() {
        music.update_stream();
        for i in 0..BUFFER_SIZE {
            let src = (cursor + i) % frames;
            fft_data[i] = if channels == 1 {
                samples[src]
            } else {
                let l = samples[src * 2];
                let r = samples[src * 2 + 1];
                0.5 * (l + r)
            };
        }
        cursor = (cursor + hop) % frames;
        fft.update_audio_texture(&mut fft_data, &mut fft_image);
        let byte_count = fft_image.get_pixel_data_size();
        let data_ptr = unsafe { fft_image.data() } as *const u8;
        let pixel_bytes: &[u8] = unsafe { std::slice::from_raw_parts(data_ptr, byte_count) };
        fft_texture.update_texture(pixel_bytes).unwrap();
        render.draw_texture(&mut fft_texture, &mut buffer_a);
        unsafe {
            rlActiveTextureSlot(7);
            rlEnableTexture(buffer_a.texture().id);
        }
        shader.set_shader_value(i_channel_0_location, 7_i32);
        render.draw_shader_screen(&mut shader, &mut buffer_a);
    }
}
