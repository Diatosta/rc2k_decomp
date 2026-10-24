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
use crate::framework::{game_read, game_write, rva};
use crate::menu::screen_loop::Carried;

const CHEATS_RVA: usize = rva(0x75870C); // g_cheats
const OPEN_RVA: usize = rva(0x758800); // byte_758800
const LEVEL_RVA: usize = rva(0x7587EC); // dword_7587EC
const STAGE_RVA: usize = rva(0x7587F0); // dword_7587F0

/// The Rust version for `level` (1 .. 5). `c` the carried values the item gets. (The
/// original leaves eax -1 for a closed level; nothing reads it.)
pub unsafe fn arcade_level(c: &Carried, level: u32) -> Result<(), Refused> {
    let mut r = *c;
    unsafe {
        // Converted: `rc2k_core::menu::arcade::level_open`.
        use rc2k_core::menu::arcade::{CLOSED_SOUND, level_open};
        let open = level_open(game_read(CHEATS_RVA), || {
            // `mov eax, level - 1` for the look-up, kept on the open path.
            r.arg = level - 1;
            game_read::<u8>(OPEN_RVA + level as usize - 1)
        });
        if !open {
            r.arg = CLOSED_SOUND;
            crate::sound::play_menu_sound::play_menu_sound_in_game(r.arg);
            return Ok(());
        }
        game_write(LEVEL_RVA, level);
        game_write(STAGE_RVA, 0u32);
        crate::menu::take_player1_units::take_player1_units()?;
        crate::menu::arcade_event::arcade_event(&r)
    }
}
