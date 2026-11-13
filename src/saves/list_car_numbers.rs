//! Two lists of numbers from two lists of pointers, the other way round from
//! `saves::rebuild_car_lists`.
//!
//! 0x28 dwords of -1 in `first_numbers` and 0x28 in `second_numbers` (the second list has 0x14
//! entries in `var\ral.dat`; the rest of what is filled is whatever lies after it); then for
//! each entry of `cars_second` up to its -1 the first dword of what it points at goes to the
//! first, and likewise from `shown_cars` to the second.
//! Called by `menu::run_front_end` after the main menu is left, and `menu::add_shown_car`.
//! Refuses a list of pointers without its -1 within the numbers' room, and a null entry.

use crate::errors::BadInput;
use crate::menu::owner::Menus;
use crate::session::PlainDword;
use crate::system::checked_reads::CheckedReads;

/// How many dwords of each list of numbers are filled.
const FILLED: usize = 0x28;
/// (the numbers, the pointers)
fn lists(menus: &Menus) -> [(&[PlainDword], &[PlainDword]); 2] {
    [
        (
            &menus.cars.numbers.first_numbers,
            &menus.tables.lists.cars_second,
        ),
        (
            &menus.cars.numbers.second_numbers,
            &menus.tables.lists.shown_cars,
        ),
    ]
}

pub fn list_car_numbers_in(menus: &Menus) -> Result<(), BadInput> {
    let lists = lists(menus);
    // Both lists are worked out before either is written.
    {
        let mut found = Vec::new();
        for (_, pointers) in lists {
            let list: Vec<u32> = (0..FILLED).map(|n| pointers[n].get()).collect();
            let reads = CheckedReads::new();
            let answer = rc2k_core::saves::cars::numbers(&list, |car| {
                reads.value(
                    crate::menu::cars_store::description(&menus.cars.table, car)
                        .map(|d| d.get().car),
                )
            });
            found.push(reads.finish(answer)??);
        }
        for ((numbers, _), list) in lists.into_iter().zip(found) {
            for (n, v) in list.into_iter().enumerate() {
                numbers[n].set(v);
            }
        }
        Ok(())
    }
}
