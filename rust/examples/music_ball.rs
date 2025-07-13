use asset_payload::payloads::{BAYER_PNG, MIDI_FILE, MUSIC_BALL_FRAG_330, SOUND_FONT_FILE};
#[cfg(not(feature = "nasa-embed"))]
use asset_payload::CACHED_WAV_PATH;
use bath::midi::pitch::{PitchDimension, HSV_BUFFER_LEN};
use bath::render::raylib::RaylibRenderer;
use bath::render::raylib_util::{N64_HEIGHT, N64_WIDTH};
use bath::render::renderer::RendererVector3;
use bath::render::{renderer::Renderer, renderer::RendererVector2};
use bath::sound_render::raylib::RaylibFFTTexture;
use bath::sound_render::sound_renderer::{
    FFTTexture, AUDIO_STREAM_RING_BUFFER_SIZE, BUFFER_SIZE, FFT_HISTORICAL_SMOOTHING_BUFFER_TIME_SECONDS,
    FFT_WINDOW_SIZE, MONO, PER_SAMPLE_BIT_DEPTH_HARDCODED, RING_BUFFER_PADDING, SAMPLE_RATE_HARDCODED, WINDOW_TIME,
};
use fftw2_sys::fftw_complex;
use hound::WavReader;
use raylib::core::audio::RaylibAudio;
use raylib::texture::RaylibTexture2D;
use std::io::Cursor;
use std::slice::from_raw_parts;

fn main() {
    let mut pitch_dimension = PitchDimension::default();
    pitch_dimension.resolve_payload_to_midi_buffer(MIDI_FILE());

    #[cfg(not(feature = "nasa-embed"))]
    let wav_bytes = pitch_dimension.resolve_payload_to_pcm_buffer_cache(
        SAMPLE_RATE_HARDCODED as i32,
        MONO as u16,
        MIDI_FILE(),
        SOUND_FONT_FILE(),
        CACHED_WAV_PATH,
    );

    #[cfg(feature = "nasa-embed")]
    let wav_bytes = pitch_dimension.resolve_payload_to_pcm_buffer(
        SAMPLE_RATE_HARDCODED as i32,
        MONO as u16,
        MIDI_FILE,
        SOUND_FONT_FILE,
    );

    let mut render = RaylibRenderer::init(N64_WIDTH, N64_HEIGHT);
    let i_resolution = RendererVector2::new(
        render.handle.get_screen_width() as f32,
        render.handle.get_screen_height() as f32,
    );
    let mut buffer_a = render.init_render_target(i_resolution, true);
    #[cfg(feature = "glsl-100")]
    let mut shader = render.load_shader_fragment(MUSIC_BALL_FRAG_100());
    #[cfg(not(feature = "glsl-100"))]
    let mut shader = render.load_shader_fragment(MUSIC_BALL_FRAG_330());
    render.set_uniform_vec2(&mut shader, "iResolution", i_resolution);
    let mut i_channel0 = render.load_texture(BAYER_PNG(), "png");
    render.tweak_texture_parameters(&mut i_channel0, true, true);
    render.set_uniform_sampler2d(&mut shader, "iChannel0", &i_channel0);

    let fft_history_len: usize =
        (FFT_HISTORICAL_SMOOTHING_BUFFER_TIME_SECONDS as f64 / WINDOW_TIME).ceil() as usize + RING_BUFFER_PADDING;
    let mut fft = RaylibFFTTexture {
        plan: None,
        spectrum: [fftw_complex { re: 0.0, im: 0.0 }; FFT_WINDOW_SIZE],
        fft_history: vec![[0.0; BUFFER_SIZE]; fft_history_len],
        history_pos: 0_usize,
        last_fft_time: 0_f64,
        tapback_pos: 0.01_f32,
    };
    let mut fft_data = [0_f32; FFT_WINDOW_SIZE];
    let mut fft_image = fft.init_audio_texture();
    let mut fft_texture = render
        .handle
        .load_texture_from_image(&render.thread, &fft_image)
        .unwrap();
    render.set_uniform_sampler2d(&mut shader, "iChannel1", &fft_texture);

    let raylib_audio = RaylibAudio::init_audio_device().unwrap();
    raylib_audio.set_audio_stream_buffer_size_default(AUDIO_STREAM_RING_BUFFER_SIZE as i32);
    let mut audio_stream = raylib_audio.new_audio_stream(SAMPLE_RATE_HARDCODED, PER_SAMPLE_BIT_DEPTH_HARDCODED, MONO);
    audio_stream.play();
    let mut chunk_samples = [0_i16; AUDIO_STREAM_RING_BUFFER_SIZE];

    let cursor = Cursor::new(wav_bytes);
    let mut wav = WavReader::new(cursor).unwrap();
    let mut wav_iter = wav.samples::<i16>();

    let mut i_time = 0.0_f32;
    while !render.handle.window_should_close() {
        let delta_time = render.handle.get_frame_time();
        i_time += delta_time;
        render.set_uniform_float(&mut shader, "iTime", i_time);
        if audio_stream.is_processed() {
            for sample in &mut chunk_samples {
                *sample = wav_iter.next().unwrap_or(Ok(0)).unwrap();
            }
            let _ = audio_stream.update(&chunk_samples);
            for (fft_sample, wav_sample) in fft_data.iter_mut().zip(chunk_samples.chunks_exact(2)) {
                let avg = (wav_sample[0] as i32 + wav_sample[1] as i32) / 2_i32;
                *fft_sample = avg as f32 / i16::MAX as f32;
            }
        }
        fft.update_audio_texture(&mut fft_data, &mut fft_image);
        let len = fft_image.get_pixel_data_size();
        let pixels = unsafe { from_raw_parts(fft_image.data as *const u8, len) };
        //println!("FFT image bytes [0..8]: {:?}", &pixels[0..8.min(len)]);
        fft_texture.update_texture(pixels).unwrap();
        render.set_uniform_sampler2d(&mut shader, "iChannel1", &fft_texture);
        pitch_dimension.update_hsv_buffer(i_time);
        let hsv_buffer = pitch_dimension.get_hsv_buffer();
        let mut raylib_vec3_array = [RendererVector3::new(0.0, 0.0, 0.0); HSV_BUFFER_LEN];
        for i in 0..hsv_buffer.len() {
            let [h, s, v] = hsv_buffer[i];
            raylib_vec3_array[i] = RendererVector3::new(h, s, v);
        }
        render.set_uniform_vec3_array(&mut shader, "hsv_buffer", &raylib_vec3_array);
        render.draw_shader_screen(&mut shader, &mut buffer_a);
    }
}
