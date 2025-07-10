use asset_payload::payloads::{DEBUG_VERT_PATH, FFT_FRAG_PATH, MIDI_FILE_PATH, SOUND_FONT_FILE_PATH};
use asset_payload::LocalCachePaths;
use bath::midi::pitch::PitchDimension;
use bath::render::raylib::RaylibRenderer;
use bath::render::raylib_util::{EXPERIMENTAL_WINDOW_HEIGHT, EXPERIMENTAL_WINDOW_WIDTH};
use bath::render::{renderer::Renderer, renderer::RendererVector2};
use bath::sound_render::raylib::RaylibFFTTexture;
use bath::sound_render::sound_renderer::{
    FFTTexture, AUDIO_STREAM_RING_BUFFER_SIZE, BUFFER_SIZE, CHANNELS, FFT_HISTORICAL_SMOOTHING_BUFFER_TIME_SECONDS,
    FFT_WINDOW_SIZE, PER_SAMPLE_BIT_DEPTH_HARDCODED, RING_BUFFER_PADDING, SAMPLE_RATE_HARDCODED, WINDOW_TIME,
};
use fftw2_sys::fftw_complex;
use hound::SampleFormat::Int;
use hound::WavReader;
use raylib::core::audio::RaylibAudio;
use raylib::ffi::{
    IsAudioStreamProcessed, LoadAudioStream, PlayAudioStream, SetAudioStreamBufferSizeDefault, UpdateAudioStream,
};
use raylib::texture::RaylibTexture2D;
use std::fs;
use std::io::Cursor;
use std::slice::from_raw_parts;
use std::time::SystemTime;

fn main() {
    let mut render = RaylibRenderer::init(EXPERIMENTAL_WINDOW_WIDTH, EXPERIMENTAL_WINDOW_HEIGHT);
    let i_resolution = RendererVector2::new(
        render.handle.get_screen_width() as f32,
        render.handle.get_screen_height() as f32,
    );
    let mut buffer_a = render.init_render_target(i_resolution, true);
    let mut shader = render.load_shader_full(DEBUG_VERT_PATH(), FFT_FRAG_PATH());
    render.set_uniform_vec2(&mut shader, "iResolution", i_resolution);
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
    render.set_uniform_sampler2d(&mut shader, "iChannel0", &fft_texture);

    let raylib_audio = RaylibAudio::init_audio_device().unwrap();
    raylib_audio.set_audio_stream_buffer_size_default(AUDIO_STREAM_RING_BUFFER_SIZE as i32);
    unsafe {
        SetAudioStreamBufferSizeDefault(AUDIO_STREAM_RING_BUFFER_SIZE as i32);
    }
    let audio_stream = unsafe { LoadAudioStream(SAMPLE_RATE_HARDCODED, PER_SAMPLE_BIT_DEPTH_HARDCODED, CHANNELS) };
    let mut chunk_samples: [i16; AUDIO_STREAM_RING_BUFFER_SIZE] = [0; AUDIO_STREAM_RING_BUFFER_SIZE];

    let mut pitch = PitchDimension::default();
    pitch.resolve_payload_to_midi_buffer(MIDI_FILE_PATH());
    let wav_bytes = pitch.resolve_payload_to_pcm_buffer_cache(
        SAMPLE_RATE_HARDCODED as i32,
        MIDI_FILE_PATH(),
        SOUND_FONT_FILE_PATH(),
        LocalCachePaths::CACHED_WAV_PATH,
    );

    let cursor = Cursor::new(wav_bytes);
    let mut wav = WavReader::new(cursor).unwrap();

    //TODO: WTF just happened in this: ffmpeg -i "shadertoy_music_experiment_min_bitrate.ogg" -ac 1 -sample_fmt s16 -c:a pcm_s16le shadertoy.wav
    // let mut wav = WavReader::open(CACHED_WAV_PATH).unwrap();
    let wav_spec = wav.spec();
    let mut wav_iter = wav.samples::<i16>();
    let is_stereo = wav_spec.channels == 2_u16;
    assert!(
        wav_spec.sample_format == Int && wav_spec.bits_per_sample == 16_u16,
        "WAV must be 16-bit signed PCM"
    );
    print!(
        "fmt: {:?}, bits per sample: {}",
        wav_spec.sample_format, wav_spec.bits_per_sample
    );
    unsafe {
        PlayAudioStream(audio_stream);
    }
    let mut vert_mod_time = get_file_mod_time(LocalCachePaths::DEBUG_VERT_PATH);
    let mut frag_mod_time = get_file_mod_time(LocalCachePaths::FFT_FRAG_PATH);
    let mut song_time = 0.0f32;

    while !render.handle.window_should_close() {
        let delta_time = render.handle.get_frame_time();
        song_time += delta_time;
        let _ = pitch.update_hsv_buffer(song_time);

        if unsafe { IsAudioStreamProcessed(audio_stream) } {
            for sample in &mut chunk_samples {
                //downMIX
                if is_stereo {
                    let left = wav_iter.next().unwrap_or(Ok(0)).unwrap();
                    let right = wav_iter.next().unwrap_or(Ok(0)).unwrap();
                    *sample = ((left as i32 + right as i32) / 2_i32) as i16;
                } else {
                    *sample = wav_iter.next().unwrap_or(Ok(0)).unwrap();
                }
            }
            unsafe {
                UpdateAudioStream(
                    audio_stream,
                    chunk_samples.as_ptr() as *const _,
                    AUDIO_STREAM_RING_BUFFER_SIZE as i32,
                );
            }
            //downSAMPLE
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
        let new_vert_mod_time = get_file_mod_time(LocalCachePaths::DEBUG_VERT_PATH);
        let new_frag_mod_time = get_file_mod_time(LocalCachePaths::FFT_FRAG_PATH);
        if new_vert_mod_time != vert_mod_time || new_frag_mod_time != frag_mod_time {
            println!("Shader modified, reloading...");
            let vert_src = fs::read_to_string(LocalCachePaths::DEBUG_VERT_PATH).unwrap();
            let frag_src = fs::read_to_string(LocalCachePaths::FFT_FRAG_PATH).unwrap();
            let hot_vert_leaked = Box::leak(vert_src.into_boxed_str());
            let hot_frag_leaked = Box::leak(frag_src.into_boxed_str());
            shader = render.load_shader_full(hot_vert_leaked, hot_frag_leaked);
            render.set_uniform_vec2(&mut shader, "iResolution", i_resolution);
            render.set_uniform_sampler2d(&mut shader, "iChannel0", &fft_texture);
            vert_mod_time = new_vert_mod_time;
            frag_mod_time = new_frag_mod_time;
        }
        render.draw_shader_screen_tilted_geom(&mut shader, &mut buffer_a, 0_f32);
        //render.draw_shader_screen(&mut shader, &mut buffer_a);
    }
}

fn get_file_mod_time(path: &str) -> SystemTime {
    fs::metadata(path)
        .and_then(|m| m.modified())
        .unwrap_or(SystemTime::UNIX_EPOCH)
}
