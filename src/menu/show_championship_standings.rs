//! `menu::show_championship_standings`: the championship standings: the first player's entry
//! of a group in standings table B (`menu::first_player_unplaced`), its figure held to 0xAB
//! into `champ_view.rows.count`; `menu::first_page_stage`, `menu::stage_range_caption`; then the
//! page (`menu::championship_standings_loop`).
//!
//! Refuses what the steps refuse, before anything; raises the search's fatal error.

use crate::errors::Refused;
use crate::menu::champ_store::ChampView;
use crate::menu::screen_id::ScreenId;
use crate::menu::screen_loop::{Entry, run_screen};
use rc2k_core::menu::results_pages::{Page, held_figure};
pub fn show_championship_standings(
    champ: &ChampView,
    entry: &Entry,
    tables: &crate::saves::best_times::BestTables,
) -> Result<(), Refused> {
    let found = crate::menu::first_player_unplaced::first_player_unplaced(tables)?;
    let count = held_figure(Page::ChampionshipStandings, found.figure);
    champ.rows.count.set(count);
    crate::menu::first_page_stage::first_page_stage(champ);
    crate::menu::stage_range_caption::stage_range_caption(champ, &entry.menus.text_buffers)?;
    run_screen(ScreenId::ChampionshipStandings, entry)?;
    Ok(())
}
