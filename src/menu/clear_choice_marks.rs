//! `menu::clear_choice_marks` (0x490346): `dword_75AAC8` .. `dword_75AAD4` set to -1.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! Four dwords, one a player by the look of it (their meaning is not read). In two entry
//! lists (0x759406, 0x759412: the car choice's).
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, rva};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

const MARKS_RVA: usize = rva(0x75AAC8); // dword_75AAC8 .. dword_75AAD4

pub fn clear_choice_marks() {
    unsafe { std::ptr::write_bytes(game_addr(MARKS_RVA) as *mut u8, 0xFF, 16) };
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    clear_choice_marks();
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
