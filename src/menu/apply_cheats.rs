//! `menu::apply_cheats` (0x480AC1): acts on the cheats that are switched on.
//!
//! In:  nothing
//! Out: every register preserved
//! The cheats are the bits of `g_cheats` (0x75870C); each that is set writes "Cheat n
//! (...)" to the game's log (`CHEATS` below, in this order):
//! - bits 0, 1, 13 .. 16 stay set and do nothing more here (unlock arcade, unlock cars, no
//!   damage, damage timer, 16, arcade timer): other code looks at them;
//! - bits 2 .. 9, 11 and 12 are cleared and each adds one secret car to the list of cars
//!   at `off_75A042`, unless it is there already (the ten functions `sub_470FE9` ..
//!   `sub_471332`, plain functions here). Bit 6, "Max Power", also adds its car to the
//!   list at `off_759D16`, but only when it was not in the first. The other nine have
//!   the same code for the second list, which is never reached;
//! - bit 10 is cleared and does nothing;
//! - bits 17 and 18 stay set and store 1 in `dword_75ACCC` (the A8 game; the reversed
//!   tracks).
//!
//! Callers: `system::load_game_data` with the bits 0x1FBC (when the setting
//! `allsecretcars` is 1) and `sub_480CBF`, which finds a cheat's name typed as the fourth
//! player's name.
//! Shadow mode: the original first, then the Rust version without the log lines on the
//! variables put back, compared. Self-test: a redirected copy whose log calls note the
//! text, on lists of the test's in the game's places.
//! Refuses a list without its end, or without room for a car more.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::menu::remove_secret_cars::{CHEATS_RVA, FIRST, List, SECOND, read_list, write_list};
use crate::menu::text_entry::log_l1;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;
use rc2k_core::menu::car_lists;

// ---- game data ----

const A8_RVA: usize = rva(0x75ACCC); // dword_75ACCC

/// By bit (what each does: `rc2k_core::menu::car_lists::CHEATS`): the log line's text and
/// the cheat's car, if any.
const CHEATS: [(usize, usize); 19] = [
    (0x4DE7A9, 0),        // "Cheat 1 (Unlock Arcade)"
    (0x4DE7C1, 0),        // "Cheat 2 (Unlock Cars)"
    (0x4DE7D7, 0x757210), // "Cheat 3 (Spud)"
    (0x4DE7E6, 0x757300), // "Cheat 4 (Sheep)"
    (0x4DE7F6, 0x757288), // "Cheat 5 (Cow)"
    (0x4DE804, 0x757030), // "Cheat 6 (MF Sports)"
    (0x4DE818, 0x756838), // "Cheat 7 (Max Power)"
    (0x4DE82C, 0x757468), // "Cheat 8 (Local Shop)"
    (0x4DE841, 0x757120), // "Cheat 9 (Bolt's Pug)"
    (0x4DE856, 0x757198), // "Cheat 10 (Skip)"
    (0x4DE866, 0),        // "Cheat 11"
    (0x4DE86F, 0x7570A8), // "Cheat 12 (MF Hatchback)"
    (0x4DE887, 0x7573F0), // "Cheat 13 (Sheep Wagon)"
    (0x4DE89E, 0),        // "Cheat 14 (No Damage)"
    (0x4DE8B3, 0),        // "Cheat 15 (Damage Timer)"
    (0x4DE8CB, 0),        // "Cheat 16"
    (0x4DE8D4, 0),        // "Cheat 17 (Arcade Timer)"
    (0x4DE8EC, 0),        // "Cheat 18 (Enable A8 Game)"
    (0x4DE906, 0),        // "Cheat 19 (Reverse Sim Tracks)"
];

// ---- Rust version ----

/// `log` gets the address of each line's text.
pub unsafe fn apply_cheats(log: &mut dyn FnMut(*const u8)) -> Result<(), BadInput> {
    // Converted: `rc2k_core::menu::car_lists::apply_cheats`.
    unsafe {
        let cheats: u32 = game_read(CHEATS_RVA);
        let list = |l: List| -> Result<car_lists::List, BadInput> {
            Ok(car_lists::List {
                cars: read_list(l)?,
                room: l.1,
                changed: false,
            })
        };
        let (mut first, mut second) = (list(FIRST)?, list(SECOND)?);
        let a = car_lists::apply_cheats(cheats, &mut first, &mut second, |bit| {
            game_addr(rva(CHEATS[bit].1)) as u32
        })?;
        // Nothing has been written or logged up to here.
        game_write(CHEATS_RVA, a.cheats);
        if first.changed {
            write_list(FIRST, &first.cars);
        }
        if second.changed {
            write_list(SECOND, &second.cars);
        }
        if a.stores {
            game_write(A8_RVA, 1u32);
        }
        for bit in a.logged {
            log(game_addr(rva(CHEATS[bit].0)) as *const u8);
        }
        Ok(())
    }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { apply_cheats(&mut |line| log_l1(line, true)) }?;
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

// logfile::log_write_l1_nl
