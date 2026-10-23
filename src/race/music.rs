//! The race's music at the stage's loading (`race::load_stage`'s "doingamemusic", IDA,
//! 2026-10-15): with `dword_64BB18` set `race::game_music` (tune 6 with `dword_75B108` 1,
//! else tune 5), else `race::game_music_8` (tune 8).

use crate::framework::{game_read, rva};

pub const SECOND_KIND_RVA: usize = rva(0x75B108); // dword_75B108

pub fn game_music() {
    // Converted: `rc2k_core::race::music::race_tune`.
    let second: u32 = unsafe { game_read(SECOND_KIND_RVA) };
    crate::sound::senders::play_tune(rc2k_core::race::music::race_tune(second));
}

pub fn game_music_8() {
    crate::sound::play_tune_8::play_tune_8();
}
