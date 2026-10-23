//! `physics::dot_product` (0x4212AE): esi . edi, x x' + (y y' + z z'), left in st(0)
//! unrounded. The same instructions as `scene::dot_product3` (0x46553C), whose Rust
//! version (`scene::vector3::dot_product3`) it uses.
//!
//! In:  esi, edi the two vectors (three floats)
//! Out: st(0) the product (80 bits), every register kept
//!
//! Thunk by `scene::vector3::result_hook!`. Callers (5): the cars' physics under
//! `sub_4454D2` (`sub_440B9D`, `sub_45EDB5`, `sub_45F17E` three times).
//! Self-test: the original and the Rust version on vectors by lot, under three precisions.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::scene::vector3::*;

/// The Rust version: the product as it is left in st(0).
pub unsafe fn dot_product(a: u32, b: u32) -> [u8; 10] {
    unsafe { dot_product3(a, b) }
}
