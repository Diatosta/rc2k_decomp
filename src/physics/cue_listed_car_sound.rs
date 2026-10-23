//! `physics::cue_listed_car_sound` (0x45D1BC): eax the current car's entry in
//! `dword_641680`; unless -1, `physics::cue_car_sound` with it.
//!
//! In:  ebx the cue
//! Out: eax the entry; the rest kept; no flags read
//!
//! Caller: `physics::off_line_warnings` (twice).
//! Refuses (before anything) a current car past 7.
//! Shadow mode runs the original alone (`physics/sound_cues.rs`).
//! Self-test: a copy whose call goes to a stand-in, against the Rust version: cars 0 ..
//! 7 listed and not.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, rva};
use crate::physics::sound_cues::{CUE, cue_entry};
use rc2k_core::physics::listed_cars::listed_entry;

fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

/// The steps on `r`.
// Converted: `rc2k_core::physics::listed_cars::listed_entry`.
pub fn cue_listed_car_sound_in(r: &mut Regs) -> Result<(), Refused> {
    let (entry, listed) = listed_entry(g(0x71B3F8), |car| g(0x641680 + 4 * car as usize))?;
    r.eax = entry;
    if listed {
        crate::physics::sound_cues::game_part(CUE, r)?;
    }
    Ok(())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    cue_entry(r, cue_listed_car_sound_in)
}
