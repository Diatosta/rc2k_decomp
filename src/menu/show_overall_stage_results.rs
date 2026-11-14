//! `menu::show_overall_stage_results`: the overall stage results: the first player's entry of
//! any group in standings table A (`menu::first_player_a`), its figure held to 0xAB into
//! `champ_view.count`; `menu::first_page_stage`, `menu::stage_range_caption`; then the page
//! (`menu::overall_stage_results_loop`).
//!
//! Refuses what the steps refuse, before anything; raises the search's fatal error.

use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::menu::screen_loop::{Entry, run_screen};
use crate::menu::standings::*;
use rc2k_core::menu::results_pages::{Page, held_figure};
pub fn show_overall_stage_results(menus: &Menus, entry: &Entry) -> Result<(), Refused> {
    let play = &menus.play;
    let found = crate::menu::first_player_a::first_player_a(ANY_GROUP)?;
    let count = held_figure(Page::OverallStage, found.figure);
    menus.play.champ_view.count.set(count);
    crate::menu::first_page_stage::first_page_stage(play);
    crate::menu::stage_range_caption::stage_range_caption(play)?;
    run_screen(&crate::menu::overall_stage_results_loop::SCREEN, entry)?;
    Ok(())
}
