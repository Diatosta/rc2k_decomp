//! `screens::draw_replay_slot_names` (0x46F41A): draws the names of the 20 replay slots of
//! the current page in two columns of 10 (the save and load replay pages' drawer).
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`); carry clear (the last `cmp` equal)
//! The page is `dword_755248`; a name is 0x20 bytes of `byte_528934`. Columns at x = 0xC6 and
//! 0x213, rows from y = 0xB5 in steps of 0x1B, font 0x51E8C4. It keeps its position in
//! `g_markerX`/`g_markerY` and its row counter in `dword_7559EC`, left at 0x213, 0x183 and 10.
//! Refuses a page past the fifth: the table has 100 names. Called through the menu tables
//! (0x758BCE, 0x758BD6). Was not a function in IDA (made 2026-10-19: the standalone's
//! replay pages showed no names).
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, rva};
use crate::menu::name_lists::{NAME_LEN, REPLAY_NAMES_RVA, REPLAY_PAGE_RVA};
use crate::screens::harness::{Left, draw_entry};
use crate::text::draw_text::Canvas;

// ---- game data ----

const X_RVA: usize = rva(0x755A18); // g_markerX
const Y_RVA: usize = rva(0x755A1C); // g_markerY
const ROW_RVA: usize = rva(0x7559EC); // dword_7559EC: a work variable, here the row counter

const SLOT_PAGE_RVA: usize = REPLAY_PAGE_RVA;
pub const FONT: usize = 0x51E8C4; // stru_51E8C4
pub const COLUMNS: [i32; 2] = [0xC6, 0x213];
pub const PAGES: u32 = 5;

/// The name of replay slot `slot`.
fn slot_name(slot: u32) -> *const u8 {
    crate::framework::game_ptr::<u8>(REPLAY_NAMES_RVA).wrapping_add(slot as usize * NAME_LEN)
}

// ---- Rust version ----

pub unsafe fn draw_replay_slot_names(canvas: &mut dyn Canvas) -> Result<(), Refused> {
    // Converted: `rc2k_core::screens::slot_lists::names`.
    let page: u32 = unsafe { game_read(SLOT_PAGE_RVA) };
    rc2k_core::screens::slot_lists::names(
        &mut crate::screens::game_screen::SlotScreen::new(canvas),
        &rc2k_core::screens::slot_lists::REPLAYS,
        page,
        |slot| slot_name(slot) as u32,
    )
    .map(|_| ())
}

pub const STATE_RVAS: [usize; 3] = [X_RVA, Y_RVA, ROW_RVA];

fn rust(_: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    unsafe { draw_replay_slot_names(canvas) }?;
    Ok(Left::carry(false))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    draw_entry("screens::draw_replay_slot_names", r, &rust)
}
