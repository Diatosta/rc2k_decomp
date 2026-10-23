//! `menu::hide_in_game_options` (0x46FB63): the in-game options' last items hidden.
//!
//! In:  eax = 1, 2 or 3 (anything else: nothing)
//! Out: every register preserved
//!
//! See `menu/in_game_options_items.rs`: items 3 + eax .. 6 get no string, are passed over
//! (+0x48 0xFFFFFC1C) and have nothing behind (+0x1C 0xFFFFFFA9). Callers: the event loops
//! (`sub_482382` with 1) and seven more. Self-test: the game's menu, values 0 .. 4, put
//! back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::menu::in_game_options_items::hide_in_game_options;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

// A `jz` at +4 is inside the first 5 bytes: the jump is written by hand and the original
// runs from a private copy of the whole function (no calls, every jump inside it).
fn rust(r: &Regs) -> Result<Left, Refused> {
    hide_in_game_options(r.eax);
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
