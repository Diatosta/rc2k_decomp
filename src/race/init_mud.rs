//! `race::init_mud` (0x449DD6): the mud set up (`render3d initmud`).
//!
//! In:  nothing
//! Out: the registers as `race::sample_mud_colours` leaves them
//!
//! See `race/mud.rs`. One caller, `race::loadlevel_init`. Shadow mode runs the original
//! alone (it takes memory). Self-test: a redirected copy whose four calls note themselves,
//! the game's variables put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::mud::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let g = unsafe { init_mud(r) }?;
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..g
        }
    }
    Ok(())
}
