/// Asks the audio thread to play tune 9 (a music command).
pub fn play_tune_9() {
    crate::sound::senders::play_tune(9);
}
