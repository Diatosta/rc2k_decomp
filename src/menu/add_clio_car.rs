//! `menu::add_clio_car`: puts the Clio into three lists of cars.
//!
//! The lists are the car list without the Clio, the second car list and the car list with the
//! Clio: if the car description is not in a list, it is written over the list's -1. No new -1 is
//! written after it, so the list then ends wherever the next -1 happens to be (a 1999 slip kept
//! as is; the lists are followed by -1 up to the next one). One caller, the main menu's set-up
//! `menu::set_up_front_end`, when the Clio update's file is there. Refuses a list without its
//! end within its room.

use crate::errors::BadInput;
use crate::menu::car_lists_store::car_lists;
use crate::menu::remove_secret_cars::{List, first, read_list};
use rc2k_core::menu::car_lists::place_if_missing;

/// The three lists, with the room each has before the next variable.
pub fn add_clio_car() -> Result<(), BadInput> {
    let lists: [List<'_>; 3] = [
        (&car_lists().cars_first, 0x22),
        first(),
        (&car_lists().cars_with_clio, 0x3E),
    ];
    let clio = crate::menu::cars_store::car_handle(33);
    let mut ends = Vec::new();
    for list in lists {
        if let Some(n) = place_if_missing(&read_list(list)?, clio) {
            ends.push(&list.0[n]);
        }
    }
    for end in ends {
        end.set(clio);
    }
    Ok(())
}
