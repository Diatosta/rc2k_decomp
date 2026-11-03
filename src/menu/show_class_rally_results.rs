//! `menu::show_class_rally_results` (0x4A3317): the rally results of a class: `dword_7558F4` the class (ebx); the first player's entry of class ebx in standings
//! table A (`menu::first_player_a`), its figure held to 0x51 (event mode 0x13) or 0x33 into `dword_7556B4`;
//! `menu::first_page_stage`, `menu::stage_range_caption`; then the page
//! (`menu::class_rally_results_loop`).
//!
//! In:  ebx the class
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `sub_4A32A5`. The steps are their Rust versions. Refuses what the steps refuse, before
//! anything; raises the search's fatal error as the original.

use crate::errors::Refused;
use crate::menu::champ_store::CHAMP_VIEW;
use crate::menu::event_store::EVENT_STATE;
use crate::menu::screen_loop::{Entry, run_screen};
use rc2k_core::menu::results_pages::{Page, held_figure};

/// The Rust version (see the header).
pub fn show_class_rally_results(class: u32) -> Result<(), Refused> {
    EVENT_STATE.class.set(class);
    let found = crate::menu::first_player_a::first_player_a(class)?;
    // Converted: `rc2k_core::menu::results_pages::held_figure`.
    let a8 = CHAMP_VIEW.event_mode.get() == rc2k_core::menu::results_pages::A8;
    let count = held_figure(Page::ClassRally { a8 }, found.figure);
    CHAMP_VIEW.count.set(count);
    crate::menu::first_page_stage::first_page_stage();
    crate::menu::stage_range_caption::stage_range_caption()?;
    run_screen(
        &crate::menu::class_rally_results_loop::SCREEN,
        &Entry::default(),
    )?;
    Ok(())
}
