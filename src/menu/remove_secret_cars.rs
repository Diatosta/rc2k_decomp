//! `menu::remove_secret_cars`: takes the secret cars out of the two lists of cars that can be
//! chosen, and clears the cheats.
//!
//! For the second car list and then the A8 championship car list: the entries that are not in
//! the list of secret cars `lists.secret_cars` are copied to the scratch car list and from
//! there back to the list's start, a -1 put after them, and the cheats' flags set to 0. The
//! lists hold the handles of car descriptions; each ends with -1.
//! One caller, `system::load_game_data`, when the setting `allsecretcars` is 0.
//! Refuses a list without its end within its room, and a list that would be left with no car
//! at all (the count would run through all 32 bits).

use crate::errors::BadInput;
use crate::menu::owner::Menus;
use crate::session::PlainDword;
use rc2k_core::menu::car_lists;

/// A list of dwords ended by -1: its cells and how many dwords it has room for.
pub type List<'a> = (&'a [PlainDword], usize);

pub fn first(menus: &Menus) -> List<'_> {
    (&menus.tables.lists.cars_second, 0x28)
}

pub fn second(menus: &Menus) -> List<'_> {
    (&menus.tables.lists.a8_championship_cars, 0x2D)
}

fn secret(menus: &Menus) -> List<'_> {
    (&menus.tables.lists.secret_cars, 0x0C)
}

fn scratch(menus: &Menus) -> List<'_> {
    (&menus.tables.lists.cars_scratch, 0x32)
}

pub fn read_list(list: List) -> Result<Vec<u32>, BadInput> {
    Ok(car_lists::entries(list.1, |n| list.0[n].get())?)
}

/// The entries and a -1 after them. The caller has made sure of the room.
pub fn write_list(list: List, entries: &[u32]) {
    for (n, &entry) in entries.iter().enumerate() {
        list.0[n].set(entry);
    }
    list.0[entries.len()].set(car_lists::END);
}

pub fn remove_secret_cars(menus: &Menus) -> Result<(), BadInput> {
    {
        let secret = read_list(secret(menus))?;
        // Each list is read and checked in turn.
        let scratch = scratch(menus);
        let mut kept = Vec::new();
        for list in [first(menus), second(menus)] {
            kept.extend(car_lists::without_secret(
                &[read_list(list)?],
                &secret,
                scratch.1,
            )?);
        }
        for (list, cars) in [first(menus), second(menus)].into_iter().zip(&kept) {
            // The scratch list gets no end mark.
            for (n, &car) in cars.iter().enumerate() {
                scratch.0[n].set(car);
            }
            write_list(list, cars);
            menus.play.cheats.cheats.set(0u32);
        }
        Ok(())
    }
}
