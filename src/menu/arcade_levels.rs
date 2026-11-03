//! The arcade start menu's five level items (`menu::arcade_level_1` .. `arcade_level_5`,
//! 0x487CC9, 0x487D10, 0x487D57, 0x487D9E, 0x487DE5): a level not yet open (cheat bit 0 off
//! and `byte_758800[level - 1]` not 1) gets the menu's sound 8 and eax -1; an open one
//! `dword_7587EC` the level, `dword_7587F0` 0, `menu::take_player1_units` and
//! `menu::arcade_event`.
//!
//! In:  nothing
//! Out: eax -1 for a closed level; else the registers as the event leaves them (not
//! `pusha`'d) Callees at the game's addresses but `take_player1_units` (its Rust version) and the
//! event (its Rust version with its own parts).

use crate::errors::Refused;
use crate::menu::arcade_store::ARCADE_GAME;
use crate::menu::cheats_store::CHEAT_VARS;

/// The Rust version for `level` (1 .. 5).
pub fn arcade_level(level: u32) -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::arcade::level_open`.
    use rc2k_core::menu::arcade::{CLOSED_SOUND, level_open};
    let open = level_open(CHEAT_VARS.cheats.get(), || {
        ARCADE_GAME.levels_won.get(level as usize - 1)
    });
    if !open {
        crate::sound::play_menu_sound::play_menu_sound_in_game(CLOSED_SOUND);
        return Ok(());
    }
    ARCADE_GAME.level.set(level);
    ARCADE_GAME.stage.set(0u32);
    crate::menu::take_player1_units::take_player1_units()?;
    crate::menu::arcade_event::arcade_event()
}
