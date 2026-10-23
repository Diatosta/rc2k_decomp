//! `screens::draw_setup_slot_names` (0x46F0DD): draws the names of the 30 setup slots of the
//! current page in two columns of 15.
//!
//! In:  nothing
//! Out: every register preserved; carry clear
//! The page is `dword_755244`; a name is 0x20 bytes of `byte_527880`. Columns at x = 233 and
//! 568, rows from y = 68 in steps of 29, font 0x51E8A4. It keeps its position in
//! `g_markerX`/`g_markerY` and its row counter in `dword_7559EC`, which are left at 568, 503
//! and 15. Refuses a page past the third: the table has 100 names.
//! Called by `draw_save_setup_list` and `draw_load_setup_list`.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_read};
use crate::screens::harness::{Left, draw_entry};
use crate::service::load_setup_slot::SLOT_PAGE_RVA;
use crate::service::read_setup_slot::slot_name;
use crate::text::draw_text::Canvas;

// ---- game data ----

// g_markerX
// g_markerY
// dword_7559EC: a work variable, here the row counter

// ---- Rust version ----

pub unsafe fn draw_setup_slot_names(canvas: &mut dyn Canvas) -> Result<(), Refused> {
    // Converted: `rc2k_core::screens::slot_lists::names`.
    let page: u32 = unsafe { game_read(SLOT_PAGE_RVA) };
    rc2k_core::screens::slot_lists::names(
        &mut crate::screens::game_screen::SlotScreen::new(canvas),
        &rc2k_core::screens::slot_lists::SETUPS,
        page,
        |slot| slot_name(slot) as u32,
    )
    .map(|_| ())
}

fn rust(_: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    unsafe { draw_setup_slot_names(canvas) }?;
    Ok(Left::carry(false))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    draw_entry("screens::draw_setup_slot_names", r, &rust)
}
