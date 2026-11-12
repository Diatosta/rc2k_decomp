/// Asks the audio thread to play tune 9 (a music command).
pub fn play_tune_9(
    sound: &crate::sound::owner::Sound,
    audio: &crate::audio::owner::Audio,
    system: &crate::system::owner::System,
) {
    crate::sound::senders::play_tune(sound, audio, system, 9);
}
