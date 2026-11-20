//! `menu::show_overall_rally_results`: the overall rally results: the first player's entry of
//! any group in standings table A (`menu::first_player_a`), its figure held to 0xAB into `champ_view.rows.count`; `menu::first_page_stage`, `menu::stage_range_caption`; then
//! the page (`menu::overall_rally_results_loop`).
//!
//! Refuses what the steps refuse, before anything; raises the search's fatal error.

use crate::errors::Refused;
use crate::menu::champ_store::ChampView;
use crate::menu::screen_id::ScreenId;
use crate::menu::screen_loop::{Entry, run_screen};
use crate::menu::standings::*;
use rc2k_core::menu::results_pages::{Page, held_figure};
pub(super) fn show_overall_rally_results(
    champ: &ChampView,
    entry: &Entry,
    tables: &crate::saves::best_times::BestTables,
) -> Result<(), Refused> {
    let found = crate::menu::first_player_a::first_player_a(ANY_GROUP, tables)?;
    let count = held_figure(Page::OverallRally, found.figure);
    champ.rows.count.set(count);
    crate::menu::first_page_stage::first_page_stage(champ);
    crate::menu::stage_range_caption::stage_range_caption(champ, &entry.menus.text_buffers)?;
    run_screen(ScreenId::OverallRallyResults, entry)?;
    Ok(())
}
