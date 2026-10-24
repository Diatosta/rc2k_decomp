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
use crate::net::lobby::a;

const SECRET: usize = 0x75A042; // off_75A042: the secret cars
const WON: usize = 0x759F4A; // g_secretCars: the cars won
const WITH: usize = 0x759E52; // off_759E52
const WITHOUT: usize = 0x759DCA; // off_759DCA

fn get(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

/// Whether one of the cars at `won` is in the list at `secret`.
pub fn any_won(secret: u32, won: u32) -> Result<bool, Refused> {
    // Converted: `rc2k_core::net::cars::any_won`, the lists' dwords by index.
    let at = |list: u32| move |n: u32| get(list.wrapping_add(4 * n));
    Ok(rc2k_core::net::cars::any_won(at(secret), at(won))?)
}

/// The Rust version.
pub fn car_list() -> Result<u32, Refused> {
    Ok(a(if any_won(a(SECRET), a(WON))? {
        WITH
    } else {
        WITHOUT
    }))
}
