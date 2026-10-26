//! `menu::show_in_game_options` (0x46FBE3): the in-game options' items 3 .. 6 shown.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `menu/in_game_options_items.rs`: strings 0x171 .. 0x174, choosable (+0x48
//! 0xFFFFFC1B), +0x1C 0xFFFFFFA7. Callers: the event loops at their end (`sub_482382`) and
//! seven more. Self-test: the game's menu filled with 0xAA, put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::menu::in_game_options_items::show_in_game_options;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(_: &Regs) -> Result<Left, Refused> {
    show_in_game_options();
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
