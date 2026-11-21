//! `menu::remove_secret_cars`: takes the secret cars out of the two lists of cars that can be
//! chosen, and clears the cheats.
//!
//! For the second car list and then the A8 championship car list: the entries that are not in the
//! list of secret cars `lists.secret_cars` are copied to the scratch car list and from there back
//! to the list's start, a -1 put after them, and the cheats' flags set to 0. The lists hold the
//! handles of car descriptions; each ends with -1. Refuses a list without its end within its room,
//! and a list that would be left with no car at all (the count would run through all 32 bits).

use crate::errors::BadInput;
use crate::menu::car_lists_store::{CarEntries, CarLists};
use crate::menu::owner::Menus;
use rc2k_core::menu::car_lists;

/// A list of dwords ended by -1: its cells and how many dwords it has room for.
pub type List<'a> = (&'a dyn CarEntries, usize);

pub fn first(lists: &CarLists) -> List<'_> {
    (&lists.cars_second, 0x28)
}

pub fn second(lists: &CarLists) -> List<'_> {
    (&lists.a8_championship_cars, 0x2D)
}

fn secret(lists: &CarLists) -> List<'_> {
    (&lists.secret_cars, 0x0C)
}

fn scratch(lists: &CarLists) -> List<'_> {
    (&lists.cars_scratch, 0x32)
}

pub(super) fn read_list(list: List) -> Result<Vec<u32>, BadInput> {
    Ok(car_lists::entries(list.1, |n| list.0.entry(n))?)
}

/// The entries and a -1 after them. The caller has made sure of the room.
pub(super) fn write_list(list: List, entries: &[u32]) {
    for (n, &entry) in entries.iter().enumerate() {
        list.0.set_entry(n, entry);
    }
    list.0.set_entry(entries.len(), car_lists::END);
}

pub fn remove_secret_cars(menus: &Menus) -> Result<(), BadInput> {
    let lists = &menus.cars.lists;
    {
        let secret = read_list(secret(lists))?;
        // Each list is read and checked in turn.
        let scratch = scratch(lists);
        let mut kept = Vec::new();
        for list in [first(lists), second(lists)] {
            kept.extend(car_lists::without_secret(
                &[read_list(list)?],
                &secret,
                scratch.1,
            )?);
        }
        for (list, cars) in [first(lists), second(lists)].into_iter().zip(&kept) {
            // The scratch list gets no end mark.
            for (n, &car) in cars.iter().enumerate() {
                scratch.0.set_entry(n, car);
            }
            write_list(list, cars);
            menus.play.cheats.write().cheats = 0u32;
        }
        Ok(())
    }
}
