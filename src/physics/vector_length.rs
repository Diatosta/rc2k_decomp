//! `physics::vector_length` (0x421282): the length of the three floats at esi, left in
//! st(0): `fsqrt` of x x + (y y + z z), not rounded to a float.
//!
//! Takes: the vector
//!
//! Callers (17): the cars' physics under `drive::race_step` (0x43B635,
//! `physics::store_motion_length`,
//! `physics::spin_matrix`, `physics::limit_motion`, `physics::stall_when_still`,
//! `physics::update_camera`, `physics::wall_push_back`, `physics::face_contact`,
//! `physics::point_hits_car`, `physics::contact_force`, `physics::bump_cars`, `physics::move_slot`,
//! and three in unowned code at 0x43AD63 ..).

use crate::math::cells::{Cells, load};
use crate::math::x87_run::length_80;

/// The Rust version: the length as it is left in st(0).
pub fn vector_length(v: &(impl Cells + ?Sized)) -> [u8; 10] {
    length_80(load::<3>(v))
}
