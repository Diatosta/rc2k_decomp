//! `menu::previous_gearing` (0x469994): the car choice's gearing back (below 0 to 2).
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `screens/gearing_text.rs`: `dword_7554F0`, the current player's `gearing` (+0x30)
//! and its text (string 0x21F + it into `byte_A8BF54`). Its item in the car choice's
//! table (0x74B978). Converted: its Rust version, `screens::gearing_text::step_gearing`,
//! goes through `rc2k_core::screens::captions::step_gearing`. Refuses a current player past
//! four.
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::menu::player::Player;
use crate::screens::gearing_text::{current, regions, step_gearing};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(_: &Regs) -> Result<Left, Refused> {
    // Converted: `rc2k_core::screens::captions::step_gearing` (through `screens::gearing_text`).
    unsafe { step_gearing(false) }?;
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let mut regions = regions().to_vec();
    if let Ok(p) = current() {
        regions.push((p, std::mem::size_of::<Player>()));
    }
    state_entry(r, &rust)
}
