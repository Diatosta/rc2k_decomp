//! `menu::rally_results` (0x4A94EC): the rally's results pages, `dword_75AADC` 1 while
//! they show (the pages' button "voltar"). Outside event mode 0x13: standings table A from
//! all (`menu::standings_a_from_all`), sorted by total (`menu::sort_standings_by_total`),
//! cleared and numbered (`clear_standings`, `number_all`), the overall
//! page (`menu::show_overall_rally_results`); in it: table A from all and the class pages
//! (`menu::class_rally_results_pass`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `sub_473126`. The steps are their Rust versions. Refuses what the steps refuse. The
//! menus and draw lists run its `entry` (`menu::table_calls`); the registers above are the
//! original's, for reference.

use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::menu::standings::Which;
use crate::menu::standings_steps::{clear_standings, number_all};

/// The Rust version (see the header).
pub fn rally_results(menus: &'static Menus) -> Result<(), Refused> {
    menus.screens.state.menus.which_title.set(1u32);
    crate::menu::standings_a_from_all::standings_a_from_all(menus)?;
    // Converted: `rc2k_core::menu::results_pages::class_pages_only`.
    if !rc2k_core::menu::results_pages::class_pages_only(menus.play.champ_view.event_mode.get()) {
        crate::menu::sort_standings_by_total::sort_standings_by_total(menus)?;
        clear_standings(Which::A)?;
        number_all(Which::A)?;
        crate::menu::show_overall_rally_results::show_overall_rally_results(menus)?;
    } else {
        crate::menu::class_rally_results_pass::class_rally_results_pass(menus)?;
    }
    menus.screens.state.menus.which_title.set(0u32);
    Ok(())
}
