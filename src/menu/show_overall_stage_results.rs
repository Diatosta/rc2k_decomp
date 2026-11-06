//! `menu::show_overall_stage_results` (0x4A1FE5): the overall stage results: the first player's entry of any group in standings
//! table A (`menu::first_player_a`), its figure held to 0xAB into `dword_7556B4`;
//! `menu::first_page_stage`, `menu::stage_range_caption`; then the page
//! (`menu::overall_stage_results_loop`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `sub_4A0E60`. The steps are their Rust versions. Refuses what the steps refuse, before
//! anything; raises the search's fatal error as the original.

use crate::errors::Refused;
use crate::menu::screen_loop::{Entry, run_screen};
use crate::menu::standings::*;
use rc2k_core::menu::results_pages::{Page, held_figure};

// dword_7558F4
// g_eventMode

/// The Rust version (see the header).
pub fn show_overall_stage_results() -> Result<(), Refused> {
    let found = crate::menu::first_player_a::first_player_a(ANY_GROUP)?;
    // Converted: `rc2k_core::menu::results_pages::held_figure`.
    let count = held_figure(Page::OverallStage, found.figure);
    crate::menu::owner::menus().play.champ_view.count.set(count);
    crate::menu::first_page_stage::first_page_stage();
    crate::menu::stage_range_caption::stage_range_caption()?;
    run_screen(
        &crate::menu::overall_stage_results_loop::SCREEN,
        &Entry::default(),
    )?;
    Ok(())
}
