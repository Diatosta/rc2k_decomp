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
use crate::framework::{game_read, rva};
use crate::screens::car_caption::put_string;

// ---- game data ----

const PAGE_RVA: usize = rva(0x755998); // dword_755998
const TITLES_RVA: usize = rva(0x759AFA); // dword_759AFA
const TITLE_RVA: usize = rva(0xA8BF54); // byte_A8BF54
const TITLE_LEN: usize = 0x100;
const PAGES: u32 = 4;

// ---- Rust version ----

pub unsafe fn car_page_title() -> Result<(), Refused> {
    unsafe {
        let page: u32 = game_read(PAGE_RVA);
        if page >= PAGES {
            return Err(BadInput("a page past the four").into());
        }
        put_string(
            game_read(TITLES_RVA + page as usize * 4),
            TITLE_RVA,
            TITLE_LEN,
        )
    }
}
