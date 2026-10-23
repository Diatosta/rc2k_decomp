//! `menu::next_gearing` (0x469948): the car choice's gearing on (past 2 to 0).
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
use crate::screens::gearing_text::step_gearing;

/// The Rust version the typed tables run.
pub fn run() -> Result<(), Refused> {
    // Converted: `rc2k_core::screens::captions::step_gearing` (through `screens::gearing_text`).
    unsafe { step_gearing(true) }
}
