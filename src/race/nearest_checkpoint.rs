//! `race::nearest_checkpoint` (0x44134F): the checkpoint nearest the work car (x
//! `dword_71A10C`, z `dword_71A114`, put in `dword_639824` / `dword_63982C`), by
//! `race::nearest_checkpoint_in` over them all; its distance in `dword_62400C` and / 65536
//! (`fild` / `fdiv` / `fstp`) in `g_nearestCheckpointDistance` (0x614C68).
//!
//! In:  nothing
//! Out: eax = the checkpoint (0 with none, then nothing else changed); else ecx and edx
//!      the count, ebp the checkpoint, edi its distance, ebx and esi as the last measure
//!      leaves them
//!
//! See `race/start_place.rs`. Callers: `sub_4464AD` (race init's camera start), `sub_43FB00`
//! and `sub_4413F2` (later). Refuses more checkpoints than the lists hold, before anything.
//! Shadow mode: the Rust version, the variables put back, the original's, compared.
//! Self-test: faces, points and checkpoints of the test's (`TestLine`), the car's place
//! set, the game's put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::start_place::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    let g = unsafe { nearest_checkpoint(r) }?;
    Ok(left_changed(r, &g))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
