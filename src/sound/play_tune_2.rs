/// Asks the audio thread to play tune 2 (a music command).
pub fn play_tune_2() {
    crate::sound::senders::play_tune(2);
}
