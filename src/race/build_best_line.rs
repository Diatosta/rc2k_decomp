//! `race::build_best_line` (0x445772): the stage's best line built, step by step, with its
//! log.
//!
//! In:  nothing
//! Out: the registers as the last step leaves them
//!
//! See `race/best_line.rs`. Callers: `race::locate_current_course` and `sub_4462B9` (race
//! init). Shadow mode runs the original alone (it logs). Self-test: a redirected copy
//! whose 27 calls note themselves, with two line faces and with one, against the Rust
//! version noting the same.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::best_line::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let g = unsafe { build_best_line(r, &mut build_part) }?;
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..g
        }
    }
    Ok(())
}
