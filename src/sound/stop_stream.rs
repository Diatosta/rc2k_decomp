/// Stops the sound stream and waits until the audio thread has acted on it.
pub fn stop_stream(o: crate::owners::Owners<'static>) {
    crate::sound::senders::send_and_wait(
        o,
        rc2k_core::audio::commands::Command::Stream {
            path: 0,
            volume: 0,
            action: rc2k_core::audio::commands::StreamAction::Stop,
        },
    );
}
