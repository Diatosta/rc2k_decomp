//! `menu::rally_results` (0x4A94EC): the rally's results pages, `state.menus.which_title` 1 while
//! they show (the pages' button "voltar"). Outside event mode 0x13: standings table A from
//! all (`menu::standings_a_from_all`), sorted by total (`menu::sort_standings_by_total`),
//! cleared and numbered (`clear_standings`, `number_all`), the overall
//! page (`menu::show_overall_rally_results`); in it: table A from all and the class pages
//! (`menu::class_rally_results_pass`).
//!
//! Refuses what the steps refuse. The
//! menus and draw lists run its `entry` (`menu::table_calls`).

use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::menu::screen_loop::Entry;
use crate::menu::standings::Which;
use crate::menu::standings_steps::{clear_standings, number_all};

pub fn rally_results(menus: &Menus, entry: &Entry) -> Result<(), Refused> {
    menus.screens.state.menus.which_title.set(true);
    crate::menu::standings_a_from_all::standings_a_from_all(
        crate::strings::append_string_by_id::StringOwners::of(entry.owners),
        menus,
    )?;
    // The logic is in `rc2k_core::menu::results_pages::class_pages_only`.
    if !rc2k_core::menu::results_pages::class_pages_only(menus.play.champ_view.event_mode.get()) {
        crate::menu::sort_standings_by_total::sort_standings_by_total(menus)?;
        clear_standings(Which::A)?;
        number_all(menus, Which::A)?;
        crate::menu::show_overall_rally_results::show_overall_rally_results(menus, entry)?;
    } else {
        crate::menu::class_rally_results_pass::class_rally_results_pass(menus, entry)?;
    }
    menus.screens.state.menus.which_title.set(false);
    Ok(())
}
