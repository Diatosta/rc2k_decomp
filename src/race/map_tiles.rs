//! `race::map_tiles` (0x45B663): the map picture's slot and first tile (its corner >> 4)
//! for the tiles' blit (`race::map_tiles_stub`); see `race/map_scroll.rs`.
//!
//! In:  nothing (the work copy `g_car`)
//! Out: eax the slot, ecx / edx the tile, then as the blit leaves them
//!
//! Caller: `race::scroll_map`. Shadow mode runs the original alone (the blit writes a
//! picture). Self-test: a copy whose call goes to a stand-in, against the Rust version with
//! the same: corners, `dword_52A910` each way.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::map_scroll::map_tiles;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let _ = unsafe { map_tiles(r) };
    Ok(())
}
