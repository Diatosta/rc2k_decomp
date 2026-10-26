//! `drive::countdown_cue` (0x45D1A5): `drive::countdown_sound` for the current car when it
//! has a sound slot (`dword_641680[g_currentCar]` not -1), ebx the sound.
//!
//! In:  ebx the sound
//! Out: eax the slot; the rest kept; no flags read
//!
//! Caller: `drive::countdown_beep`.
//! Refuses a current car past 7. Shadow mode runs the original alone (a sound command).
//! Self-test: none (a sound command).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::physics::flow::g;

/// The Rust version: eax.
// Converted: `rc2k_core::physics::listed_cars::listed_entry`.
pub fn countdown_cue(sound: u32) -> Result<u32, Refused> {
    let (slot, listed) = rc2k_core::physics::listed_cars::listed_entry(g(0x71B3F8), |car| {
        g(0x641680 + 4 * car as usize)
    })?;
    if listed {
        crate::drive::countdown_sound::countdown_sound(slot, sound);
    }
    Ok(slot)
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let eax = countdown_cue(r.ebx)?;
        r.eax = eax
    }
    Ok(())
}
