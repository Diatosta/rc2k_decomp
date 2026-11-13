//! The race's music at the stage's loading (`race::load_stage`'s "doingamemusic"): with `no_place` set `race::game_music` (tune 6 with the music flag 1,
//! else tune 5), else `race::game_music_8` (tune 8).

use crate::race::owner_shared::RaceShared;

pub fn game_music(shared: &RaceShared, o: crate::owners::Owners<'static>) {
    // Converted: `rc2k_core::race::music::race_tune`.
    let second: u32 = shared.state.results.finished.get();
    crate::sound::senders::play_tune(o, rc2k_core::race::music::race_tune(second));
}

pub fn game_music_8(o: crate::owners::Owners<'static>) {
    crate::sound::play_tune_8::play_tune_8(o);
}
