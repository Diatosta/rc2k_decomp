/// Asks the audio thread to play tune 6 (a music command).
pub fn play_tune_6(o: crate::owners::Owners<'static>) {
    crate::sound::senders::play_tune(o, 6);
}
