//! `menu::flip_yes_no` (0x46FA52): the "leave?" question's button turned to the other
//! answer.
//!
//! In:  nothing
//! Out: every register preserved; the flags of the `xor`
//!
//! `g_yesNoAnswer` (0x75524C) `xor` 1. The left and right action of the question's
//! button (the menu table at 0x7544D8).
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, game_write, rva};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

const ANSWER_RVA: usize = rva(0x75524C); // g_yesNoAnswer

// ---- Rust version ----

pub fn flip_yes_no() -> u32 {
    unsafe {
        // Converted: `rc2k_core::menu::steps::flip`.
        let answer = rc2k_core::menu::steps::flip(game_read::<u32>(ANSWER_RVA));
        game_write(ANSWER_RVA, answer);
        answer
    }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    flip_yes_no();
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
