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

use crate::errors::BadInput;
use crate::menu::car_numbers_store::CAR_NUMBERS;
use crate::menu::lists_store::MENU_LISTS;
use crate::session::Dword;
use crate::system::checked_reads::CheckedReads;

// ---- game data ----

/// How many dwords of each list of numbers are filled.
const FILLED: usize = 0x28;
/// (the numbers, the pointers)
const LISTS: [(&[Dword], &[Dword]); 2] = [
    (&CAR_NUMBERS.first_numbers, &MENU_LISTS.cars_second), // unk_755258, off_75A042
    (&CAR_NUMBERS.second_numbers, &MENU_LISTS.shown_cars), // unk_7552F8, off_75A12E
];

// ---- Rust version ----

pub fn list_car_numbers() -> Result<(), BadInput> {
    // Converted: `rc2k_core::saves::cars::numbers`, both lists worked out before either is
    // written.
    {
        let mut found = Vec::new();
        for (_, pointers) in LISTS {
            let list: Vec<u32> = (0..FILLED).map(|n| pointers[n].get()).collect();
            let reads = CheckedReads::new();
            let answer = rc2k_core::saves::cars::numbers(&list, |car| {
                reads.value(
                    crate::menu::cars_store::description_ptr(car)
                        .and_then(|d| d.read::<u32>("a car's number")),
                )
            });
            found.push(reads.finish(answer)??);
        }
        for ((numbers, _), list) in LISTS.into_iter().zip(found) {
            for (n, v) in list.into_iter().enumerate() {
                numbers[n].set(v);
            }
        }
        Ok(())
    }
}
