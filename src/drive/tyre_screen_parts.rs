//! `drive::tyre_screen_parts` (0x41C790): the tyre-change screen's fixed parts: ten quads
//! (`drive::tyre_quad`, place 2) from texture rectangles set in `dword_5C2A1C` and screen
//! rectangles in `dword_5C2A0C`, placed from the layout (`TyreChange::parts`, `wheel`,
//! the car's half by `player`). All in `pusha` / `popa`.
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `drive::tyre_screen`.

use crate::errors::Refused;
use crate::physics::flow::{a, p};
use crate::race::tyre_store::TYRE_CHANGE as TYRE;
use rc2k_core::drive::tyre_screen::fixed_parts;

const UV: usize = 0x5C2A1C;
const RECT: usize = 0x5C2A0C;

fn set(at: usize, v: [u32; 4]) {
    for (k, x) in v.iter().enumerate() {
        p(at + 4 * k, *x);
    }
}

/// The Rust version: its quads (`drive::tyre_quad`); every register kept.
// Converted: `rc2k_core::drive::tyre_screen::fixed_parts`.
pub unsafe fn tyre_screen_parts() -> Result<(), Refused> {
    let parts = std::array::from_fn(|k| TYRE.parts[k].get());
    let wheel = std::array::from_fn(|k| TYRE.wheel[k].get());
    for (uv, rect) in fixed_parts(parts, wheel, TYRE.player.get() != 0) {
        set(UV, uv);
        set(RECT, rect);
        unsafe { crate::drive::tyre_quad::tyre_quad(a(UV), a(RECT), 0xFF_FFFF) }?;
    }
    Ok(())
}
