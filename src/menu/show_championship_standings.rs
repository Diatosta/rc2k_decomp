//! `menu::show_championship_standings` (0x4A5A47): the championship standings: the first player's entry of a group in standings
//! table B (`menu::first_player_unplaced`), its figure held to 0xAB into `champ_view.count`;
//! `menu::first_page_stage`, `menu::stage_range_caption`; then the page
//! (`menu::championship_standings_loop`).
//!
//! Refuses what the steps
//! refuse, before anything; raises the search's fatal error as the original.

use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::menu::screen_loop::{Entry, run_screen};
use rc2k_core::menu::results_pages::{Page, held_figure};
// g_eventMode
pub fn show_championship_standings(menus: &Menus) -> Result<(), Refused> {
    let play = &menus.play;
    let found = crate::menu::first_player_unplaced::first_player_unplaced()?;
    // The logic is in `rc2k_core::menu::results_pages::held_figure`.
    let count = held_figure(Page::ChampionshipStandings, found.figure);
    menus.play.champ_view.count.set(count);
    crate::menu::first_page_stage::first_page_stage(play);
    crate::menu::stage_range_caption::stage_range_caption(play)?;
    run_screen(
        &crate::menu::championship_standings_loop::SCREEN,
        &Entry::new(crate::owners::boundary()),
    )?;
    Ok(())
}
