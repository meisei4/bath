fn main() {
    if let Err(e) = bath::midi::test_playback::run_playback() {
        eprintln!("Error in playback: {:#}", e);
    }
}
