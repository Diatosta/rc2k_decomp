//! `screens::slideshow_title` (0x469A57): the slideshow's title.
//!
//!
//! String 0x33D and a NUL into the title buffer. In two draw lists (the slideshow). Refuses a title
//! longer than 0x100 bytes. The menus and draw lists run its `entry` (`menu::table_calls`).

use crate::errors::Refused;
use crate::screens::car_caption::put_string;

const TITLE_LEN: usize = 0x100;
const TITLE: u32 = 0x33D;

pub fn slideshow_title(
    text: &crate::screens::text_buffers_store::TextBuffers,
) -> Result<(), Refused> {
    put_string(TITLE, &text.title, TITLE_LEN)
}
