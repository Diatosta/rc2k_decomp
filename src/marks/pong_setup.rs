//! `marks::pong_setup` (0x4B88CB): the pong game's frame set up, as
//! `marks::squares_setup` with the texture places from `dword_769222` .. (the original
//! also loads `dword_76921E` into eax and drops it).
//!
//! In:  nothing
//! Out: nothing (its caller reads no register after it)
//!
//! Caller: `marks::run_pong`.

use crate::marks::squares_setup::setup_with;
use crate::marks::store::Slot;

const PLACES_RVA: Slot = Slot::at(0x769222); // dword_769222 .. dword_76922E
// g_polygonVertices

/// The Rust version (see the header).
pub unsafe fn pong_setup() {
    unsafe { setup_with(PLACES_RVA) }
}
