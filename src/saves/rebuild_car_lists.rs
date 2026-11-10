//! `saves::rebuild_car_lists` (0x47088C): two lists of pointers from two lists of numbers.
//!
//! The list at `off_75A042` (0x28 entries) and the one at `off_75A12E` (0x14) are filled
//! with -1; then, for each number of `unk_755258` up to its -1, the entry of
//! `off_7564A0` with that number goes to the first list, and likewise from `unk_7552F8`
//! to the second. One caller, `saves::load_options`, whose file holds the two lists of
//! numbers. What they are lists of is a reading: cars that can be chosen.
//! Refuses a list of numbers without its -1 within its own length, and a number of
//! 0x400 or more.

use crate::errors::BadInput;
use crate::menu::owner::Menus;
use crate::session::Dword;
use crate::system::checked_reads::CheckedReads;

// ---- game data ----

/// (the numbers, the pointers, how many entries each has)
const LISTS: [(&[Dword], &[Dword], usize); 2] = [
    (
        &crate::menu::owner::MENUS.cars.numbers.first_numbers,
        &crate::menu::owner::MENUS.tables.lists.cars_second,
        0x28,
    ), // unk_755258, off_75A042
    (
        &crate::menu::owner::MENUS.cars.numbers.second_numbers,
        &crate::menu::owner::MENUS.tables.lists.shown_cars,
        0x14,
    ), // unk_7552F8, off_75A12E
];

pub fn rebuild_car_lists(menus: &Menus) -> Result<(), BadInput> {
    // Converted: `rc2k_core::saves::cars::pointers`, both lists worked out before either
    // is written.

    let mut found = Vec::new();
    for (numbers, _, len) in LISTS {
        let list: Vec<u32> = (0..len).map(|n| numbers[n].get()).collect();
        // Saved before the front end first ended: the lists stay as they start.
        if rc2k_core::saves::cars::never_listed(&list) {
            found.push(None);
            continue;
        }
        let reads = CheckedReads::new();
        let answer = rc2k_core::saves::cars::pointers(&list, |id| {
            // Past the table's 34 the original reads on into the bytes after it: refused.
            reads.value(crate::menu::cars_store::car_entry(menus, id))
        });
        found.push(Some(reads.finish(answer)??));
    }
    for ((_, pointers, _), list) in LISTS.into_iter().zip(found) {
        for (n, v) in list.into_iter().flatten().enumerate() {
            pointers[n].set(v);
        }
    }
    Ok(())
}
