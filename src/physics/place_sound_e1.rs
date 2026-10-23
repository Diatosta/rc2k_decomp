//! `physics::place_sound_e1` (0x45CEF9): when the current car is listed, `physics::sound_band_1` (ebx);
//! a band not 0 goes to its sound object with +0x238 0xE1, +0x23C the band, the place at
//! +0x240 .. and +0x10 1 (see `physics/car_sounds.rs`).
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `sub_45D8FB`, `sub_45DA14`, `sub_45DC3E`, `sub_45EFF3`.
//! Refuses (before anything) a car number past 7, a null sound object when it is
//! written.
//! Self-test: `physics/car_sounds.rs` (`test_sound`): the original and the Rust version on
//! the game's variables (saved and put back) with a test object, exact at double precision.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, rva};
use crate::physics::car_sounds::{cue_plain, g, listed, object, place, placed_entry, put};
use crate::physics::sound_band_1::TWIN;
use crate::physics::sound_bands::{band_value, band_value_plain};
use crate::screens::harness::Left;
use rc2k_core::physics::sound_places::{FIRST, SOUND_E1};

/// The Rust version (the exact one, for `physics::wall_push_back` and
/// `physics::wall_edge_contact`).
pub unsafe fn place_sound_e1() -> Result<Left, Refused> {
    if !listed(g(0x71B3F8))? {
        return Ok(Left::nothing());
    }
    let scratch = g(0x6D1AEC);
    let b = unsafe { band_value(TWIN) };
    if b == 0 {
        return Ok(Left::nothing());
    }
    // The band left `dword_6D1AEC` written: put back if the object refuses.
    let o = match object() {
        Ok(o) => o,
        Err(e) => {
            unsafe { crate::framework::game_write(rva(0x6D1AEC), scratch) };
            return Err(e);
        }
    };
    unsafe {
        put(o + 0x238, 0xE1);
        put(o + 0x23C, b);
        place(o, 0x240, None);
        put(o + 0x10, 1);
    }
    Ok(Left::nothing())
}

/// The converted version (`rc2k_core::physics::sound_places`, plain floats): the hook's.
// Converted: `rc2k_core::physics::sound_places`.
pub unsafe fn place_sound_e1_plain() -> Result<Left, Refused> {
    if !listed(g(0x71B3F8))? {
        return Ok(Left::nothing());
    }
    let scratch = g(0x6D1AEC);
    let b = unsafe { band_value_plain(TWIN) };
    if b == 0 {
        return Ok(Left::nothing());
    }
    // The band left `dword_6D1AEC` written: put back if the object refuses.
    let o = match object() {
        Ok(o) => o,
        Err(e) => {
            unsafe { crate::framework::game_write(rva(0x6D1AEC), scratch) };
            return Err(e);
        }
    };
    unsafe { cue_plain(o, FIRST, SOUND_E1, b, None) };
    Ok(Left::nothing())
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { place_sound_e1_plain() }
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    placed_entry(r, &rust)
}
