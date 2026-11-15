//! `menu::show_class_stage_results`: the stage results of a class: `event.shown.class` the class;
//! the
//! first player's entry of that class in standings table A (`menu::first_player_a`), its
//! figure held to 0x33 into `champ_view.count`; `menu::first_page_stage`,
//! `menu::stage_range_caption`; then the page (`menu::class_stage_results_loop`).
//!
//! Refuses what the steps refuse, before anything; raises the search's fatal error.

use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::menu::screen_id::ScreenId;
use crate::menu::screen_loop::{Entry, run_screen};
use rc2k_core::menu::results_pages::{Page, held_figure};

pub fn show_class_stage_results(menus: &Menus, class: u32, entry: &Entry) -> Result<(), Refused> {
    menus.screens.event.shown.class.set(class);
    let found = crate::menu::first_player_a::first_player_a(class)?;
    let count = held_figure(Page::ClassStage, found.figure);
    menus.play.champ_view.count.set(count);
    crate::menu::first_page_stage::first_page_stage(&menus.play);
    crate::menu::stage_range_caption::stage_range_caption(&menus.play)?;
    run_screen(ScreenId::ClassStageResults, entry)?;
    Ok(())
}
