//! `race::load_mud` (0x449D95): the car's mud file read into the mud.
//!
//! In:  nothing
//! Out: eax, edx the file number's quotient and remainder by 10 (their low bytes + '0'),
//!      ebx 10, esi the mud; the rest preserved
//!
//! See `race/mud.rs`. One caller, `race::init_mud`. Self-test: a redirected copy whose read
//! notes itself, cars of the test's, the game's path and variables put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::mud::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let g = unsafe { load_mud(r, &mut game_part) }?;
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..g
        }
    }
    Ok(())
}
