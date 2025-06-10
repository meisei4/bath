use bath::midi::tests::run_playback;

// fluidsynth -a coreaudio -m coremidi ../godot/Resources/audio/dsdnm.sf2
// cargo run --example tests --features tests-only
fn main() {
    run_playback().expect("TODO: panic message");
}
