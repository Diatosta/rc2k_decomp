//! `race::free_object_points` (0x41F89B): `stru_5C6DD0` freed, its total off
//! `dword_5C6F90`, `g_objectPoints` and `g_faceMemory` 0. One caller, `sub_44346B`.
//!
//! In:  nothing
//! Out: eax = the list's total, ebx = the list; the rest preserved
//!
//! See `race/stage_free.rs`. Shadow mode runs the original alone (it frees). Refuses,
//! before anything, a list with a block while the game's own heap is in use. Self-test:
//! with the list empty in the game, blocks of the test's (none, one, three) freed by each.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::stage_free::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let g = unsafe { free(r, &OBJECT_POINTS) }?;
        r.eax = g.eax;
        r.ebx = g.ebx;
        // `memory::free_list` leaves carry clear, and nothing after it sets flags.
        r.set_carry(false);
    }
    Ok(())
}
