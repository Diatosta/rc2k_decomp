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
//! Confirmed by the owner with `blank on screens::draw_savegame_preview` the same day ("worked as
//! intended": the block under the slot list went away). The menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::saves::read_savegame_slot::read_savegame_slot;
use crate::screens::draw_empty_savegame::draw_empty_savegame;
use crate::screens::draw_savegame_details::draw_savegame_details;
use crate::service::state::menu_item;
use crate::text::draw_text::Canvas;

// ---- game data ----

// ---- Rust version ----

/// Returns the carry flag of the panel drawn.
pub fn draw_savegame_preview(canvas: &mut dyn Canvas) -> Result<bool, Refused> {
    // Converted: `rc2k_core::screens::savegame_preview::preview` (the slot); the panels
    // are converted in their own functions.
    use rc2k_core::screens::savegame_preview::{NO_SLOT, Preview, preview};
    let page = crate::menu::owner::menus().tables.pages.save_page.get();
    match preview(
        menu_item(),
        page,
        crate::saves::owner::saves()
            .savegame_preview
            .shown_slot
            .get(),
    ) {
        Preview::NoSlot => {
            crate::saves::owner::saves()
                .savegame_preview
                .nothing_to_show
                .set(1u32);
            crate::saves::owner::saves()
                .savegame_preview
                .shown_slot
                .set(NO_SLOT);
            return draw_empty_savegame(canvas);
        }
        Preview::Slot { slot, read: true } => {
            // Refused before anything is written: a slot past the list.
            let found = read_savegame_slot(slot)?;
            crate::saves::owner::saves()
                .savegame_preview
                .shown_slot
                .set(slot);
            crate::saves::owner::saves()
                .savegame_preview
                .nothing_to_show
                .set(!found as u32);
        }
        Preview::Slot { .. } => {}
    }
    if crate::saves::owner::saves()
        .savegame_preview
        .nothing_to_show
        .get()
        == 1
    {
        draw_empty_savegame(canvas)
    } else {
        draw_savegame_details(canvas)
    }
}
