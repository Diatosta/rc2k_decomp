//! `race::render3d_tidy` (0x4433A7): `race::free_point_records` (a `call` and a `retn`;
//! "render3d_tidy"). Eight callers (`sub_444940`, `sub_485570` and menus).
//!
//! In:  nothing
//! Out: eax = the list's total, ebx = the list; the rest preserved
//!
//! See `race/stage_free.rs`. It begins with its call: private copy, the call relocated,
//! manual patch. Shadow mode runs the original alone (it frees). Self-test: as the freeing's
//! (blocks of the test's in the list when it is empty, freed by each).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::stage_free::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let g = unsafe { free(r, &POINT_RECORDS) }?;
        r.eax = g.eax;
        r.ebx = g.ebx;
        // `memory::free_list` leaves carry clear, and nothing after it sets flags.
        r.set_carry(false);
    }
    Ok(())
}
