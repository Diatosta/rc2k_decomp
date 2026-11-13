//! The car specification page's title, from the page's title list. Refuses a page past the
//! four and a title longer than 0x100 bytes.

use crate::errors::{BadInput, Refused};
use crate::menu::owner::Menus;
use crate::screens::car_caption::put_string;

// ---- game data ----

const TITLE_LEN: usize = 0x100;
const PAGES: u32 = 4;

pub fn car_page_title(
    so: crate::strings::append_string_by_id::StringOwners<'_>,
    menus: &Menus,
    text: &crate::screens::text_buffers_store::TextBuffers,
) -> Result<(), Refused> {
    let page: u32 = menus.screens.event.car_page.get();
    if page >= PAGES {
        return Err(BadInput("a page past the four").into());
    }
    put_string(
        so,
        menus.tables.lists.car_page_titles[page as usize].get(),
        &text.title,
        TITLE_LEN,
    )
}
