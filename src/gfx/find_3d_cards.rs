//! `gfx::find_3d_cards` (0x404270): which display cards can draw in 3D.
//!
//! The primary and secondary 3D card flags (`cards.primary`, `cards.secondary`) are set to
//! 0; unless the "no search" setting (`cards.no_search`) is not 0: "Finding 3d hardware"
//! (log level 1, a line), then `DirectDrawEnumerateA(Callback, 0)`: the game's callback
//! makes DirectDraw for each card, logs " dd:" its description and name, and for one with
//! 3D ("3d:") sets "primary card" (`cards.primary` 1, unless the `ignoreprimary` switch)
//! or "secondary card" (`cards.secondary` 1 and its GUID kept, unless the
//! `ignoresecondary` switch). Then a line break in the log. The enumeration (and the
//! callback, DirectDraw throughout) is the platform's: in the standalone its rasteriser
//! is the one card, a 3D primary one (unless `ignoreprimary`), as the callback would set
//! it. One caller, `start`, which calls it directly.

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
