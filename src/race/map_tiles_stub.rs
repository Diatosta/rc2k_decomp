//! `race::map_tiles_stub` (0x45B68F): `call sub_41B931; retn`: the map's tiles blitted into
//! the map picture (the game's software blit, section 15); see `race/map_scroll.rs`.
//!
//! In:  eax = the picture's slot, ecx / edx = the first tile
//! Out: as `sub_41B931` leaves them (`pusha` .. `popa`: all kept)
//!
//! It is its call: private copy (6, the call relocated), manual patch. Caller:
//! `race::map_tiles`. Shadow mode runs the original alone. Self-test: a copy whose call goes
//! to a stand-in, against the Rust side's one call.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::map_scroll::tiles_part;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let _ = tiles_part(1, r);
    Ok(())
}
