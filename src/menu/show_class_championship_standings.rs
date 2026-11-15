//! `menu::show_class_championship_standings`: the championship standings of a class:
//! `event.shown.class` the class; the first player's entry of that class in standings table B
//! (`menu::first_player_b`), its figure held to 0x33 into `champ_view.rows.count`;
//! `menu::first_page_stage`, `menu::stage_range_caption`; then the page
//! (`menu::class_championship_standings_loop`).
//!
//! Refuses what the steps refuse, before anything; raises the search's fatal error.

use crate::errors::Refused;
use crate::menu::champ_store::ChampView;
use crate::menu::event_store::EventState;
use crate::menu::screen_id::ScreenId;
use crate::menu::screen_loop::{Entry, run_screen};
use rc2k_core::menu::results_pages::{Page, held_figure};

pub fn show_class_championship_standings(
    champ: &ChampView,
    event: &EventState,
    class: u32,
    entry: &Entry,
    tables: &crate::saves::best_times::BestTables,
) -> Result<(), Refused> {
    event.shown.class.set(class);
    let found = crate::menu::first_player_b::first_player_b(class, tables)?;
    let count = held_figure(Page::ClassChampionship, found.figure);
    champ.rows.count.set(count);
    crate::menu::first_page_stage::first_page_stage(champ);
    crate::menu::stage_range_caption::stage_range_caption(champ)?;
    run_screen(ScreenId::ClassChampionshipStandings, entry)?;
    Ok(())
}
