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
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, rva};
use crate::menu::parts::{Part, copy, dispatch_parted, run_both, test_part};
use crate::screens::harness::CallOriginal;
use crate::service::harness::{Region, at, capture, restore};

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
        let unlocked = game_read::<u32>(CHEATS_RVA) & 1 != 0;
        if !unlocked {
            // `mov eax, level - 1` for the look-up, kept on the open path.
            r.eax = level - 1;
        }
        if !unlocked && game_read::<u8>(OPEN_RVA + level as usize - 1) != 1 {
            r.eax = 8;
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
        _ => unsafe { call_game(r, game_addr(rva(CALLEES[place as usize]))) },
    }
    Ok(())
}

pub fn dispatch_level(hook: &RawHook, r: &mut Regs, call_original: CallOriginal, level: u32) {
    dispatch_parted(
        hook,
        r,
        call_original,
        &|r, part| unsafe { arcade_level(r, level, part) },
        game_part,
    );
}

pub fn test_level(hook: &RawHook, address: usize, level: u32) -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(hook) else {
        return (0, 0);
    };
    let copy = match copy(
        address,
        0x47,
        &[0x0F, 0xBA, 0x25, 0x0C, 0x87, 0x75, 0x00, 0x00],
        &[0x1D, 0x3C, 0x41],
        &CALLEES,
        &[],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions: [Region; 4] = [
        at(CHEATS_RVA),
        (game_addr(OPEN_RVA) as u32, 8),
        at(LEVEL_RVA),
        at(STAGE_RVA),
    ];
    let saved = capture(&regions);
    for (cheats, open) in [(0u32, 0u8), (0, 1), (1, 0), (0x8000_0001, 2), (2, 2)] {
        run_both(
            &mut t,
            &format!("cheats {cheats:#x}, open {open}"),
            &copy,
            &regions,
            &|| unsafe {
                game_write(CHEATS_RVA, cheats);
                for n in 0..8 {
                    game_write(
                        OPEN_RVA + n,
                        if n == level as usize - 1 { open } else { 0x5A },
                    );
                }
                game_write(LEVEL_RVA, 0xAAAA_AAAAu32);
                game_write(STAGE_RVA, 0xBBBB_BBBBu32);
            },
            Regs::SENTINEL,
            false,
            &|r| unsafe { arcade_level(r, level, &mut test_part) },
        );
    }
    restore(&regions, &saved);
    t.finish()
}
