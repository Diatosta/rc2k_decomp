//! `scene::full_view_window` (0x44570A): `scene::choose_view_window` with `word_636526` 2
//! (800 x 600 when framed), the word back to 0 after.
//!
//! In:  nothing
//! Out: as `choose_view_window` leaves them
//!
//! Callers: `sub_45A893` (0x45A906), code at 0x4435B8. Self-test: the original and the
//! Rust version on the game's variables, framed and not.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_write};
use crate::scene::choose_view_window::{MODE_RVA, choose_view_window};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

/// The Rust version (see the header): what it leaves in the registers.
pub unsafe fn full_view_window() -> Left {
    unsafe {
        game_write(MODE_RVA, 2u16);
        let left = choose_view_window();
        game_write(MODE_RVA, 0u16);
        left
    }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { full_view_window() })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
