//! Two lists of pointers from two lists of numbers.
//!
//! The car lists (`cars_second`, 0x28 entries, and `shown_cars`, 0x14) are filled with -1; then,
//! for each number of `first_numbers` up to its -1, the entry of the car table (`cars`) with that
//! number goes to the first list, and likewise from `second_numbers` to the second (the options file holds the two lists of numbers). What they are
//! lists of is a reading: cars that can be chosen. Refuses a list of numbers without its -1 within
//! its own length, and a number of 0x400 or more.

use crate::errors::BadInput;
use crate::memory::session::CarEntries;
use crate::menu::owner::Menus;
use crate::system::checked_reads::CheckedReads;

/// (the numbers, the pointers, how many entries each has)
fn lists(menus: &Menus) -> [([u32; 40], &dyn CarEntries, usize); 2] {
    let numbers = menus.cars.numbers.read();
    [
        (numbers.first_numbers, &menus.cars.lists.cars_second, 0x28),
        (numbers.second_numbers, &menus.cars.lists.shown_cars, 0x14),
    ]
}

pub fn rebuild_car_lists(menus: &Menus) -> Result<(), BadInput> {
    let lists = lists(menus);
    // Both lists are worked out before either is written.

    let mut found = Vec::new();
    for (numbers, _, len) in lists {
        let list: Vec<u32> = numbers[..len].to_vec();
        // Saved before the front end first ended: the lists stay as they start.
        if rc2k_core::saves::cars::never_listed(&list) {
            found.push(None);
            continue;
        }
        let reads = CheckedReads::new();
        let answer = rc2k_core::saves::cars::pointers(&list, |id| {
            // Past the table's 34: refused.
            reads.value(crate::menu::cars_store::car_entry(menus, id))
        });
        found.push(Some(reads.finish(answer)??));
    }
    for ((_, pointers, _), list) in lists.into_iter().zip(found) {
        for (n, v) in list.into_iter().flatten().enumerate() {
            pointers.set_entry(n, v);
        }
    }
    Ok(())
}
