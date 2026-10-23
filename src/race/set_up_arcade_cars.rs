//! `race::set_up_arcade_cars` (0x4468FB): with `dword_62389C` 2, "SETUPARCADECARS" logged,
//! `race::choose_arcade_opponents` with the log indented, `g_raceCars` 4; then each
//! block's +0x24 its kind.
//!
//! In:  nothing
//! Out: the registers as `choose_arcade_opponents` leaves them
//!
//! See `race/race_cars.rs`. Callers: `sub_443F0B`, `sub_443F96`, code at 0x444000,
//! `race::credits_race`, `sub_463A94` (`race::set_start_states` follows each; no flags read).
//! Shadow mode runs the original alone (it logs). Self-test: a copy whose two calls go to
//! stand-ins that note them, its two references to `g_logIndent` the test's, against the
//! Rust version on the same, modes 2 and 0.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{Regs, game_addr, rva};
use crate::race::race_cars::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let indent = game_addr(rva(0x4E00A0)) as u32; // g_logIndent
    {
        let g = unsafe { set_up_arcade_cars(r, indent, &mut cars_part) }?;
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..g
        }
    }
    Ok(())
}
