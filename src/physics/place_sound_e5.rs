//! `physics::place_sound_e5` (0x45CF95): when the current car is listed, `physics::sound_band_4` (ebx)
//! goes to its sound object with +0x24C 0xE5, +0x250 the band, the place at +0x254 .. and
//! +0x10 1 (see `physics/car_sounds.rs`).
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `sub_45EDB5`.
//! Refuses (before anything) a car number past 7, a null sound object when it is
//! written.
//! Self-test: `physics/car_sounds.rs` (`test_sound`): the original and the Rust version on
//! the game's variables (saved and put back) with a test object, exact at double precision.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::physics::car_sounds::{cue_plain, g, listed, object, placed_entry};
use crate::physics::sound_band_4::TWIN;
use crate::physics::sound_bands::band_value_plain;
use crate::screens::harness::Left;
use rc2k_core::physics::sound_places::{SECOND, SOUND_E5};

/// The Rust version.
// Converted: `rc2k_core::physics::sound_places`.
pub unsafe fn place_sound_e5() -> Result<Left, Refused> {
    if !listed(g(0x71B3F8))? {
        return Ok(Left::nothing());
    }
    let o = object()?;
    let b = unsafe { band_value_plain(TWIN) };
    unsafe { cue_plain(o, SECOND, SOUND_E5, b, None) };
    Ok(Left::nothing())
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { place_sound_e5() }
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    placed_entry(r, &rust)
}
