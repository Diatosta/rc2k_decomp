//! `menu::remove_secret_cars` (0x471BC7): takes the secret cars out of the two lists of
//! cars that can be chosen, and clears the cheats.
//!
//! In:  nothing
//! Out: every register preserved
//! For the list at `off_75A042` and then the one at `off_759D16` (`sub_471BEC`): the
//! entries that are not in the list of secret cars `off_759F4A` (`sub_471BAC`) are copied
//! to the scratch list `unk_759F7A` and from there back to the list's start, a -1 put
//! after them, and the cheats' flags `g_cheats` (0x75870C) set to 0. The lists hold the
//! addresses of car descriptions; each ends with -1.
//! One caller, `system::load_game_data`, when the setting `allsecretcars` is 0.
//! Refuses a list without its end within its room, and a list that would be left with no
//! car at all: the original copies one entry back and then counts through all 32 bits.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::BadInput;
use crate::framework::{game_read, game_write, rva};
use rc2k_core::menu::car_lists;

// ---- game data ----

/// A list of dwords ended by -1: where it is and how many dwords it has room for.
pub type List = (usize, usize);
/// off_75A042: the cars of one kind of event (0x28 entries are kept in `var\ral.dat`).
pub const FIRST: List = (rva(0x75A042), 0x28);
/// off_759D16: the next variable is 0xB4 bytes on.
pub const SECOND: List = (rva(0x759D16), 0x2D);
/// off_759F4A: the secret cars.
const SECRET: List = (rva(0x759F4A), 0x0C);
/// unk_759F7A: where a list is copied to and back from.
const SCRATCH: List = (rva(0x759F7A), 0x32);
pub const CHEATS_RVA: usize = rva(0x75870C); // g_cheats

/// The list's entries before its -1.
pub unsafe fn read_list(list: List) -> Result<Vec<u32>, BadInput> {
    Ok(car_lists::entries(list.1, |n| unsafe {
        game_read::<u32>(list.0 + n * 4)
    })?)
}

/// The entries and a -1 after them. The caller has made sure of the room.
pub unsafe fn write_list(list: List, entries: &[u32]) {
    unsafe {
        for (n, &entry) in entries.iter().enumerate() {
            game_write(list.0 + n * 4, entry);
        }
        game_write(list.0 + entries.len() * 4, car_lists::END);
    }
}

// ---- Rust version ----

pub unsafe fn remove_secret_cars() -> Result<(), BadInput> {
    // Converted: `rc2k_core::menu::car_lists` (`entries`, `without_secret`).
    unsafe {
        let secret = read_list(SECRET)?;
        // Each list read and checked in turn, as the original goes.
        let mut kept = Vec::new();
        for list in [FIRST, SECOND] {
            kept.extend(car_lists::without_secret(
                &[read_list(list)?],
                &secret,
                SCRATCH.1,
            )?);
        }
        for (list, cars) in [FIRST, SECOND].into_iter().zip(&kept) {
            // The scratch list gets no end mark.
            for (n, &car) in cars.iter().enumerate() {
                game_write(SCRATCH.0 + n * 4, car);
            }
            write_list(list, cars);
            game_write(CHEATS_RVA, 0u32);
        }
        Ok(())
    }
}
