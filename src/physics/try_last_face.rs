//! `physics::try_last_face` (0x441E0C): ebx = `dword_5CCC78` (the face found last); when
//! it is not -1 and below `g_courseFaceCount`, `physics::face_under_point` on it inside
//! `pusha` / `popa`. CF set when the point is still on it.
//!
//! In:  nothing
//! Out: ebx = `dword_5CCC78`; CF; the rest kept
//!
//! See `physics/ground.rs`. Caller: `sub_441E2F`, first (`jb`), every search.
//! Self-test: a copy of the original (its call to a copy of `face_under_point`) on the
//! test's ground: last faces -1, every face, the count and past it, against the test's
//! points.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::physics::ground::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

/// The Rust version: `r` as the original leaves it, and the carry.
pub unsafe fn try_last_face(r: &mut Regs) -> Result<bool, Refused> {
    unsafe { try_last_face_at(&Ground::game(), r) }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let mut g = *r;
    let carry = unsafe { try_last_face(&mut g) }?;
    Ok(left_of(&g, carry))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
