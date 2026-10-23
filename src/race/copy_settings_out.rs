//! `race::copy_settings_out` (0x4486FB): "COPY SHAUN TO GRAHAM": after a race (not a
//! replay's, `dword_5BEC94`) the race's variables and the two players' blocks back into
//! the front end's (copies, floats to whole numbers), the results by mode
//! (`race::store_results_by_mode`), each logged.
//!
//! In:  nothing
//! Out: eax as the steps leave it, the rest as `race::store_results_by_mode`
//!
//! See `race/settings_copy.rs` (the steps a table made from the disassembly by script).
//! One caller, `sub_443F0B` (its last step; no flags read). Shadow mode runs the original
//! alone (it logs). Self-test: a copy whose 228 calls go to stand-ins, against the Rust
//! version on the same: every dword it reads set eight ways, one and two players, a
//! replay's and not.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::settings_copy::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let g = unsafe { copy_settings_out(r, &mut game_part) }?;
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..g
        }
    }
    Ok(())
}
