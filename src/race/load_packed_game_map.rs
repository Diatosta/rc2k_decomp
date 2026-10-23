//! `race::load_packed_game_map` (0x41B785): the in-race map read from its packed file.
//!
//! In:  eax = the path (`files\maps\mapxx.sq`)
//! Out: every register preserved
//!
//! See `race/game_map.rs`. One caller, `race::load_game_map`. Refuses a count of 0 or
//! past 0xFFFF and a short read not of whole pixels. Shadow mode runs the original alone
//! (it allocates and reads). Self-test: stage 21's map and a missing file, read by both,
//! compared with the pointers taken out; the map freed and the game's put back after
//! each.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::game_map::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { load_packed_game_map(r.eax) }?;
    Ok(())
}
