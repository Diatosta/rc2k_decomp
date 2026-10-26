//! `physics::vector_length` (0x421282): the length of the three floats at esi, left in
//! st(0): `fsqrt` of x x + (y y + z z), not rounded to a float.
//!
//! In:  esi the vector
//! Out: st(0) the length (80 bits), every register kept
//!
//! Callers (17): the cars' physics under `sub_4454D2` (`sub_43B635`, `sub_43BD2D`,
//! `sub_43DD54`, `sub_43F7AC`, `sub_440B9D`, `sub_437A53`, `sub_45D8FB`, `sub_45E3BF`,
//! `sub_45E800`, `sub_45F17E`, `sub_45F7B1`, `sub_462485`, and three in unowned code at
//! 0x43AD63 ..).

use crate::math::x87_run::length_80;

/// The Rust version: the length as it is left in st(0).
pub fn vector_length(v: u32) -> [u8; 10] {
    length_80(v as *const u32)
}
