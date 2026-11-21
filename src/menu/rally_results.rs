//! The rally's results pages: the standings table filled from the drivers and players, then the
//! overall page or, in class mode, one page per class.

use crate::errors::Refused;
use crate::menu::owner::{Cars, Play, Screens};
use crate::menu::pages_store::MenuPages;
use crate::menu::screen_loop::Entry;
use crate::menu::standings::Which;
use crate::menu::standings_steps::{clear_standings, number_all};

/// Shows the rally's results; refuses what its steps refuse.
pub fn rally_results(
    cars: &Cars,
    screens: &Screens,
    play: &Play,
    pages: &MenuPages,
    entry: &Entry,
    tables: &crate::saves::best_times::BestTables,
) -> Result<(), Refused> {
    screens.state.write().menus.which_title = true;
    crate::menu::standings_a_from_all::standings_a_from_all(
        crate::strings::append_string_by_id::StringOwners::of(entry.owners),
        cars,
        screens,
        play,
        tables,
    )?;
    if !rc2k_core::menu::results_pages::class_pages_only(play.champ_view.read().event_mode) {
        crate::menu::sort_standings_by_total::sort_standings_by_total(screens, play, tables)?;
        clear_standings(Which::A, tables)?;
        number_all(screens, play, Which::A, tables)?;
        crate::menu::show_overall_rally_results::show_overall_rally_results(
            &play.champ_view,
            entry,
            tables,
        )?;
    } else {
        crate::menu::class_rally_results_pass::class_rally_results_pass(
            screens, play, pages, entry, tables,
        )?;
    }
    screens.state.write().menus.which_title = false;
    Ok(())
}
