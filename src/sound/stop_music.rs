/// Stops the music and waits until the audio thread has acted on it.
pub fn stop_music(o: crate::owners::Owners<'static>) {
    // The logic is `rc2k_core::sound::senders::Music::STOP`.
    crate::sound::senders::music(o, rc2k_core::sound::senders::Music::STOP);
}
