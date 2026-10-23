//! `net::car_list` (0x4AAAF7): the car list the network game's car screen shows: with any
//! of the cars won (`g_secretCars`, ended by -1) in the secret cars' list (`off_75A042`,
//! ended by -1), the list with them (`off_759E52`), else the one without (`off_759DCA`).
//!
//! In:  nothing
//! Out: esi = the list; the rest kept (`pusha` .. `popa`); the flags of the last compare
//!      (no caller reads them)
//! Each won car is compared with the secret list's first entry before its end is looked
//! for (so the -1 of an empty secret list's end can match a -1 entry, as in the original).
//! Lists without their end in 0x100 entries are refused. Caller: `net::choose_car`.
//! Shadow mode: both run, esi compared. Self-test: a copy whose two lists are the test's,
//! against the Rust version: no car won, one in the list, none of them in it.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::lobby::{Part, a};

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

/// The Rust version on the lists at `secret` and `won`.
pub fn car_list_with(entry: &Regs, secret: u32, won: u32) -> Result<Regs, Refused> {
    let found = any_won(secret, won)?;
    Ok(Regs {
        esi: a(if found { WITH } else { WITHOUT }),
        ..*entry
    })
}

/// The Rust version: the registers it leaves.
pub fn car_list(entry: &Regs, _: Part) -> Result<Regs, Refused> {
    car_list_with(entry, a(SECRET), a(WON))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    crate::race::viewer::regs_entry(r, &|r| car_list(r, &mut crate::net::lobby::game_part))
}
