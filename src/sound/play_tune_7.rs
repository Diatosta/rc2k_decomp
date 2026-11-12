/// Asks the audio thread to play tune 7 (a music command).
pub fn play_tune_7(
    sound: &crate::sound::owner::Sound,
    audio: &crate::audio::owner::Audio,
    system: &crate::system::owner::System,
) {
    crate::sound::senders::play_tune(sound, audio, system, 7);
}
