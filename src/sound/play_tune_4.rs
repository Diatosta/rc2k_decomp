/// Asks the audio thread to play tune 4 (a music command).
pub fn play_tune_4() {
    crate::sound::senders::play_tune(4);
}
