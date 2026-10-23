//! `race::locate_current_course` (0x4394CB): the stage's course data loaded and set up
//! (`LOADLEVEL_INIT : locacurrcourse`).
//!
//! In:  nothing
//! Out: the registers as `race::build_best_line` leaves them
//!
//! See `race/course_data.rs`. One caller, `sub_44426F` (loadlevel_init). Shadow mode runs
//! the original alone (it reads and logs). Self-test: a redirected copy whose four calls
//! note themselves, against the Rust version noting the same.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::course_data::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let g = unsafe { locate_current_course(r, &mut locate_part) }?;
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..g
        }
    }
    Ok(())
}
