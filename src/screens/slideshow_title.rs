//! The slideshow's title (string 0x33D). Refuses a title longer than 0x100 bytes.

use crate::errors::Refused;
use crate::screens::car_caption::put_string;

const TITLE_LEN: usize = 0x100;
const TITLE: u32 = 0x33D;

pub fn slideshow_title(
    so: crate::strings::append_string_by_id::StringOwners<'_>,
    text: &crate::screens::text_buffers_store::TextBuffers,
) -> Result<(), Refused> {
    put_string(so, TITLE, &text.title, TITLE_LEN)
}
