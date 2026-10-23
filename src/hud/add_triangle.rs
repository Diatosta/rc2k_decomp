//! `hud::add_triangle` (0x4B6DA7): `g_polygonVertexCount` 3 and `hud::add_polygon`.
//!
//! In:  nothing
//! Out: all kept (`pusha` .. `popa`)
//!
//! Caller: `sub_4B6D10` (0x4B6D03). The polygon through `part` (1), in the game its Rust
//! version. Shadow mode runs the original alone (it draws). Self-test: a copy whose call
//! goes to a stand-in, against the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::game_write;
use crate::hud::batch::POLYGON_COUNT_RVA;

/// The Rust version (see the header).
pub unsafe fn add_triangle() -> Result<(), Refused> {
    unsafe {
        game_write(POLYGON_COUNT_RVA, 3u32);
        crate::hud::add_polygon::add_polygon()
    }
}
