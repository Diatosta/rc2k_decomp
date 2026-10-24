//! `saves::rebuild_car_lists` (0x47088C): two lists of pointers from two lists of numbers.
//!
//! In:  nothing
//! Out: every register preserved
//! The list at `off_75A042` (0x28 entries) and the one at `off_75A12E` (0x14) are filled
//! with -1; then, for each number of `unk_755258` up to its -1, the entry of
//! `off_7564A0` with that number goes to the first list, and likewise from `unk_7552F8`
//! to the second. One caller, `saves::load_options`, whose file holds the two lists of
//! numbers. What they are lists of is a reading: cars that can be chosen.
//! Refuses a list of numbers without its -1 within its own length, and a number of
//! 0x400 or more.

use crate::errors::BadInput;
use crate::framework::{game_read, game_write, rva};

// ---- game data ----

const TABLE_RVA: usize = rva(0x7564A0); // off_7564A0
/// (the numbers, the pointers, how many entries each has)
const LISTS: [(usize, usize, usize); 2] = [
    (rva(0x755258), rva(0x75A042), 0x28), // unk_755258, off_75A042
    (rva(0x7552F8), rva(0x75A12E), 0x14), // unk_7552F8, off_75A12E
];

// ---- Rust version ----

pub unsafe fn rebuild_car_lists() -> Result<(), BadInput> {
    // Converted: `rc2k_core::saves::cars::pointers`, both lists worked out before either
    // is written.
    unsafe {
        let mut found = Vec::new();
        for (numbers, _, len) in LISTS {
            let list: Vec<u32> = (0..len)
                .map(|n| game_read::<u32>(numbers + n * 4))
                .collect();
            found.push(rc2k_core::saves::cars::pointers(&list, |id| {
                game_read::<u32>(TABLE_RVA + id as usize * 4)
            })?);
        }
        for ((_, pointers, _), list) in LISTS.into_iter().zip(found) {
            for (n, v) in list.into_iter().enumerate() {
                game_write(pointers + n * 4, v);
            }
        }
        Ok(())
    }
}
