/// Asks the audio thread to play tune 8 (a music command).
pub fn play_tune_8(o: crate::owners::Owners<'static>) {
    crate::sound::senders::play_tune(o, 8);
}
