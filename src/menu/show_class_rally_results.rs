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
use crate::framework::{game_read, game_write, rva};
use crate::menu::screen_loop::{Carried, run_screen};
use crate::menu::standings::*;
use rc2k_core::menu::results_pages::{Page, held_figure};

const CLASS_RVA: usize = rva(0x7558F4); // dword_7558F4
const COUNT_RVA: usize = rva(0x7556B4); // dword_7556B4
const EVENT_MODE_RVA: usize = rva(0x7556D0); // g_eventMode

/// The Rust version (see the header). `c` the carried values (the registers the page's
/// screen is run with).
pub unsafe fn show_class_rally_results(class: u32, c: &Carried) -> Result<(), Refused> {
    unsafe {
        let mut c = Carried {
            palette: class,
            ..*c
        };
        c.arg = game_read(rva(0x7559AC)); // seconds
        game_write(CLASS_RVA, class);
        let found = crate::menu::first_player_a::first_player_a(game_read(TABLE_A_RVA), class)?;
        found.carry_on(&mut c);
        // Converted: `rc2k_core::menu::results_pages::held_figure`.
        let a8 = game_read::<u32>(EVENT_MODE_RVA) == rc2k_core::menu::results_pages::A8;
        let count = held_figure(Page::ClassRally { a8 }, found.figure);
        game_write(COUNT_RVA, count);
        crate::menu::first_page_stage::carry_on(
            crate::menu::first_page_stage::first_page_stage(),
            &mut c,
        );
        crate::menu::stage_range_caption::stage_range_caption();
        run_screen(&crate::menu::class_rally_results_loop::SCREEN, &c)?;
    }
    Ok(())
}
