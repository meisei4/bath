fn main() {
    if let Err(e) = bath::midi::tests::run_playback() {
        eprintln!("Error in playback: {:#}", e);
    }
}
