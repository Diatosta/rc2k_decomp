//! `menu::championship_standings` (0x4A9583): the championship's standings pages,
//! `dword_75AB34` 1 while they show. Standings table B from all
//! (`menu::standings_b_from_all`); outside event mode 0x13: sorted by figure and by score
//! (`menu::sort_standings_b_by_figure`, `_by_score`), numbered (`menu::number_all_b`), the
//! overall page (`menu::show_championship_standings`); in it: sorted by figure twice and
//! by points (`_by_points`), the class pages (`menu::class_championship_pass`).
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

const FLAG_RVA: usize = rva(0x75AB34); // dword_75AB34
const EVENT_MODE_RVA: usize = rva(0x7556D0); // g_eventMode

/// A call by place: 1 `menu::show_championship_standings`, 2 `menu::class_championship_pass`.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn championship_standings(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        use crate::menu::{sort_standings_b_by_figure as fig, sort_standings_b_by_points as pts};
        // Converted: `rc2k_core::menu::results_pages` (`class_pages_only`, `standings_sorts`).
        use rc2k_core::menu::results_pages::{Sort, class_pages_only, standings_sorts};
        g = crate::menu::standings_b_from_all::standings_b_from_all(&g)?;
        let a8 = class_pages_only(game_read(EVENT_MODE_RVA));
        for sort in standings_sorts(a8) {
            g = match sort {
                Sort::ByFigure => fig::sort_standings_b_by_figure(&g)?,
                Sort::ByScore => {
                    crate::menu::sort_standings_b_by_score::sort_standings_b_by_score(&g)?
                }
                Sort::ByPoints => pts::sort_standings_b_by_points(&g)?,
            };
        }
        if !a8 {
            g = crate::menu::number_all_b::number_all_b(&g)?;
        }
        game_write(FLAG_RVA, 1u32);
        part(if a8 { 2 } else { 1 }, &mut g)?;
        game_write(FLAG_RVA, 0u32);
    }
    Ok(*entry)
}

/// In the game: the pages' Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        *r = match place {
            1 => {
                use crate::menu::show_championship_standings as p;
                p::show_championship_standings(r, &mut p::game_part)
            }
            _ => {
                use crate::menu::class_championship_pass as p;
                p::class_championship_pass(r, &mut p::game_part)
            }
        }?;
    }
    Ok(())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { championship_standings(r, &mut game_part) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
