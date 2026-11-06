//! `menu::show_class_stage_results` (0x4A1F7A): the stage results of a class: `dword_7558F4` the class (ebx); the first player's entry of class ebx in standings
//! table A (`menu::first_player_a`), its figure held to 0x33 into `dword_7556B4`;
//! `menu::first_page_stage`, `menu::stage_range_caption`; then the page
//! (`menu::class_stage_results_loop`).
//!
//! In:  ebx the class
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: `sub_4A1EE6`. The steps are their Rust versions. Refuses what the steps refuse, before
//! anything; raises the search's fatal error as the original.

use crate::errors::Refused;
use crate::menu::screen_loop::{Entry, run_screen};
use rc2k_core::menu::results_pages::{Page, held_figure};

// g_eventMode

/// The Rust version (see the header).
pub fn show_class_stage_results(class: u32) -> Result<(), Refused> {
    crate::menu::owner::menus().screens.event.class.set(class);
    let found = crate::menu::first_player_a::first_player_a(class)?;
    // Converted: `rc2k_core::menu::results_pages::held_figure`.
    let count = held_figure(Page::ClassStage, found.figure);
    crate::menu::owner::menus().play.champ_view.count.set(count);
    crate::menu::first_page_stage::first_page_stage();
    crate::menu::stage_range_caption::stage_range_caption()?;
    run_screen(
        &crate::menu::class_stage_results_loop::SCREEN,
        &Entry::default(),
    )?;
    Ok(())
}
