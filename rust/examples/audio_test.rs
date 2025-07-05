use bath::render::raylib::RaylibRenderer;
use bath::render::raylib_util::{EXPERIMENTAL_WINDOW_HEIGHT, EXPERIMENTAL_WINDOW_WIDTH};
use bath::render::renderer::Renderer;
use raylib::audio::RaylibAudio;
use raylib::color::Color;
use raylib::consts::MouseButton::MOUSE_BUTTON_LEFT;
use raylib::drawing::RaylibDraw;
use raylib::ffi::{
    IsAudioStreamProcessed, LoadAudioStream, PlayAudioStream, SetAudioStreamBufferSizeDefault, UpdateAudioStream,
};
use raylib::math::rvec2;
use std::f32::consts::PI;
// const SAMPLE_RATE_DERIVED: u32 = 22_050_u32; //TODO: derive this with some context to the human brain or other phsycis or science or some stuff
// const PER_SAMPLE_BYTE_SIZE_DERIVED: u32 = size_of::<i16>() as u32; // Bytes
// const PER_SAMPLE_BIT_DEPTH_DERIVED: u32 = PER_SAMPLE_BYTE_SIZE_DERIVED * u8::BITS; // Bits
//const PER_SAMPLE_BYTE_SIZE_DERIVED: u32 = size_of::<i32>() as u32;
//const PER_SAMPLE_BIT_DEPTH_DERIVED: u32 = PER_SAMPLE_BYTE_SIZE_DERIVED * u8::BITS;
// const SINGLE_CYCLE_LOOK_UP_TABLE_SAMPLE_COUNT_DERIVED: usize = 512_usize; //TODO: derive this from the above consts please
// const PER_CYCLE_PUSHED_RING_BUFFER_CHUNK_SIZE_DERIVED: usize = 4096_usize; //TODO: derive this from the above stuff please

const CHANNELS: u32 = 1_u32;
//const SAMPLE_RATE_HARDCODED: u32 = 22_050_u32;
const SAMPLE_RATE_HARDCODED: u32 = 44_100_u32;
//const SAMPLE_RATE_HARDCODED: u32 = 48_000_u32;

const PER_SAMPLE_BIT_DEPTH_HARDCODED: u32 = 16_u32;
//const PER_SAMPLE_BIT_DEPTH_HARDCODED: u32 = 32_u32;
const HALF_CYCLE_LOOK_UP_TABLE_SAMPLE_COUNT_HARDCODED: usize = 512_usize; // = 512 BYTES!
                                                                          //const PER_CYCLE_PUSHED_RING_BUFFER_CHUNK_SIZE_HARDCODED: usize = 4096_usize; // = 4096 BYTES
const PER_CYCLE_PUSHED_RING_BUFFER_CHUNK_SIZE_HARDCODED: usize = 1024_usize;

const LUT_ELEMENTS_PER_CYCLE: usize = HALF_CYCLE_LOOK_UP_TABLE_SAMPLE_COUNT_HARDCODED * 2_usize;

