//! `marks::pick_hexagon` (0x4BC6F1): one step of the marks' random (`g_marksSeed`, times
//! 0x9D plus 0x47) split into a place (bits 0 .. 10, 11 .. 20) and a kind (bits 22 .. 24)
//! for `marks::spawn_hexagon`.
//!
//! In:  nothing
//! Out: nothing (its caller reads no register after it)
//!
//! Caller: `marks::hexagons_spawn` (twice). The call is `spawn_hexagon`'s Rust version.
//! Self-test: the original and the Rust version on the game's items, seeds by lot; put back.
//!
//! Converted 2026-10-21: the hook's Rust version is `pick_hexagon_plain`, through
//! `rc2k_core::marks::items::pick_hexagon`; compared exactly as before (the earlier version
//! stays for the Rust callers in unconverted code).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{game_read, game_write};
use crate::marks::setup::SEED_RVA;
use crate::marks::spawn_hexagon::spawn_hexagon;

/// The Rust version (see the header).
pub unsafe fn pick_hexagon() {
    unsafe {
        let s = game_read::<u32>(SEED_RVA)
            .wrapping_mul(0x9D)
            .wrapping_add(0x47);
        game_write(SEED_RVA, s);
        spawn_hexagon(
            s & 0x7FF,
            s.rotate_right(11) & 0x3FF,
            s.rotate_right(22) & 7,
        );
    }
}
