//! `marks::copy_first_point` (0x4C1163): `dword_769306` .. `dword_76930E` into
//! `flt_811BDF` .. `flt_811BE7`, and `dword_7692F6` into `dword_811BCF` (`push`/`pop`).
//!
//! In:  nothing
//! Out: every register kept
//!
//! Caller: `sub_4C1148`.

use crate::marks::tilt_shape::SHAPE_TILT;
use crate::marks::view_vectors::KEPT;

/// The Rust version (see the header).
pub fn copy_first_point() {
    for (n, to) in SHAPE_TILT.point.iter().enumerate() {
        to.set_bits(KEPT.vector[n].bits());
    }
    SHAPE_TILT.length.set(KEPT.length.get());
}
