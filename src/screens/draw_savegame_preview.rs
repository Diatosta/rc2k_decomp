//! `screens::draw_savegame_preview` (0x46D88F): draws the preview of the highlighted saved
//! game on the load-game screen, reading its file when the highlight has moved.
//!
//! In:  nothing
//! Out: every register preserved; carry as the panel drawn leaves it
//! `dword_755148` is the slot the buffer was last filled for and `dword_75514C` is 1 if there
//! is nothing to show. A highlighted item above 10 is no slot: the empty panel, and the
//! remembered slot becomes 999. Otherwise the slot is `dword_75523C * 10 + item - 1`; if it
//! is not the remembered one it is read with `read_savegame_slot`. Then
//! `draw_savegame_details` or `draw_empty_savegame`.
//! Its address is in the draw lists at 0x758BAE and 0x758BC6.
//!
//! Screen: the load-game screen ("CARREGAR CAMPEONATO"), the part under the ten slots, by the
//! owner's screenshot of 2026-10-10 and 1,455 shadow calls while it was open. Confirmed by the
//! owner with `blank on screens::draw_savegame_preview` the same day ("worked as intended": the
//! block under the slot list went away). Hook removed 2026-10-22 (typed tables): the menus and
//! draw lists run its `entry` (`menu::table_calls`); the registers above are the original's,
//! for reference.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, game_write, rva};
use crate::menu::name_lists::SAVEGAME_PAGE_RVA;
use crate::saves::read_savegame_slot::read_savegame_slot;
use crate::screens::draw_empty_savegame::draw_empty_savegame;
use crate::screens::draw_savegame_details::draw_savegame_details;
use crate::screens::harness::{Left, draw_entry};
use crate::service::state::menu_item;
use crate::text::draw_text::Canvas;

// ---- game data ----

const SHOWN_SLOT_RVA: usize = rva(0x755148); // dword_755148
const NOTHING_TO_SHOW_RVA: usize = rva(0x75514C); // dword_75514C

// ---- Rust version ----

/// Returns the carry flag of the panel drawn.
pub unsafe fn draw_savegame_preview(canvas: &mut dyn Canvas) -> Result<bool, Refused> {
    // Converted: `rc2k_core::screens::savegame_preview::preview` (the slot); the panels
    // are converted in their own hooks.
    use rc2k_core::screens::savegame_preview::{NO_SLOT, Preview, preview};
    unsafe {
        let page = game_read::<u32>(SAVEGAME_PAGE_RVA);
        match preview(menu_item(), page, game_read(SHOWN_SLOT_RVA)) {
            Preview::NoSlot => {
                game_write(NOTHING_TO_SHOW_RVA, 1u32);
                game_write(SHOWN_SLOT_RVA, NO_SLOT);
                return draw_empty_savegame(canvas);
            }
            Preview::Slot { slot, read: true } => {
                // Refused before anything is written: a slot past the list.
                let found = read_savegame_slot(slot)?;
                game_write(SHOWN_SLOT_RVA, slot);
                game_write(NOTHING_TO_SHOW_RVA, !found as u32);
            }
            Preview::Slot { .. } => {}
        }
        if game_read::<u32>(NOTHING_TO_SHOW_RVA) == 1 {
            draw_empty_savegame(canvas)
        } else {
            draw_savegame_details(canvas)
        }
    }
}

fn rust(_: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    Ok(Left::carry(unsafe { draw_savegame_preview(canvas) }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    draw_entry("screens::draw_savegame_preview", r, &rust)
}
