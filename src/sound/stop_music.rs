/// Stops the music and waits until the audio thread has acted on it.
pub fn stop_music() {
    // Converted: `rc2k_core::sound::senders::Music::STOP`.
    crate::sound::senders::music(rc2k_core::sound::senders::Music::STOP);
}
