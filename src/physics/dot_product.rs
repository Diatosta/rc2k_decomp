//! `physics::dot_product` (0x4212AE): esi . edi, x x' + (y y' + z z'), left in st(0)
//! unrounded. The same instructions as `scene::dot_product3` (0x46553C), whose Rust
//! version (`scene::vector3::dot_product3`) it uses.
//!
//! In:  esi, edi the two vectors (three floats)
//! Out: st(0) the product (80 bits), every register kept
//!
//! Callers (5): the cars' physics under `sub_4454D2` (`sub_440B9D`, `sub_45EDB5`,
//! `sub_45F17E` three times).

use crate::scene::vector3::*;

/// The Rust version: the product as it is left in st(0).
pub unsafe fn dot_product(a: u32, b: u32) -> [u8; 10] {
    unsafe { dot_product3(a, b) }
}
