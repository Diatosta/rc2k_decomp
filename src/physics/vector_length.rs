//! The length of a vector of three floats, left on the x87 stack: the square root of x x plus (y y
//! plus z z), not rounded to a float.

use crate::math::cells::{Words, load};
use crate::math::x87::length_80;

/// The length as it is left in st(0).
pub fn vector_length(v: &(impl Words + ?Sized)) -> [u8; 10] {
    length_80(load::<3>(v))
}
