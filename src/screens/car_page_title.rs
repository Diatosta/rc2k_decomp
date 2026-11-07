//! `screens::car_page_title` (0x469A34): the car specification page's title.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! The string `dword_759AFA[dword_755998]` (the page: 0x24C, 0x249, 0x24B, 0x24A) and a NUL into
//! `byte_A8BF54`. In two draw lists. Refuses a page past the four, and a title longer than 0x100
//! bytes. The menus and draw lists run its `entry` (`menu::table_calls`); the registers above are
//! the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::menu::owner::Menus;
use crate::screens::car_caption::put_string;

// ---- game data ----

const TITLE_LEN: usize = 0x100;
const PAGES: u32 = 4;

// ---- Rust version ----

pub fn car_page_title(menus: &Menus) -> Result<(), Refused> {
    let page: u32 = menus.screens.event.car_page.get();
    if page >= PAGES {
        return Err(BadInput("a page past the four").into());
    }
    put_string(
        menus.tables.lists.car_page_titles[page as usize].get(),
        &crate::screens::text_buffers_store::TEXT_BUFFERS.title,
        TITLE_LEN,
    )
}
