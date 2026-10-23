//! The race's music at the stage's loading (`race::load_stage`'s "doingamemusic", IDA,
//! 2026-10-15): with `dword_64BB18` set `race::game_music` (tune 6 with `dword_75B108` 1,
//! else tune 5), else `race::game_music_8` (tune 8).

use crate::errors::Refused;
use crate::framework::{Regs, game_read, rva};

pub const SECOND_KIND_RVA: usize = rva(0x75B108); // dword_75B108

/// A tune sent: 5, 6 or 8.
pub type Tune<'a> = &'a mut dyn FnMut(u32, &Regs) -> Result<(), Refused>;

pub fn game_music(entry: &Regs, tune: Tune) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::music::race_tune`.
    let second: u32 = unsafe { game_read(SECOND_KIND_RVA) };
    tune(rc2k_core::race::music::race_tune(second), entry)
}

pub fn game_music_8(entry: &Regs, tune: Tune) -> Result<(), Refused> {
    tune(8, entry)
}

/// In the game: the tune's Rust version, sent and waited for.
pub fn game_tune(tune: u32, entry: &Regs) -> Result<(), Refused> {
    use crate::sound::*;
    let sender = match tune {
        5 => &play_tune_5::SENDER,
        6 => &play_tune_6::SENDER,
        _ => &play_tune_8::SENDER,
    };
    senders::send_in_game(sender, entry);
    Ok(())
}
