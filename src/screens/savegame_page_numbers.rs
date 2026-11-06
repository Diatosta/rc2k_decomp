//! `screens::savegame_page_numbers` (0x46D262): the numbers of the saved games' list
//! for the page shown.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! With n = the page (`dword_75523C`) times 10: the caption (`g_captionText`) is n and n + 9 with a
//! dash between, and the ten texts of four bytes from `byte_A8C158` on are n .. n + 9; each number
//! at least two digits (a 0 before one under 10), each text ended by a NUL. In four draw lists (the
//! load and save screens). Refuses a page whose numbers would not fit their four bytes (100 or
//! more). The menus and draw lists run its `entry` (`menu::table_calls`); the registers above are
//! the original's, for reference.

use crate::errors::Refused;
use crate::menu::owner::menus;

// ---- game data ----

// ---- Rust version ----

/// A number with a 0 before it under 10.
pub fn savegame_page_numbers() -> Result<(), Refused> {
    // Converted: `rc2k_core::screens::captions::savegame_numbers`.
    let page = menus().tables.pages.save_page.get();
    let (caption, numbers) = rc2k_core::screens::captions::savegame_numbers(page)?;
    crate::pointers::GamePtr::to(&crate::screens::text_buffers_store::TEXT_BUFFERS.caption)
        .write_bytes(&caption, "the caption")?;
    for (n, text) in numbers.iter().enumerate() {
        let place = crate::pointers::GamePtr::to(
            &crate::screens::text_buffers_store::TEXT_BUFFERS.page_numbers,
        )
        .offset(4 * n as u32);
        place.write_bytes(text, "a page number")?;
        place
            .offset(text.len() as u32)
            .write::<u8>(0, "a page number")?;
    }
    Ok(())
}
