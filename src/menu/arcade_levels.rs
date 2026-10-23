//! The arcade start menu's five level items (`menu::arcade_level_1` .. `arcade_level_5`,
//! 0x487CC9, 0x487D10, 0x487D57, 0x487D9E, 0x487DE5): a level not yet open (cheat bit 0 off
//! and `byte_758800[level - 1]` not 1) gets the menu's sound 8 and eax -1; an open one
//! `dword_7587EC` the level, `dword_7587F0` 0, `menu::take_player1_units` and
//! `menu::arcade_event`.
//!
//! In:  nothing
//! Out: eax -1 for a closed level; else the registers as the event leaves them (not
//!      `pusha`'d)
//! Callees at the game's addresses but `take_player1_units` (its Rust version) and the
//! event (its Rust version with its own parts). Ungated thunks (the event); shadow mode
//! runs the original alone. Self-test: a copy whose three calls go to stand-ins, against
//! the Rust version: the cheat on and off, the level open and not.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, game_write, rva};
use crate::menu::parts::Part;

const CALLEES: [usize; 3] = [
    0x40F059, // 0 sound::play_menu_sound
    0x467EA8, // 1 menu::take_player1_units
    0x487E2C, // 2 menu::arcade_event
];
const CHEATS_RVA: usize = rva(0x75870C); // g_cheats
const OPEN_RVA: usize = rva(0x758800); // byte_758800
const LEVEL_RVA: usize = rva(0x7587EC); // dword_7587EC
const STAGE_RVA: usize = rva(0x7587F0); // dword_7587F0

/// The Rust version for `level` (1 .. 5): the registers it leaves.
pub unsafe fn arcade_level(entry: &Regs, level: u32, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    unsafe {
        // Converted: `rc2k_core::menu::arcade::level_open`.
        use rc2k_core::menu::arcade::{CLOSED_SOUND, level_open};
        let open = level_open(game_read(CHEATS_RVA), || {
            // `mov eax, level - 1` for the look-up, kept on the open path.
            r.eax = level - 1;
            game_read::<u8>(OPEN_RVA + level as usize - 1)
        });
        if !open {
            r.eax = CLOSED_SOUND;
            part(0, &mut r)?;
            r.eax = u32::MAX;
            return Ok(r);
        }
        game_write(LEVEL_RVA, level);
        game_write(STAGE_RVA, 0u32);
        part(1, &mut r)?;
        part(2, &mut r)?;
    }
    Ok(r)
}

pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    match place {
        0 => crate::sound::play_menu_sound::play_menu_sound_in_game(r),
        1 => crate::menu::take_player1_units::take_player1_units()?,
        2 => {
            *r = unsafe {
                crate::menu::arcade_event::arcade_event(
                    r,
                    &mut crate::menu::arcade_event::game_part,
                )
            }?
        }
        _ => crate::calls::call(rva(CALLEES[place as usize]), r),
    }
    Ok(())
}

/// `dispatch_level` without its hook (typed tables).
pub fn level_entry(r: &mut Regs, level: u32) -> Result<(), Refused> {
    crate::menu::parts::parted_entry(
        r,
        &|r, part| unsafe { arcade_level(r, level, part) },
        game_part,
    )
}
