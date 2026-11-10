//! `gfx::find_3d_cards` (0x404270): which display cards can draw in 3D.
//!
//! `dword_5AB4AC` (a 3D primary card) and `dword_5AB4B0` (a 3D secondary card) 0; unless
//! `dword_5AB94C` (a setting) is not 0: "Finding 3d hardware" (log level 1, a line), then
//! `DirectDrawEnumerateA(Callback, 0)`: the game's `Callback` (0x4042AE) makes DirectDraw
//! for each card, logs " dd:" its description and name, and for one with 3D ("3d:") sets
//! "primary card" (`dword_5AB4AC` 1, unless `dword_5AB950`) or "secondary card"
//! (`dword_5AB4B0` 1 and its GUID to `unk_5AB4B4`, unless `dword_5AB954`). Then a line
//! break in the log. The enumeration (and the callback, DirectDraw throughout) is the
//! platform's: in the standalone its rasteriser is the one card, a 3D primary one (unless
//! `dword_5AB950`), as the callback would set it. One caller, `start` (0x401093), which
//! calls it directly.

use crate::d3d::owner::d3d;
use crate::gfx::owner::Display;
use crate::logfile::log_write::log_newline_l1;
pub fn find_3d_cards(display: &Display) {
    let d3d = d3d();
    display.cards.primary.set(0);
    display.cards.secondary.set(0);
    if d3d.cards.no_search.get() == 0 {
        crate::logfile::log_write::log_nul(&crate::gfx::texts::FINDING_3D, true, 1);
        if d3d.cards.not_primary.get() == 0 {
            display.cards.primary.set(1);
        }
    }
    log_newline_l1();
}
