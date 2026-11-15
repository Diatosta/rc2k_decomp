//! `menu::championship_standings`: the championship's standings pages,
//! `state.menus.standings_shown` set while they show. Standings table B from all
//! (`menu::standings_b_from_all`); outside event mode 0x13: sorted by figure and by score
//! (`menu::sort_standings_b_by_figure`, `_by_score`), numbered (`number_all`), the
//! overall page (`menu::show_championship_standings`); in it: sorted by figure twice and
//! by points (`_by_points`), the class pages (`menu::class_championship_pass`).
//!
//! Refuses what the steps refuse. The
//! menus and draw lists run its `entry` (`menu::table_calls`).

use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::menu::screen_loop::Entry;
use crate::menu::standings::Which;
use crate::menu::standings_steps::number_all;

pub fn championship_standings(menus: &Menus, entry: &Entry) -> Result<(), Refused> {
    use crate::menu::{sort_standings_b_by_figure as fig, sort_standings_b_by_points as pts};
    use rc2k_core::menu::results_pages::{Sort, class_pages_only, standings_sorts};
    crate::menu::standings_b_from_all::standings_b_from_all(
        crate::strings::append_string_by_id::StringOwners::of(entry.owners),
        menus,
    )?;
    let a8 = class_pages_only(menus.play.champ_view.event_mode.get());
    for sort in standings_sorts(a8) {
        match sort {
            Sort::ByFigure => fig::sort_standings_b_by_figure(menus)?,
            Sort::ByScore => {
                crate::menu::sort_standings_b_by_score::sort_standings_b_by_score(menus)?;
            }
            Sort::ByPoints => pts::sort_standings_b_by_points(menus)?,
        }
    }
    if !a8 {
        number_all(menus, Which::B)?;
    }
    menus.screens.state.menus.standings_shown.set(true);
    if a8 {
        crate::menu::class_championship_pass::class_championship_pass(menus, entry)?;
    } else {
        crate::menu::show_championship_standings::show_championship_standings(
            &menus.play.champ_view,
            entry,
        )?;
    }
    menus.screens.state.menus.standings_shown.set(false);
    Ok(())
}
