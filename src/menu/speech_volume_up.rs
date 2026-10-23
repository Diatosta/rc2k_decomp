//! `menu::speech_volume_up` (0x468055): the speech's volume one more, heard (see `menu/option_steps.rs`).
//!
//! In:  nothing
//! Out: as `option_steps` says (not `pusha`'d)
//! A left/right handler of the sound options page's items.
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::framework::Regs;
use crate::menu::option_steps::{SPEECH_LEVEL_RVA, Then, Volume, volume_entry};

const STEP: Volume = Volume {
    value_rva: SPEECH_LEVEL_RVA,
    up: true,
    then: Then::Speech,
};

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    volume_entry(r, STEP)
}
