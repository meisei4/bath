use bath::midi::tests::run_playback;

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
