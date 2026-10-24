//! `menu::rally_results` (0x4A94EC): the rally's results pages, `dword_75AADC` 1 while
//! they show (the pages' button "voltar"). Outside event mode 0x13: standings table A from
//! all (`menu::standings_a_from_all`), sorted by total (`menu::sort_standings_by_total`),
//! cleared and numbered (`menu::clear_standings_a`, `menu::number_all_a`), the overall
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
use crate::framework::{game_read, game_write, rva};
use crate::menu::screen_loop::Carried;

const FLAG_RVA: usize = rva(0x75AADC); // dword_75AADC
const EVENT_MODE_RVA: usize = rva(0x7556D0); // g_eventMode

/// The Rust version (see the header). `c` the carried values (the registers the pages'
/// screens are run with).
pub unsafe fn rally_results(c: &Carried) -> Result<(), Refused> {
    let mut g = *c;
    unsafe {
        game_write(FLAG_RVA, 1u32);
        crate::menu::standings_a_from_all::standings_a_from_all()?;
        // Converted: `rc2k_core::menu::results_pages::class_pages_only`.
        if !rc2k_core::menu::results_pages::class_pages_only(game_read(EVENT_MODE_RVA)) {
            crate::menu::sort_standings_by_total::sort_standings_by_total()?;
            crate::menu::clear_standings_a::clear_standings_a()?;
            crate::menu::number_all_a::number_all_a()?.carry_on(&mut g);
            crate::menu::show_overall_rally_results::show_overall_rally_results(&g)?;
        } else {
            crate::menu::class_rally_results_pass::class_rally_results_pass(&g)?;
        }
        game_write(FLAG_RVA, 0u32);
    }
    Ok(())
}
