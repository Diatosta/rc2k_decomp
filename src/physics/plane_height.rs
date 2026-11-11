//! `physics::plane_height` (0x4425B6): `ground_height` = c + (a - c) `plane_f` + (b -
//! c) `plane_g`, `fistp`'d (the control word's rounding), with a, b, c the whole
//! numbers `dword_639718`, `dword_639724`, `dword_63970C`: by the look of it the height
//! of a course face's plane under a point, from its corners' heights and the point's
//! shares (a reading; set by `physics::face_under_point`).
//!
//! Callers (2): `physics::face_under_point` (the face under a point, the ground's search) and
//! 0x457B86.

use crate::physics::ground_query::GroundQuery;

pub fn plane_height(q: &GroundQuery<'_>) {
    let h = crate::math::x87::plane_height(
        q.corners[4].get() as i32,
        q.corners[7].get() as i32,
        q.corners[1].get() as i32,
        q.plane_f.bits(),
        q.plane_g.bits(),
    );
    q.height.set(h as u32);
}
