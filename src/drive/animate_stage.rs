//! `drive::animate_stage` (0x45E9CA): `drive::animate_lit_faces`, then
//! `physics::move_slots`.
//!
//! In:  nothing
//! Out: esi past the slots, ecx 8 (`move_slots`); eax, ebx, edx as `animate_lit_faces`
//!      leaves them; no flags read (callers go on with calls)
//!
//! Callers: race init's loop (`sub_4608BB`, 0x46093E) and `sub_460B2F` (0x460BE2).
//! Refuses what the two refuse; what was written before a refusal is put back first.
//! Self-test: in a race only: a copy of the original (its two calls the game's) against
//! the Rust version on the game's own state, with the lit faces' list as the stage has
//! it; everything written saved and put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::drive::animate_lit_faces::animate_lit_faces;
use crate::errors::Refused;
use crate::physics::move_slots::move_slots;

/// The Rust version. The registers it leaves are read by no caller (liveness: ecx after
/// `drive::replay_step`'s call has no read point).
pub unsafe fn animate_stage() -> Result<(), Refused> {
    unsafe {
        animate_lit_faces()?;
        move_slots()?;
    }
    Ok(())
}
