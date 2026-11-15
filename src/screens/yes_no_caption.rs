//! The "leave?" question's button shows the answer: string 0x56B + the answer ("não" /
//! "sim" by the language file) goes to the caption text, from which
//! `screens::draw_highlighted_caption` draws the button. Refuses a caption longer than its place.

use crate::errors::Refused;
use crate::menu::pages_store::MenuPages;
use crate::strings::append_string_by_id::append_string_by_id;

// ---- game data ----

const CAPTION_LEN: usize = 0x100;
const FIRST: u32 = 0x56B;

/// Puts the answer's caption in the caption buffer.
pub fn yes_no_caption(
    so: crate::strings::append_string_by_id::StringOwners<'_>,
    pages: &MenuPages,
    buffers: &crate::screens::text_buffers_store::TextBuffers,
) -> Result<(), Refused> {
    let id = pages.yes_no_answer.get().wrapping_add(FIRST);
    let mut text = Vec::new();
    append_string_by_id(so, id, &mut |bytes| text.extend_from_slice(bytes))?;
    // The logic is in `rc2k_core::screens::captions::fit`.
    let text = rc2k_core::screens::captions::fit(text, CAPTION_LEN)?;
    buffers.caption.write_at(0, &text, "the caption")?;
    Ok(())
}

pub fn run(
    so: crate::strings::append_string_by_id::StringOwners<'_>,
    menus: &crate::menu::owner::Menus,
    buffers: &crate::screens::text_buffers_store::TextBuffers,
) -> Result<(), Refused> {
    yes_no_caption(so, &menus.tables.pages, buffers)?;
    Ok(())
}
