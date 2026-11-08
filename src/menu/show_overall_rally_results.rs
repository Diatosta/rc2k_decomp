//! `menu::show_overall_rally_results` (0x4A3BA2): the overall rally results: the first player's entry of any group in standings
//! table A (`menu::first_player_a`), its figure held to 0x51 when above 0xAB (the original's: section 16) into `dword_7556B4`;
//! `menu::first_page_stage`, `menu::stage_range_caption`; then the page
//! (`menu::overall_rally_results_loop`).
//!
//! The steps are their Rust versions. Refuses what the
//! steps refuse, before anything; raises the search's fatal error as the original.

use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::menu::screen_loop::{Entry, run_screen};
use crate::menu::standings::*;
use rc2k_core::menu::results_pages::{Page, held_figure};

// dword_7558F4
// g_eventMode

/// The Rust version.
pub fn show_overall_rally_results(menus: &Menus) -> Result<(), Refused> {
    let play = &menus.play;
    let found = crate::menu::first_player_a::first_player_a(ANY_GROUP)?;
    // Converted: `rc2k_core::menu::results_pages::held_figure`.
    let count = held_figure(Page::OverallRally, found.figure);
    menus.play.champ_view.count.set(count);
    crate::menu::first_page_stage::first_page_stage(play);
    crate::menu::stage_range_caption::stage_range_caption(play)?;
    run_screen(
        &crate::menu::overall_rally_results_loop::SCREEN,
        &Entry::new(menus),
    )?;
    Ok(())
}
