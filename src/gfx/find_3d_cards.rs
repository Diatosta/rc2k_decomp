//! `gfx::find_3d_cards` (0x404270): which display cards can draw in 3D.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//! `dword_5AB4AC` (a 3D primary card) and `dword_5AB4B0` (a 3D secondary card) 0; unless
//! `dword_5AB94C` (a setting) is not 0: "Finding 3d hardware" (log level 1, a line), then
//! `DirectDrawEnumerateA(Callback, 0)`: the game's `Callback` (0x4042AE) makes DirectDraw
//! for each card, logs " dd:" its description and name, and for one with 3D ("3d:") sets
//! "primary card" (`dword_5AB4AC` 1, unless `dword_5AB950`) or "secondary card"
//! (`dword_5AB4B0` 1 and its GUID to `unk_5AB4B4`, unless `dword_5AB954`). Then a line
//! break in the log. The enumeration (and the callback, DirectDraw throughout) is the
//! platform's: in the standalone its rasteriser is the one card, a 3D primary one (unless
//! `dword_5AB950`), as the callback would set it. One caller, `start` (0x401093), which
//! calls it directly (2026-10-23).

use crate::framework::rva;
use crate::gfx::cards::CARDS;
use crate::logfile::log_write::{log_newline_l1, log_write};
use crate::store::game_read;

const NO_SEARCH_RVA: usize = rva(0x5AB94C); // dword_5AB94C: not 0, no search
const NOT_PRIMARY_RVA: usize = rva(0x5AB950); // dword_5AB950: no 3D primary card

/// The Rust version (see the header).
pub fn find_3d_cards() {
    unsafe {
        CARDS.primary.set(0);
        CARDS.secondary.set(0);
        if game_read::<u32>(NO_SEARCH_RVA) == 0 {
            log_write(
                crate::gfx::texts::address(&crate::gfx::texts::FINDING_3D) as *const u8,
                true,
                1,
            );
            if game_read::<u32>(NOT_PRIMARY_RVA) == 0 {
                CARDS.primary.set(1);
            }
        }
    }
    log_newline_l1();
}
