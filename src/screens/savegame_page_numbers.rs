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
use crate::framework::{game_addr, game_read, rva};
use crate::menu::name_lists::SAVEGAME_PAGE_RVA;

// ---- game data ----

const CAPTION_RVA: usize = rva(0xA8BE54); // g_captionText
const NUMBERS_RVA: usize = rva(0xA8C158); // byte_A8C158: ten texts of four bytes

// ---- Rust version ----

/// A number with a 0 before it under 10.
pub fn savegame_page_numbers() -> Result<(), Refused> {
    // Converted: `rc2k_core::screens::captions::savegame_numbers`.
    let page = unsafe { game_read::<u32>(SAVEGAME_PAGE_RVA) };
    let (caption, numbers) = rc2k_core::screens::captions::savegame_numbers(page)?;
    unsafe {
        std::ptr::copy_nonoverlapping(
            caption.as_ptr(),
            game_addr(CAPTION_RVA) as *mut u8,
            caption.len(),
        );
        for (n, text) in numbers.iter().enumerate() {
            let place = game_addr(NUMBERS_RVA) as *mut u8;
            std::ptr::copy_nonoverlapping(text.as_ptr(), place.add(n * 4), text.len());
            *place.add(n * 4 + text.len()) = 0;
        }
    }
    Ok(())
}
