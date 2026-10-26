//! `net::reset_cars` (0x463E52): the eight cars' blocks (`g_carBlocks`) made ready for a
//! network race: +0xAA8 the car's number + 1, +0x684 4, +0x28 2, +0x34, +0x30, +0x20,
//! +0x3C 0x80, then `net::clear_car_state`; `dword_6238CC` and `dword_6238D0` 0.
//!
//! In:  nothing
//! Out: ecx = 8; the rest kept; the flags of `cmp ecx, 8` (no caller reads them)
//! Caller: `net::set_up_race`. Its calls through `part`. Shadow mode runs the original
//! alone. Self-test: a copy whose call goes to a stand-in, against the Rust version, on
//! test blocks put in `g_carBlocks`.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::lobby::{Part, g, lobby_entry, p};
use rc2k_core::net::cars;

pub const CAR_BLOCKS: usize = 0x71B3B8; // g_carBlocks: eight pointers
const CLEAR_CAR_STATE: u32 = 0x463EB6; // net::clear_car_state
const AFTER: [usize; 2] = [0x6238CC, 0x6238D0]; // dword_6238CC, dword_6238D0

/// The Rust version: the registers it leaves.
pub fn reset_cars(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    for car in 0..cars::CARS {
        let mut c = r;
        c.ecx = car;
        c.esi = g(CAR_BLOCKS + 4 * car as usize);
        c.eax = car + 1;
        // Converted: `rc2k_core::net::cars::reset_fields`.
        for (offset, v) in cars::reset_fields(car) {
            unsafe { ((c.esi + offset) as *mut u32).write_unaligned(v) };
        }
        part(CLEAR_CAR_STATE, &mut c)?;
        r.eflags = c.eflags;
    }
    r.ecx = 8;
    for x in AFTER {
        p(x, 0);
    }
    Ok(r)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    lobby_entry(r, reset_cars)
}
