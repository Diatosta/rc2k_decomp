//! `menu::show_class_championship_standings` (0x4A4AD5): the championship standings of a class: `dword_7558F4` the class (ebx); the first player's entry of class ebx in standings
//! table B (`menu::first_player_b`), its figure held to 0x33 into `dword_7556B4`;
//! `menu::first_page_stage`, `menu::stage_range_caption`; then the page
//! (`menu::class_championship_standings_loop`).
//!
//! In:  ebx the class
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `sub_4A4A1C`. The steps are their Rust versions. Refuses what the steps refuse, before
//! anything; raises the search's fatal error as the original.

use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::menu::screen_loop::{Entry, run_screen};
use rc2k_core::menu::results_pages::{Page, held_figure};

// g_eventMode

/// The Rust version (see the header).
pub fn show_class_championship_standings(menus: &Menus, class: u32) -> Result<(), Refused> {
    menus.screens.event.class.set(class);
    let found = crate::menu::first_player_b::first_player_b(class)?;
    // Converted: `rc2k_core::menu::results_pages::held_figure`.
    let count = held_figure(Page::ClassChampionship, found.figure);
    menus.play.champ_view.count.set(count);
    crate::menu::first_page_stage::first_page_stage(&menus.play);
    crate::menu::stage_range_caption::stage_range_caption(&menus.play)?;
    run_screen(
        &crate::menu::class_championship_standings_loop::SCREEN,
        &Entry::new(menus),
    )?;
    Ok(())
}
