//! `menu::championship_standings` (0x4A9583): the championship's standings pages,
//! `dword_75AB34` 1 while they show. Standings table B from all
//! (`menu::standings_b_from_all`); outside event mode 0x13: sorted by figure and by score
//! (`menu::sort_standings_b_by_figure`, `_by_score`), numbered (`menu::number_all_b`), the
//! overall page (`menu::show_championship_standings`); in it: sorted by figure twice and
//! by points (`_by_points`), the class pages (`menu::class_championship_pass`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `sub_473126`. The steps are their Rust versions; the pages through `part`, in
//! the game their Rust versions. Ungated (the pages wait for the player). Refuses what the
//! steps refuse.
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::{game_read, game_write, rva};
use crate::menu::screen_loop::Carried;

const FLAG_RVA: usize = rva(0x75AB34); // dword_75AB34
const EVENT_MODE_RVA: usize = rva(0x7556D0); // g_eventMode

/// The Rust version (see the header). `c` the carried values (the registers the pages'
/// screens are run with).
pub unsafe fn championship_standings(c: &Carried) -> Result<(), Refused> {
    let mut g = *c;
    unsafe {
        use crate::menu::{sort_standings_b_by_figure as fig, sort_standings_b_by_points as pts};
        // Converted: `rc2k_core::menu::results_pages` (`class_pages_only`, `standings_sorts`).
        use rc2k_core::menu::results_pages::{Sort, class_pages_only, standings_sorts};
        crate::menu::standings_b_from_all::standings_b_from_all()?;
        let a8 = class_pages_only(game_read(EVENT_MODE_RVA));
        for sort in standings_sorts(a8) {
            match sort {
                Sort::ByFigure => fig::sort_standings_b_by_figure()?,
                Sort::ByScore => {
                    crate::menu::sort_standings_b_by_score::sort_standings_b_by_score()?
                }
                Sort::ByPoints => pts::sort_standings_b_by_points()?,
            }
        }
        if !a8 {
            crate::menu::number_all_b::number_all_b()?.carry_on(&mut g);
        }
        game_write(FLAG_RVA, 1u32);
        if a8 {
            crate::menu::class_championship_pass::class_championship_pass(&g)?;
        } else {
            crate::menu::show_championship_standings::show_championship_standings(&g)?;
        }
        game_write(FLAG_RVA, 0u32);
    }
    Ok(())
}
