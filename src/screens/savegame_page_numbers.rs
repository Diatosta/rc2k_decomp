//! The numbers of the saved games' list for the page shown (`pages.save_page`): the caption
//! is "n-n+9" for n the page times 10, and the ten four-byte texts are n to n + 9, each at least
//! two digits. Refuses a page whose numbers would not fit their four bytes (100 or more).

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
