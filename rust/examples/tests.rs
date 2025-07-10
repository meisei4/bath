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
// cargo run --example raylib_test_2 --features tests-only
// cargo run --example raylib_test_4 --features tests-only
// cargo run --example raylib_fft_test --features tests-only
// cargo run --example debug_space --features tests-only
// cargo run --example audio_test --features tests-only
// cargo run --example rlgl_test --features tests-only
// cargo run --example feedback_buffer --features tests-only
