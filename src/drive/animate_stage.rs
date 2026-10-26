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
use crate::framework::{EAX, EBX, ECX, EDX, ESI, Regs, game_addr, rva};
use crate::physics::move_slots::move_slots;
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, restore, state_entry};

/// Everything the two write.
pub fn regions() -> Vec<Region> {
    let mut r = crate::drive::animate_lit_faces::regions();
    r.extend(crate::physics::move_slot::regions());
    r.push((game_addr(rva(0x71BAC8)) as u32, 0x3C8));
    r
}

/// The Rust version: `r` as the original leaves it.
pub unsafe fn animate_stage(r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        let all = regions();
        let saved = capture(&all);
        let result = (|| {
            animate_lit_faces(r)?;
            let left = move_slots(r)?;
            left.apply(r);
            Ok(())
        })();
        if result.is_err() {
            restore(&all, &saved);
        }
        result
    }
}

fn left_of(g: &Regs) -> Left {
    Left::nothing()
        .with(EAX, g.eax)
        .with(EBX, g.ebx)
        .with(ECX, g.ecx)
        .with(EDX, g.edx)
        .with(ESI, g.esi)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let mut g = *r;
    unsafe { animate_stage(&mut g) }?;
    Ok(left_of(&g))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
