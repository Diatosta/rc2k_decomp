//! `physics::scale_vector` (0x421297): the three floats at esi each times st(0)
//! (`fld; fmul st, st(1); fstp`, x, y, z), st(0) left as it was.
//!
//! In:  esi the vector, st(0) the factor
//! Out: the vector scaled; st(0) the factor still; every register kept
//! Callers (12): the cars' physics under `sub_4454D2` (`sub_43BDD1`, `sub_43DEF4`,
//! `sub_43F7AC`, `sub_45F17E`, `sub_45F7B1`, `sub_45FAE1`, two in unowned code at 0x43F5A0,
//! 0x43F647).

use crate::math::x87_run::scale_by_80;

/// The Rust version: the vector at `v` times the 80-bit `factor`.
pub unsafe fn scale_vector(v: u32, factor: [u8; 10]) {
    unsafe { scale_by_80(v as *mut u32, factor) }
}
