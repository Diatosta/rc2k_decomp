//! `race::allocate_course_memory` (0x41F8C4): the course's memory taken (`alloc coursemem`).
//!
//! In:  nothing
//! Out: eax = the memory's end, ebx = the list; the rest as `memory::allocate` leaves them
//!
//! See `race/level_init.rs`. One caller, `sub_44426F` (`race::load_stage`'s
//! "loadlevel_init"). Shadow mode runs the original alone. Self-test: a redirected copy
//! whose variables and memory are the test's and whose calls note themselves.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::level_init::*;

fn rust(r: &Regs, v: &Vars) -> Result<Regs, crate::errors::Refused> {
    unsafe { allocate_course_memory(r, v, crate::framework::game_addr(COURSE_LIST_RVA) as u32) }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let left = rust(r, &Vars::game())?;
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..left
        }
    }
    Ok(())
}
