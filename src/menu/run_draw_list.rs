//! `menu::run_draw_list` (0x4681ED): `run_list` on the current screen's draw list, the one
//! `dword_7554F8` points at.
//!
//! In:  nothing
//! Out: every register preserved
//! Three callers, the menu's frame step among them. Refuses a list without an end in its first
//! 256 entries.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::BadInput;
use crate::framework::{Regs, game_read, rva};
use crate::menu::run_list::{Call, run_list};

// ---- game data ----

pub const DRAW_LIST_RVA: usize = rva(0x7554F8); // dword_7554F8

// ---- Rust version ----

pub unsafe fn run_draw_list(entry_regs: &Regs, call: Call) -> Result<(), BadInput> {
    unsafe { run_list(game_read(DRAW_LIST_RVA), entry_regs, call) }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let entry_regs = *r;
    unsafe { run_draw_list(&entry_regs, &mut crate::menu::table_calls::call) }?;
    Ok(())
}
