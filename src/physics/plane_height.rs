//! `physics::plane_height` (0x4425B6): `dword_639828` = c + (a - c) `flt_639810` + (b -
//! c) `flt_639814`, `fistp`'d (the control word's rounding), with a, b, c the whole
//! numbers `dword_639718`, `dword_639724`, `dword_63970C`: by the look of it the height
//! of a course face's plane under a point, from its corners' heights and the point's
//! shares (a reading; set by `sub_442006`).
//!
//! In:  nothing
//! Out: every register kept
//!
//! Callers (2): `sub_442006` (the face under a point, the ground's search) and
//! `sub_457B86`.

use crate::race::owner_physics::Physics;

/// The Rust version.
pub fn plane_height(physics: &Physics) {
    let w = &physics.ground;
    let h = crate::math::x87_run::plane_height(
        w.face_corners[4].get() as i32,
        w.face_corners[7].get() as i32,
        w.face_corners[1].get() as i32,
        w.plane_f.bits(),
        w.plane_g.bits(),
    );
    w.ground_height.set(h as u32);
}
