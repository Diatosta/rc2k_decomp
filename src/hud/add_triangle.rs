//! `hud::add_triangle` (0x4B6DA7): `g_polygonVertexCount` 3 and `hud::add_polygon`.
//!
//! In:  nothing
//! Out: all kept (`pusha` .. `popa`)
//!
//! Caller: `sub_4B6D10` (0x4B6D03).

use crate::errors::Refused;
use crate::hud::batch::POLYGON_COUNT_RVA;
use crate::hud::store::game_write;

/// The Rust version (see the header).
pub unsafe fn add_triangle() -> Result<(), Refused> {
    unsafe {
        game_write(POLYGON_COUNT_RVA, 3u32);
        crate::hud::add_polygon::add_polygon()
    }
}
