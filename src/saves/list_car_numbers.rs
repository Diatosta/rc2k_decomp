//! `saves::list_car_numbers` (0x470825): two lists of numbers from two lists of pointers,
//! the other way round from `saves::rebuild_car_lists`.
//!
//! In:  nothing
//! Out: every register preserved
//! 0x28 dwords of -1 at `unk_755258` and 0x28 at `unk_7552F8` (the second list has 0x14
//! entries in `var\ral.dat`; the rest of what is filled is whatever lies after it); then
//! for each entry of `off_75A042` up to its -1 the first dword of what it points at goes
//! to the first, and likewise from `off_75A12E` to the second.
//! Callers: `sub_47486C` after the main menu is left, and `sub_486BD3`.
//! Refuses a list of pointers without its -1 within the numbers' room, and a null entry.
//!
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::BadInput;
use crate::framework::{game_read, game_write, rva};

// ---- game data ----

/// How many dwords of each list of numbers are filled.
const FILLED: usize = 0x28;
/// (the numbers, the pointers)
const LISTS: [(usize, usize); 2] = [
    (rva(0x755258), rva(0x75A042)), // unk_755258, off_75A042
    (rva(0x7552F8), rva(0x75A12E)), // unk_7552F8, off_75A12E
];

// ---- Rust version ----

pub unsafe fn list_car_numbers() -> Result<(), BadInput> {
    // Converted: `rc2k_core::saves::cars::numbers`, both lists worked out before either is
    // written.
    unsafe {
        let mut found = Vec::new();
        for (_, pointers) in LISTS {
            let list: Vec<u32> = (0..FILLED)
                .map(|n| game_read::<u32>(pointers + n * 4))
                .collect();
            found.push(rc2k_core::saves::cars::numbers(&list, |car| {
                (car as *const u32).read_unaligned()
            })?);
        }
        for ((numbers, _), list) in LISTS.into_iter().zip(found) {
            for (n, v) in list.into_iter().enumerate() {
                game_write(numbers + n * 4, v);
            }
        }
        Ok(())
    }
}
