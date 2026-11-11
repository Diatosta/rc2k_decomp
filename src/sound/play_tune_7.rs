/// Asks the audio thread to play tune 7 (a music command).
pub fn play_tune_7() {
    crate::sound::senders::play_tune(7);
}
