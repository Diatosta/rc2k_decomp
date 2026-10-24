//! `scene::find_object_of_kind` (0x422298): the first object (1 .. 0xFF) of a kind.
//!
//! In:  eax = the kind
//! Out: eax = the object, 0 if none; the rest preserved
//!
//! `g_objectKinds` (0x5CD27C, 0x100 dwords; object 0 is never looked at). 16 call sites
//! (loadlevel_init, the carousels and car pictures, the replay cameras). Self-test: the
//! original and the Rust version on the game's table, kinds of the test's, put back
//! afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{game_read, rva};

const KINDS_RVA: usize = rva(0x5CD27C); // g_objectKinds

/// `scene::find_object_of_kind` (see the header).
pub unsafe fn find_object_of_kind(kind: u32) -> u32 {
    // Converted: `rc2k_core::scene::kinds::first_of_kind`.
    rc2k_core::scene::kinds::first_of_kind(|n| unsafe { game_read(KINDS_RVA + n * 4) }, kind)
}
