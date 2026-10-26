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
use crate::menu::lists_store::MENU_LISTS;
use crate::pointers::GamePtr;

/// Whether one of the cars at `won` is in the list at `secret`.
pub fn any_won(secret: u32, won: u32) -> Result<bool, Refused> {
    // Converted: `rc2k_core::net::cars::any_won`, the lists' dwords by index.
    // The core asks infallibly: the first refusal is kept and raised after.
    let refused = std::cell::Cell::new(None);
    let at = |list: u32| {
        let refused = &refused;
        move |n: u32| {
            GamePtr::new(list.wrapping_add(4 * n))
                .read::<u32>("a car list's entry")
                .unwrap_or_else(|e| {
                    refused.set(Some(e));
                    0
                })
        }
    };
    let answer = rc2k_core::net::cars::any_won(at(secret), at(won));
    if let Some(e) = refused.take() {
        return Err(e.into());
    }
    Ok(answer?)
}

/// The Rust version.
pub fn car_list() -> Result<u32, Refused> {
    let secret = MENU_LISTS.cars_second[0].addr();
    let won = MENU_LISTS.secret_cars[0].addr();
    Ok(if any_won(secret, won)? {
        MENU_LISTS.cars_with_clio[0].addr()
    } else {
        MENU_LISTS.cars_first[0].addr()
    })
}
