//! `menu::init_standings_a` (0x4730D8): the 200 entries of standings table A emptied: +0
//! and +4 -1, +8 .. +0x10 0, +0x14 / +0x18 3600370 (a time beyond any), +0x1C 0.
//!
//! Callers: two of the game's. Refuses a null table, before anything.
use crate::errors::Refused;
use crate::menu::standings::*;

/// The Rust version.
pub fn init_standings_a() -> Result<(), Refused> {
    // Converted: `rc2k_core::saves::rankings::empty_a` on the first `ROOM` entries.
    use rc2k_core::menu::standings::ROOM;
    with_a(|t| {
        t.entries[..ROOM as usize]
            .iter_mut()
            .for_each(rc2k_core::saves::rankings::empty_a);
    })?;
    Ok(())
}
