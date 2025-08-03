use bath::midi::debug::run_playback;

// cargo +nightly build -Zbuild-std --target wasm32-unknown-emscripten --lib --release
// cargo +nightly build -Zbuild-std --target wasm32-unknown-emscripten --lib
// cargo build --lib --release
// cargo build --lib
// cargo build --release
// cargo build

// fluidsynth -a coreaudio -m coremidi ../godot/Resources/audio/dsdnmoy.sf2
// cargo run --example tests --features tests-only
fn main() {
    run_playback().expect("TODO: panic message");
}

// cargo run --example raylib_test_0 --features tests-only
// cargo run --example raylib_test_1 --features tests-only
// cargo run --example ice_sheets --features tests-only
// cargo run --example drekker_effect --features tests-only
// cargo run --example fft_visualizer --features tests-only
// cargo run --example debug_space --features tests-only
// cargo run --example audio_test --features tests-only
// cargo run --example rlgl_test --features tests-only
// cargo run --example feedback_buffer --features tests-only,glsl-330
// cargo run --example ghost_dither_cpu_shader --features tests-only,glsl-100
// cargo run --example ghost_dither_opengl11_geometry --features tests-only,opengl-11
// cargo run --example ghost_dither_opengl11_texture --features tests-only,opengl-11
// cargo run --example ghost_dither_glsl100 --features tests-only,glsl-100
