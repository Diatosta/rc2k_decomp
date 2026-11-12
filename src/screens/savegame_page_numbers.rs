//! `screens::savegame_page_numbers` (0x46D262): the numbers of the saved games' list
//! for the page shown.
//!
//!
//! With n = the page (`pages.save_page`) times 10: the caption (`g_captionText`) is n and n + 9 with a
//! dash between, and the ten texts of four bytes from the page numbers buffer on are n .. n + 9; each number
//! at least two digits (a 0 before one under 10), each text ended by a NUL. In four draw lists (the
//! load and save screens). Refuses a page whose numbers would not fit their four bytes (100 or
//! more). The menus and draw lists run its `entry` (`menu::table_calls`).

use crate::errors::Refused;
use crate::menu::owner::Menus;

// ---- game data ----

/// A number with a 0 before it under 10.
pub fn savegame_page_numbers(
    menus: &Menus,
    text: &crate::screens::text_buffers_store::TextBuffers,
) -> Result<(), Refused> {
    // The logic is in `rc2k_core::screens::captions::savegame_numbers`.
    let page = menus.tables.pages.save_page.get();
    let (caption, numbers) = rc2k_core::screens::captions::savegame_numbers(page)?;
    let buffers = text;
    buffers.caption.write_at(0, &caption, "the caption")?;
    for (n, text) in numbers.iter().enumerate() {
        let mut cell = text.to_vec();
        cell.push(0);
        buffers
            .page_numbers
            .write_at(4 * n, &cell, "a page number")?;
    }
    Ok(())
}
