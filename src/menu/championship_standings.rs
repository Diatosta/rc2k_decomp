//! `menu::championship_standings` (0x4A9583): the championship's standings pages,
//! `dword_75AB34` 1 while they show. Standings table B from all
//! (`menu::standings_b_from_all`); outside event mode 0x13: sorted by figure and by score
//! (`menu::sort_standings_b_by_figure`, `_by_score`), numbered (`number_all`), the
//! overall page (`menu::show_championship_standings`); in it: sorted by figure twice and
//! by points (`_by_points`), the class pages (`menu::class_championship_pass`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `sub_473126`. The steps are their Rust versions. Refuses what the steps refuse. The
//! menus and draw lists run its `entry` (`menu::table_calls`); the registers above are the
//! original's, for reference.

use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::menu::standings::Which;
use crate::menu::standings_steps::number_all;

/// The Rust version (see the header).
pub fn championship_standings(menus: &Menus) -> Result<(), Refused> {
    use crate::menu::{sort_standings_b_by_figure as fig, sort_standings_b_by_points as pts};
    // Converted: `rc2k_core::menu::results_pages` (`class_pages_only`, `standings_sorts`).
    use rc2k_core::menu::results_pages::{Sort, class_pages_only, standings_sorts};
    crate::menu::standings_b_from_all::standings_b_from_all()?;
    let a8 = class_pages_only(menus.play.champ_view.event_mode.get());
    for sort in standings_sorts(a8) {
        match sort {
            Sort::ByFigure => fig::sort_standings_b_by_figure()?,
            Sort::ByScore => crate::menu::sort_standings_b_by_score::sort_standings_b_by_score()?,
            Sort::ByPoints => pts::sort_standings_b_by_points()?,
        }
    }
    if !a8 {
        number_all(Which::B)?;
    }
    menus.screens.state.menus.standings_shown.set(1u32);
    if a8 {
        crate::menu::class_championship_pass::class_championship_pass()?;
    } else {
        crate::menu::show_championship_standings::show_championship_standings()?;
    }
    menus.screens.state.menus.standings_shown.set(0u32);
    Ok(())
}
