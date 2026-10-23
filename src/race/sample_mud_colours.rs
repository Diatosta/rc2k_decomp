//! `race::sample_mud_colours` (0x44A0D6): 256 colours taken from a picture surface.
//!
//! In:  nothing
//! Out: eax the surface's number, ecx and edx the last place, edi past the colours, ebp
//!      0x100; the rest preserved
//!
//! See `race/mud.rs`. The surface's lock and unlock are the game's (DirectDraw). One
//! caller, `race::init_mud`. Self-test: a redirected copy whose lock and unlock note
//! themselves, surfaces of the test's (16 and 32 bits), the game's colours put back
//! afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::mud::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let g = unsafe { sample_mud_colours(r) }?;
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..g
        }
    }
    Ok(())
}
