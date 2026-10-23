//! `race::tidy_everything` (0x444AC7): the race tidied ("SHAUNTIDYEVERYTHING"): the
//! controls, the game mode, the stage (`race::tidy_stage`), the sounds; `dword_63DB74`
//! and `dword_63DB78` 1; each step logged with the log indented round it.
//!
//! In:  nothing
//! Out: the registers as the steps leave them
//!
//! See `race/race_end.rs`. Seven callers (`sub_443F0B`, `sub_443F96`, menus; none reads
//! flags after it). It begins with its call: private copy, its 14 calls relocated, manual
//! patch. A step that refuses runs in the game through its hook. Shadow mode runs the
//! original alone (it logs and frees). Self-test: a copy whose 14 calls go to stand-ins,
//! its 10 references to `g_logIndent` the test's, against the Rust version on the same.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::loadlevel::LoadVars;
use crate::race::race_end::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let g = unsafe { tidy_everything(r, &LoadVars::game(), &mut tidy_part) }?;
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..g
        }
    }
    Ok(())
}
