/// Stops the music and waits until the audio thread has acted on it.
pub fn stop_music(audio: &crate::audio::owner::Audio, system: &crate::system::owner::System) {
    // Converted: `rc2k_core::sound::senders::Music::STOP`.
    crate::sound::senders::music(audio, system, rc2k_core::sound::senders::Music::STOP);
}
