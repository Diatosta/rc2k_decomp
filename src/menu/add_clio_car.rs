//! `menu::add_clio_car` (0x471C33): puts the Clio into three lists of cars.
//!
//! For the car list without the Clio, the second car list and the car list with the Clio: if the car description
//! is not in the list, it is written over the list's -1. No -1 is written
//! after it: the list then ends wherever the next -1 happens to be (the original's
//! slip, kept; in the image the lists are followed by -1 up to the next one).
//! One caller, the main menu's set-up `menu::set_up_front_end`, when `g_hasClio` is 1 (the Clio
//! update's file is there).
//! Refuses a list without its end within its room.

use crate::errors::BadInput;
use crate::menu::remove_secret_cars::{FIRST, List, read_list};
use rc2k_core::menu::car_lists::place_if_missing;

// ---- game data ----

/// The three lists, in the original's order, with the room each has before the next
/// variable.
const LISTS: [List; 3] = [
    (&crate::menu::owner::MENUS.tables.lists.cars_first, 0x22),
    FIRST,
    (&crate::menu::owner::MENUS.tables.lists.cars_with_clio, 0x3E),
];

pub fn add_clio_car() -> Result<(), BadInput> {
    let clio = crate::menu::cars_store::car_handle(33);
    let mut ends = Vec::new();
    // Converted: `rc2k_core::menu::car_lists::place_if_missing`.
    for list in LISTS {
        if let Some(n) = place_if_missing(&read_list(list)?, clio) {
            ends.push(&list.0[n]);
        }
    }
    for end in ends {
        end.set(clio);
    }
    Ok(())
}
