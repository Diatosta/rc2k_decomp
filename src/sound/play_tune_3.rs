/// Asks the audio thread to play tune 3 (a music command).
pub fn play_tune_3() {
    crate::sound::senders::play_tune(3);
}
