//! `marks::pong_setup` (0x4B88CB): the pong game's frame set up, as
//! `marks::squares_setup` with the texture places from `dword_769222` .. (the original
//! also loads `dword_76921E` into eax and drops it).
//!
//! In:  nothing
//! Out: nothing (its caller reads no register after it)
//!
//! Caller: `marks::run_pong`. The device calls through `part` as `squares_setup`'s.
//! Hooked through a private copy (a call at the entry). Shadow mode runs the original
//! alone (it tells the device). Self-test: a copy whose calls go to stand-ins, against
//! the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::rva;
use crate::marks::squares_setup::setup_with;

const PLACES_RVA: usize = rva(0x769222); // dword_769222 .. dword_76922E
// g_polygonVertices

/// The Rust version (see the header).
pub unsafe fn pong_setup() {
    unsafe { setup_with(PLACES_RVA) }
}
