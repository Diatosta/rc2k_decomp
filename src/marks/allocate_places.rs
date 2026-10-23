//! `marks::allocate_places` (0x4C38B1): `init_marks`' step 6, the places' block taken and started.
//!
//! In:  nothing
//! Out: eax, esi the block, ebx the list
//!
//! See `marks/blocks.rs`. One caller, `marks::init_marks`. Shadow mode runs the original alone (memory
//! cannot be taken twice). Self-test: a redirected copy whose calls go to stand-ins (the
//! block a buffer of the test's), the game's variables put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::marks::blocks::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let left = unsafe { allocate_places(r, &mut game_part) }?;
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..left
        }
    }
    Ok(())
}
