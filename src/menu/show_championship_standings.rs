//! `menu::show_championship_standings` (0x4A5A47): the championship standings: the first player's entry of a group in standings
//! table B (`menu::first_player_unplaced`), its figure held to 0xAB into `dword_7556B4`;
//! `menu::first_page_stage`, `menu::stage_range_caption`; then the page
//! (`menu::championship_standings_loop`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: `sub_4A0E60`, `sub_4A9583`. The steps are their Rust versions. Refuses what the steps
//! refuse, before anything; raises the search's fatal error as the original.

use crate::errors::Refused;
use crate::menu::champ_store::CHAMP_VIEW;
use crate::menu::screen_loop::{Carried, run_screen};
use crate::menu::standings::*;
use rc2k_core::menu::results_pages::{Page, held_figure};

// dword_7558F4
// g_eventMode

/// The Rust version (see the header). `c` the carried values (the registers the page's
/// screen is run with).
pub unsafe fn show_championship_standings(c: &Carried) -> Result<(), Refused> {
    unsafe {
        let mut c = Carried {
            palette: ANY_GROUP,
            ..*c
        };
        let found = crate::menu::first_player_unplaced::first_player_unplaced(TABLE_B_RVA.get())?;
        found.carry_on(&mut c);
        // Converted: `rc2k_core::menu::results_pages::held_figure`.
        let count = held_figure(Page::ChampionshipStandings, found.figure);
        CHAMP_VIEW.count.set(count);
        crate::menu::first_page_stage::carry_on(
            crate::menu::first_page_stage::first_page_stage(),
            &mut c,
        );
        crate::menu::stage_range_caption::stage_range_caption()?;
        run_screen(&crate::menu::championship_standings_loop::SCREEN, &c)?;
    }
    Ok(())
}
