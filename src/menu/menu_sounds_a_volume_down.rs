//! `menu::menu_sounds_a_volume_down` (0x468080): the first menu sounds' volume one less, heard (see `menu/option_steps.rs`).
//!
//! In:  nothing
//! Out: as `option_steps` says (not `pusha`'d)
//! A left/right handler of the sound options page's items.
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::framework::Regs;
use crate::menu::option_steps::{SOUNDS_A_RVA, Then, Volume, volume_entry};

const STEP: Volume = Volume {
    address: 0x468080,
    len: 0x2E,
    entry: [0xFF, 0x0D, 0x9C, 0x56, 0x75, 0x00],
    call: 0x28,
    value_rva: SOUNDS_A_RVA,
    up: false,
    then: Then::Sound(0x10),
};

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    volume_entry(r, STEP)
}
