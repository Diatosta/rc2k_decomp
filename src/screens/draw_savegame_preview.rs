//! `screens::draw_savegame_preview` (0x46D88F): draws the preview of the highlighted saved
//! game on the load-game screen, reading its file when the highlight has moved.
//!
//! the shown saved game slot is the slot the buffer was last filled for and the nothing-to-show flag is 1 if there
//! is nothing to show. A highlighted item above 10 is no slot: the empty panel, and the
//! remembered slot becomes 999. Otherwise the slot is ``pages.save_page` * 10 + item - 1`; if it
//! is not the remembered one it is read with `read_savegame_slot`. Then
//! `draw_savegame_details` or `draw_empty_savegame`.
//! Its address is in the draw lists at 0x758BAE and 0x758BC6.
//!
//! Confirmed by the owner with `blank on screens::draw_savegame_preview` the same day ("worked as
//! intended": the block under the slot list went away). The menus and draw lists run its `entry`
//! (`menu::table_calls`).

use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::saves::read_savegame_slot::read_savegame_slot;
use crate::screens::draw_empty_savegame::draw_empty_savegame;
use crate::screens::draw_savegame_details::draw_savegame_details;
use crate::text::draw_text::Canvas;

// ---- game data ----

/// Returns the carry flag of the panel drawn.
pub fn draw_savegame_preview(
    menus: &Menus,
    saves: &crate::saves::owner::Saves,
    canvas: &mut dyn Canvas,
) -> Result<bool, Refused> {
    let savegame_preview = &saves.savegame_preview;
    let tables = &menus.tables;
    // The logic is in `rc2k_core::screens::savegame_preview::preview` (the slot); the panels
    // are converted in their own functions.
    use rc2k_core::screens::savegame_preview::{NO_SLOT, Preview, preview};
    let page = menus.tables.pages.save_page.get();
    match preview(
        menus.screens.loop_flags.item.get(),
        page,
        savegame_preview.shown_slot.get(),
    ) {
        Preview::NoSlot => {
            savegame_preview.nothing_to_show.set(1u32);
            savegame_preview.shown_slot.set(NO_SLOT);
            return draw_empty_savegame(canvas);
        }
        Preview::Slot { slot, read: true } => {
            // Refused before anything is written: a slot past the list.
            let found = read_savegame_slot(slot)?;
            savegame_preview.shown_slot.set(slot);
            savegame_preview.nothing_to_show.set(!found as u32);
        }
        Preview::Slot { .. } => {}
    }
    if savegame_preview.nothing_to_show.get() == 1 {
        draw_empty_savegame(canvas)
    } else {
        draw_savegame_details(tables, canvas)
    }
}
