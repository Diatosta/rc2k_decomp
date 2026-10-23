//! `install::backdrop_frame` (0x415273, `loc_415273`, inside `sub_41525D`): the frame of
//! `install::show_backdrop`: the surface locked (`gfx::lock_surface`; failing: carry set,
//! nothing more), the backdrop picture copied to it, unlocked and shown.
//!
//! In:  nothing
//! Out: as `gfx::show_frame` leaves them, or the lock's carry
//! Caller `screens::run_frame` (eax this address). Its calls through `part` (the
//! platform's and `gfx::copy_picture_to_surface`). A call at +0: private copy, manual
//! patch. Shadow mode runs the original alone. Self-test: a copy whose four calls go to
//! stand-ins (the lock failing or not), against the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::install::installer::{COPY_PICTURE, LOCK, Part, SHOW, UNLOCK, at};
use crate::race::viewer::regs_entry;

/// The surface locked, the backdrop drawn, the frame shown; the registers as the calls
/// leave them.
pub fn backdrop_frame(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    part(at(LOCK), &mut r)?;
    if r.carry() {
        return Ok(r);
    }
    for callee in [COPY_PICTURE, UNLOCK, SHOW] {
        part(at(callee), &mut r)?;
    }
    Ok(r)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &|r| {
        backdrop_frame(r, &mut crate::install::installer::game_part)
    })
}