pub fn main() {
    let mut render = RaylibRenderer::init(EXPERIMENTAL_WINDOW_WIDTH, EXPERIMENTAL_WINDOW_HEIGHT);
    let raylib_audio = RaylibAudio::init_audio_device().unwrap();
    raylib_audio.set_audio_stream_buffer_size_default(PER_CYCLE_PUSHED_RING_BUFFER_CHUNK_SIZE_HARDCODED as i32);
    unsafe {
        SetAudioStreamBufferSizeDefault(PER_CYCLE_PUSHED_RING_BUFFER_CHUNK_SIZE_HARDCODED as i32);
    }
    //let mut audio_stream = raylib_audio.new_audio_stream(SAMPLE_RATE_HARDCODED, PER_SAMPLE_BIT_DEPTH_HARDCODED, CHANNELS);
    let audio_stream = unsafe { LoadAudioStream(SAMPLE_RATE_HARDCODED, PER_SAMPLE_BIT_DEPTH_HARDCODED, CHANNELS) };
    let mut lut_samples: [i16; LUT_ELEMENTS_PER_CYCLE] = [0; LUT_ELEMENTS_PER_CYCLE];
    let mut chunk_samples: [i16; PER_CYCLE_PUSHED_RING_BUFFER_CHUNK_SIZE_HARDCODED] =
        [0; PER_CYCLE_PUSHED_RING_BUFFER_CHUNK_SIZE_HARDCODED];
    let mut freq_hz: f32 = 500.0;
    let mut prev_freq_hz: f32 = 1.0;
    let mut lut_read_index: usize = 0;
    let mut cycle_length_samples: usize = 1;

    let mut draw_position = rvec2(0, 0);
    let mut mouse_position = rvec2(-100.0, -100.0);
    //audio_stream.play();
    unsafe {
        PlayAudioStream(audio_stream);
    }
    while !render.handle.window_should_close() {
        mouse_position = render.handle.get_mouse_position();
        if render.handle.is_mouse_button_down(MOUSE_BUTTON_LEFT) {
            freq_hz = 40.0 + mouse_position.y;
        }
        if freq_hz != prev_freq_hz {
            let prev_cycle_length = cycle_length_samples;
            let sample_rate_f = SAMPLE_RATE_HARDCODED as f32;
            cycle_length_samples = (sample_rate_f / freq_hz).round() as usize;
            //cycle_length_samples = (sample_rate_f / freq_hz) as usize;
            cycle_length_samples = cycle_length_samples.clamp(1_usize, HALF_CYCLE_LOOK_UP_TABLE_SAMPLE_COUNT_HARDCODED);
            for index in 0_usize..(cycle_length_samples * 2_usize) {
                let index_f = index as f32;
                let cycle_length_samples_f = cycle_length_samples as f32;

                let phase = 2_f32 * PI * index_f / cycle_length_samples_f;
                lut_samples[index] = (phase.sin() * i16::MAX as f32) as i16;
            }
            lut_read_index = lut_read_index * cycle_length_samples / prev_cycle_length;
            prev_freq_hz = freq_hz;
        }
        if unsafe { IsAudioStreamProcessed(audio_stream) } {
            //if audio_stream.is_processed() {
            let mut chunk_write_index = 0_usize;
            while chunk_write_index < PER_CYCLE_PUSHED_RING_BUFFER_CHUNK_SIZE_HARDCODED {
                let mut chunk_len = PER_CYCLE_PUSHED_RING_BUFFER_CHUNK_SIZE_HARDCODED - chunk_write_index;
                let lut_remain = cycle_length_samples - lut_read_index;
                if chunk_len > lut_remain {
                    chunk_len = lut_remain;
                }
                chunk_samples[chunk_write_index..chunk_write_index + chunk_len]
                    .copy_from_slice(&lut_samples[lut_read_index..lut_read_index + chunk_len]);
                lut_read_index = (lut_read_index + chunk_len) % cycle_length_samples;
                chunk_write_index += chunk_len;
            }
            unsafe {
                UpdateAudioStream(
                    audio_stream,
                    chunk_samples.as_ptr() as *const _,
                    PER_CYCLE_PUSHED_RING_BUFFER_CHUNK_SIZE_HARDCODED as i32,
                );
            }
            //TODO: this is borked, and the stuff with primitive type size_of is fucking me up
            //audio_stream.update(&chunk_samples);
            let width = render.handle.get_screen_width();
            let mut d = render.handle.begin_drawing(&render.thread);
            d.clear_background(Color::RAYWHITE);
            d.draw_text(
                &format!("sine frequency: {:.1} Hz", freq_hz),
                width - 220,
                10,
                20,
                Color::RED,
            );
            d.draw_text("click mouse button to change frequency", 10, 10, 20, Color::DARKGRAY);
            for x in 0..width {
                draw_position.x = x as f32;
                let sample_index = x as usize * LUT_ELEMENTS_PER_CYCLE / width as usize;
                let amp = lut_samples[sample_index] as i32;
                draw_position.y = (250 + 50 * amp / i16::MAX as i32) as f32;
                d.draw_pixel_v(draw_position, Color::RED);
            }
        }
    }
}
