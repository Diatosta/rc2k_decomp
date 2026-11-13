/// Asks the audio thread to play tune 7 (a music command).
pub fn play_tune_7(o: crate::owners::Owners<'static>) {
    crate::sound::senders::play_tune(o, 7);
}
