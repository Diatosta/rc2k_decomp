//! `net::car_list` (0x4AAAF7): the car list the network game's car screen shows: with any
//! of the cars won (`g_secretCars`, ended by -1) in the secret cars' list (`off_75A042`,
//! ended by -1), the list with them (`off_759E52`), else the one without (`off_759DCA`).
//!
//! In:  nothing
//! Out: the list (esi); the original keeps the rest (`pusha` .. `popa`), its flags read by
//!      no caller
//! Each won car is compared with the secret list's first entry before its end is looked
//! for (so the -1 of an empty secret list's end can match a -1 entry, as in the original).
//! Lists without their end in 0x100 entries are refused. Caller: `net::choose_car`.

use crate::errors::Refused;
use crate::menu::lists_store::CarList;
use crate::session::Dword;

/// Whether one of the cars in `won` is in the list `secret` (read on past their ends, as
/// the original reads to their ends' marks).
pub fn any_won(secret: &[Dword], won: &[Dword]) -> Result<bool, Refused> {
    // Converted: `rc2k_core::net::cars::any_won`, the lists' dwords by index.
    // The core asks infallibly: the first refusal is kept and raised after.
    let refused = std::cell::Cell::new(None);
    fn reader<'a>(
        list: &'a [Dword],
        refused: &'a std::cell::Cell<Option<crate::errors::BadInput>>,
    ) -> impl Fn(u32) -> u32 + 'a {
        move |n: u32| {
            crate::session::read_on_in(
                &crate::menu::owner::menus().tables.lists,
                list,
                n as usize,
                "a car list's entry",
            )
            .unwrap_or_else(|e| {
                refused.set(Some(e));
                0
            })
        }
    }
    let answer = rc2k_core::net::cars::any_won(reader(secret, &refused), reader(won, &refused));
    if let Some(e) = refused.take() {
        return Err(e.into());
    }
    Ok(answer?)
}

/// The Rust version.
pub fn car_list() -> Result<CarList, Refused> {
    Ok(
        if any_won(
            &crate::menu::owner::menus().tables.lists.cars_second,
            &crate::menu::owner::menus().tables.lists.secret_cars,
        )? {
            CarList::WithClio
        } else {
            CarList::First
        },
    )
}
