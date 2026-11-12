/// Stops the sound stream and waits until the audio thread has acted on it.
pub fn stop_stream(audio: &crate::audio::owner::Audio, system: &crate::system::owner::System) {
    crate::sound::senders::send_and_wait(
        audio,
        system,
        rc2k_core::audio::commands::Command::Stream {
            path: 0,
            volume: 0,
            action: 0,
        },
    );
}
