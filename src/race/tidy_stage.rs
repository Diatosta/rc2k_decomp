//! `race::tidy_stage` (0x444940): the stage unloaded ("LOADLEVEL_TIDY"): the mud, the
//! point records, the marks' tidy (`marks::tidy_marks`, with the log off), the
//! fonts, the map, the textures, the course's two memories, the objects; `g_inRace` 0,
//! `dword_606294` `dword_6235F4`; each step logged with the log indented round it.
//!
//! In:  nothing
//! Out: the registers as the steps leave them
//!
//! See `race/race_end.rs`. One caller, `race::tidy_everything`. Its steps are the Rust
//! versions; a step that refuses runs in the game through
//! its hook. Shadow mode runs the original alone (it logs and frees). Self-test: a copy
//! whose 19 calls go to stand-ins, its 22 references to `g_logIndent`, `g_logEnabled`
//! and `g_inRace` the test's, against the Rust version on the same.
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
        let g = unsafe { tidy_stage(r, &LoadVars::game(), &mut tidy_part) }?;
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..g
        }
    }
    Ok(())
}
