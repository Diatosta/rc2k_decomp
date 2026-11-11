/// Stops the sound stream and waits until the audio thread has acted on it.
pub fn stop_stream() {
    crate::sound::senders::send_and_wait(rc2k_core::audio::commands::Command::Stream {
        path: 0,
        volume: 0,
        action: 0,
    });
}
