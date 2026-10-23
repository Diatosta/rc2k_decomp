//! `menu::add_clio_car` (0x471C33): puts the Clio into three lists of cars.
//!
//! In:  nothing
//! Out: every register preserved
//! For the lists at `off_759DCA`, `off_75A042` and `off_759E52`: if the car description
//! `unk_7574E0` is not in the list, it is written over the list's -1. No -1 is written
//! after it: the list then ends wherever the next -1 happens to be (the original's
//! slip, kept; in the image the lists are followed by -1 up to the next one).
//! One caller, the main menu's set-up `sub_47C63F`, when `g_hasClio` is 1 (the Clio
//! update's file is there).
//! Refuses a list without its end within its room.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::BadInput;
use crate::framework::{game_addr, game_write, rva};
use crate::menu::remove_secret_cars::{FIRST, List, read_list};
use rc2k_core::menu::car_lists::place_if_missing;

// ---- game data ----

const CLIO_RVA: usize = rva(0x7574E0); // unk_7574E0
/// The three lists, in the original's order, with the room each has before the next
/// variable.
const LISTS: [List; 3] = [
    (rva(0x759DCA), 0x22), // off_759DCA
    FIRST,                 // off_75A042
    (rva(0x759E52), 0x3E), // off_759E52
];

// ---- Rust version ----

pub unsafe fn add_clio_car() -> Result<(), BadInput> {
    unsafe {
        let clio = game_addr(CLIO_RVA) as u32;
        let mut ends = Vec::new();
        // Converted: `rc2k_core::menu::car_lists::place_if_missing`.
        for list in LISTS {
            if let Some(n) = place_if_missing(&read_list(list)?, clio) {
                ends.push(list.0 + n * 4);
            }
        }
        for end in ends {
            game_write(end, clio);
        }
        Ok(())
    }
}
