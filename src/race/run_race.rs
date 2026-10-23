//! `race::run_race` (0x443F0B): a race, start to end: the settings in, the arcade's cars,
//! the start states, the stage loaded (`race::load_stage`), the replay opened, the race
//! run (`sub_444CEE`, stream B's: in the game until ported), the replay closed,
//! everything tidied, the settings out. An arcade race's first stage draws a new noise
//! seed first.
//!
//! In:  nothing
//! Out: the registers as the last step leaves them
//!
//! See `race/race_end.rs`. One caller, `sub_4278B7` (a `retn` follows). Its steps are the
//! Rust versions but the race's run (section 15); a step that refuses runs in the game
//! through its hook. Shadow mode runs the original alone (it runs the race). Self-test: a
//! copy whose eleven calls go to stand-ins, its one reference to `g_logIndent` the test's,
//! against the Rust version on the same: arcade first stage and not.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{Regs, game_addr, rva};
use crate::race::race_end::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let indent = game_addr(rva(0x4E00A0)) as u32; // g_logIndent
    {
        let g = unsafe { run_race(r, indent) }?;
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..g
        }
    }
    Ok(())
}
