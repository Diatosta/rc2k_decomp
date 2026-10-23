//! `race::free_game_fonts` (0x41B2BC): `g_gameFontMemory` freed (not counted),
//! `g_gameFontGlyphs` 0; `pusha` / `popa`. One caller, `sub_444940`.
//!
//! In:  nothing
//! Out: every register preserved
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
        let g = unsafe { free(r, &FONTS) }?;
        r.eax = g.eax;
        r.ebx = g.ebx;
        // `memory::free_list` leaves carry clear, and nothing after it sets flags.
        r.set_carry(false);
    }
    Ok(())
}
