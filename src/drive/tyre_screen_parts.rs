//! `drive::tyre_screen_parts` (0x41C790): the tyre-change screen's fixed parts: ten quads
//! (`drive::tyre_quad`, place 2) from texture rectangles set in `dword_5C2A1C` and screen
//! rectangles in `dword_5C2A0C`, placed from the layout (`dword_5BEBD0` .., `dword_5BEAE0`
//! .., the car's half by `dword_5BEBEC`). All in `pusha` / `popa`.
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `drive::tyre_screen`.

use crate::errors::Refused;
use crate::physics::flow::{a, g, p};
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
    let four = |at: usize| std::array::from_fn(|k| g(at + 4 * k));
    for (uv, rect) in fixed_parts(four(0x5BEBD0), four(0x5BEAE0), g(0x5BEBEC) != 0) {
        set(UV, uv);
        set(RECT, rect);
        unsafe { crate::drive::tyre_quad::tyre_quad(a(UV), a(RECT), 0xFF_FFFF) }?;
    }
    Ok(())
}
