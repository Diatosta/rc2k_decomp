//! `race::loading_bar_quad` (0x446861): the loading bar, a red (`dword_623DE8` 0xFF0000)
//! flat rectangle along the bottom: x from 0 to the screen's width times `dword_623700`
//! (held to 0 .. 100) over 100, y from 0x24D to 0x258 scaled from 600 to the screen's
//! height; `dword_6240C4` -1; `race::draw_screen_quad`.
//!
//! In:  nothing
//! Out: nothing its caller reads (conversion phase 3, 2026-10-24)
//!
//! Caller: `race::loading_bar_frame`. The quad its Rust version.

use crate::errors::Refused;
use crate::framework::{game_write, rva};
use crate::race::vars_store::RACE_VARS;
use crate::scene::draw_store::DRAW;
use crate::session::SCREEN;

const QUAD_IN_RVA: usize = rva(0x623DE8); // colour; x at +0x28 / +0x2C, y at +0x38 / +0x3C

/// The Rust version (see the header).
pub unsafe fn loading_bar_quad() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::loading::loading_bar`.
    use rc2k_core::race::loading::{BAR_COLOUR, loading_bar};
    unsafe {
        let [w, h] = [SCREEN.width.get(), SCREEN.height.get()];
        let q = |o: usize, v: u32| game_write(QUAD_IN_RVA + o, v);
        let bar = loading_bar(w, h, RACE_VARS.bar_share.bits());
        q(0x28, bar.left);
        q(0, BAR_COLOUR);
        q(0x38, bar.top);
        q(0x3C, bar.bottom);
        q(0x2C, bar.right);
        DRAW.texture_page.set_bits(0xFFFF_FFFFu32);
        // `draw_screen_quad` reads no register it is given.
        crate::race::frame_draw::draw_screen_quad()
    }
}
