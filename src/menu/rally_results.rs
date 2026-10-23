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
//! Caller: `sub_473126`. The steps are their Rust versions; the pages through `part`, in
//! the game their Rust versions. Ungated (the pages wait for the player). Refuses what the
//! steps refuse.
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, game_write, rva};

const FLAG_RVA: usize = rva(0x75AADC); // dword_75AADC
const EVENT_MODE_RVA: usize = rva(0x7556D0); // g_eventMode

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn rally_results(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        game_write(FLAG_RVA, 1u32);
        g = crate::menu::standings_a_from_all::standings_a_from_all(&g)?;
        // Converted: `rc2k_core::menu::results_pages::class_pages_only`.
        if !rc2k_core::menu::results_pages::class_pages_only(game_read(EVENT_MODE_RVA)) {
            g = crate::menu::sort_standings_by_total::sort_standings_by_total(&g)?;
            g = crate::menu::clear_standings_a::clear_standings_a(&g)?;
            g = crate::menu::number_all_a::number_all_a(&g)?;
            crate::menu::rally_results::game_part(1, &mut g)?;
        } else {
            crate::menu::rally_results::game_part(2, &mut g)?;
        }
        game_write(FLAG_RVA, 0u32);
    }
    Ok(*entry)
}

/// In the game: the pages' Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        *r = match place {
            1 => {
                use crate::menu::show_overall_rally_results as p;
                p::show_overall_rally_results(r, &mut p::game_part)
            }
            _ => {
                use crate::menu::class_rally_results_pass as p;
                p::class_rally_results_pass(r, &mut p::game_part)
            }
        }?;
    }
    Ok(())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { rally_results(r) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
