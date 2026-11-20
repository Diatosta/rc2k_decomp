//! `menu::show_class_rally_results`: the rally results of a class: `event.shown.class` the class;
//! the
//! first player's entry of that class in standings table A (`menu::first_player_a`), its
//! figure held to 0x51 (event mode 0x13) or 0x33 into `champ_view.rows.count`;
//! `menu::first_page_stage`, `menu::stage_range_caption`; then the page
//! (`menu::class_rally_results_loop`).
//!
//! Refuses what the steps refuse, before anything; raises the search's fatal error.

use crate::errors::Refused;
use crate::menu::champ_store::ChampView;
use crate::menu::event_store::EventState;
use crate::menu::screen_id::ScreenId;
use crate::menu::screen_loop::{Entry, run_screen};
use rc2k_core::menu::results_pages::{Page, held_figure};

pub(super) fn show_class_rally_results(
    champ: &ChampView,
    event: &EventState,
    class: u32,
    entry: &Entry,
    tables: &crate::saves::best_times::BestTables,
) -> Result<(), Refused> {
    event.shown.class.set(class);
    let found = crate::menu::first_player_a::first_player_a(class, tables)?;
    let a8 = champ.event_mode.get() == rc2k_core::menu::results_pages::A8;
    let count = held_figure(Page::ClassRally { a8 }, found.figure);
    champ.rows.count.set(count);
    crate::menu::first_page_stage::first_page_stage(champ);
    crate::menu::stage_range_caption::stage_range_caption(champ, &entry.menus.text_buffers)?;
    run_screen(ScreenId::ClassRallyResults, entry)?;
    Ok(())
}
