//! `race::draw_mirror_view` (0x454806): the mirror's view, unless `dword_64BB18` or
//! `dword_71C3E0` is set or `dword_621480` (the mirror on, a reading) is 0:
//! `dword_63DC58` kept in `dword_63E170`; `dword_606294` and `g_gridReach` saved, made
//! `flt_5C5E2C`'s value and 2; the mirror's window (`race::mirror_view_window`),
//! `dword_63DC58` back from `dword_63E170`, the backdrop (`race::draw_view_backdrop`), the
//! scene (`race::draw_view_scene`); the two put back; the full view's window again
//! (`race::full_view_window_2`).
//!
//! In:  nothing (its steps read no register it gives them)
//! Out: nothing its one caller, `race::draw_race_scene`, reads

use crate::errors::Refused;
use crate::framework::{game_read, game_write, rva};

const SWITCHES: [usize; 3] = [rva(0x64BB18), rva(0x71C3E0), rva(0x621480)];
const VIEW_VALUE_RVA: usize = rva(0x63DC58); // dword_63DC58
const VIEW_VALUE_KEPT_RVA: usize = rva(0x63E170); // dword_63E170
const FAR_RVA: usize = rva(0x606294); // dword_606294
const MIRROR_FAR_RVA: usize = rva(0x5C5E2C); // flt_5C5E2C
const GRID_REACH_RVA: usize = rva(0x60A87C); // g_gridReach

/// The Rust version (see the header).
pub unsafe fn draw_mirror_view() -> Result<(), Refused> {
    unsafe {
        let g = |a: usize| game_read::<u32>(a);
        if g(SWITCHES[0]) != 0 || g(SWITCHES[1]) != 0 || g(SWITCHES[2]) == 0 {
            return Ok(());
        }
        game_write(VIEW_VALUE_KEPT_RVA, g(VIEW_VALUE_RVA));
        let saved = (g(FAR_RVA), g(GRID_REACH_RVA));
        game_write(FAR_RVA, g(MIRROR_FAR_RVA));
        game_write(GRID_REACH_RVA, 2u32);
        let drawn = (|| {
            crate::race::frame_views::mirror_view_window()?;
            game_write(VIEW_VALUE_RVA, g(VIEW_VALUE_KEPT_RVA));
            crate::race::frame_flush::draw_view_backdrop();
            crate::race::draw_view_scene::draw_view_scene()
        })();
        game_write(GRID_REACH_RVA, saved.1);
        game_write(FAR_RVA, saved.0);
        drawn?;
        crate::race::frame_views::full_view_window(false);
    }
    Ok(())
}
