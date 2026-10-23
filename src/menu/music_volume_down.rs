//! `menu::music_volume_down` (0x467F48): the music's volume one less (see `menu/option_steps.rs`).
//!
//! In:  nothing
//! Out: as `option_steps` says (not `pusha`'d)
//! A left/right handler of the sound options page's items.
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::framework::Regs;
use crate::menu::option_steps::{MUSIC_LEVEL_RVA, Then, Volume, volume_entry};

const STEP: Volume = Volume {
    value_rva: MUSIC_LEVEL_RVA,
    up: false,
    then: Then::Music,
};

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    volume_entry(r, STEP)
}
