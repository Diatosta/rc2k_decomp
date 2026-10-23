//! `physics::scale_vector` (0x421297): the three floats at esi each times st(0)
//! (`fld; fmul st, st(1); fstp`, x, y, z), st(0) left as it was.
//!
//! In:  esi the vector, st(0) the factor
//! Out: the vector scaled; st(0) the factor still; every register kept
//!
//! The thunk (here, not a shared macro) pops the factor onto its stack, and inside the
//! gate (as `hook_asm!`'s) copies it into `FACTOR`, runs `dispatch` and copies `KEPT`
//! (what the original or the Rust version left in st(0)) back for loading; `original_with_factor` loads the factor, runs the original and pops its
//! st(0) into `KEPT`, so shadow mode compares that too.
//! Callers (12): the cars' physics under `sub_4454D2` (`sub_43BDD1`, `sub_43DEF4`,
//! `sub_43F7AC`, `sub_45F17E`, `sub_45F7B1`, `sub_45FAE1`, two in unowned code at
//! 0x43F5A0, 0x43F647).
//! Self-test: the original and the Rust version on vectors and factors by lot, under three
//! precisions.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::math::x87_run::scale_by_80;

/// The Rust version: the vector at `v` times the 80-bit `factor`.
pub unsafe fn scale_vector(v: u32, factor: [u8; 10]) {
    unsafe { scale_by_80(v as *mut u32, factor) }
}
