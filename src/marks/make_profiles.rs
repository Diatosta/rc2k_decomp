//! `marks::make_profiles` (0x4C3B01): `init_marks`' step 3 (through `sub_4C4E1C`), the
//! rings, the profile tables once, the corners from the course, the links and the
//! settings blocks.
//!
//! In:  nothing
//! Out: as `marks::reset_settings_blocks` leaves them; edi as `marks::scale_corners`,
//!      edx kept
//!
//! See `marks/profiles.rs`. One caller, `sub_4C4E1C`. Refuses a mark neither 0 nor
//! `loc_4C3EA6` (the original jumps to it). Shadow mode runs the original alone (it takes
//! memory). Self-test: a redirected copy whose two calls go to stand-ins, the course's
//! corners and the tables' parameters of the test's (the game's own and others), the
//! mark 0 and set (to the copy's own place), the game's variables put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{Regs, game_addr};
use crate::marks::profiles::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let resume = game_addr(RESUME_RVA) as u32;
    {
        let left = unsafe { make_profiles(r, resume) }?;
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..left
        }
    }
    Ok(())
}
