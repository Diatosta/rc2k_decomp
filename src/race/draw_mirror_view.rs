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
use crate::framework::{game_read, rva};
use crate::race::level_grid_store::LEVEL_GRID;
use crate::race::numbers_store::NUMBERS;
use crate::race::prefs_store::STAGE_PREFS;
use crate::race::switches_store::FRAME_SWITCHES;
use crate::race::view_store::RACE_VIEW;
use crate::race::view_vars_store::VIEW_VARS;
use crate::race::work_store::RACE_WORK;

const NO_PLACE_RVA: usize = rva(0x64BB18); // dword_64BB18

/// The Rust version (see the header).
pub unsafe fn draw_mirror_view() -> Result<(), Refused> {
    unsafe {
        let g = |a: usize| game_read::<u32>(a);
        if g(NO_PLACE_RVA) != 0
            || RACE_WORK.in_credits.get() != 0
            || FRAME_SWITCHES.other_way.get() == 0
        {
            return Ok(());
        }
        LEVEL_GRID
            .view_value_kept
            .set_bits(RACE_VIEW.view_depth[1].bits());
        let saved = (VIEW_VARS.view_scale.bits(), STAGE_PREFS.grid_reach.bits());
        VIEW_VARS.view_scale.set_bits(NUMBERS.k256.bits());
        STAGE_PREFS.grid_reach.set_bits(2u32);
        let drawn = (|| {
            crate::race::frame_views::mirror_view_window()?;
            RACE_VIEW.view_depth[1].set_bits(LEVEL_GRID.view_value_kept.bits());
            crate::race::frame_flush::draw_view_backdrop();
            crate::race::draw_view_scene::draw_view_scene()
        })();
        STAGE_PREFS.grid_reach.set_bits(saved.1);
        VIEW_VARS.view_scale.set_bits(saved.0);
        drawn?;
        crate::race::frame_views::full_view_window(false);
    }
    Ok(())
}
