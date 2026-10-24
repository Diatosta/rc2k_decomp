//! `hud::set_batch_depth` (0x4B726C): the four vertices of `g_polygonVertices` given the
//! HUD's rhw (`flt_75BFEC`), depth (`flt_75BFE8`) and specular 0.
//!
//! In:  nothing
//! Out: all kept
//!
//! Caller: `sub_4B1CD0` (the HUD's start).

use crate::hud::batch::POLYGON_RVA;
use crate::hud::store::Slot;
use crate::hud::store::{game_read, game_write};
use crate::math::x87;

const DEPTH_RVA: Slot = Slot::at(0x75BFE8); // flt_75BFE8
const RHW_RVA: Slot = Slot::at(0x75BFEC); // flt_75BFEC

/// The Rust version (see the header).
pub unsafe fn set_batch_depth() {
    unsafe {
        let rhw = x87::loaded(game_read(RHW_RVA));
        let depth = x87::loaded(game_read(DEPTH_RVA));
        for n in 0..4 {
            let v = POLYGON_RVA + 0x20 * n;
            game_write(v + 0xC, rhw);
            game_write(v + 0x14, 0u32);
            game_write(v + 8, depth);
        }
    }
}
