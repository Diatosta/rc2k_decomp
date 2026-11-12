//! `screens::yes_no_caption` (0x46FA5A): the "leave?" question's button shows the
//! answer.
//!
//!
//! String 0x56B + `g_yesNoAnswer` (0x56B "não", 0x56C "sim", by the language file) and a NUL into
//! `g_captionText`, from which `screens::draw_highlighted_caption` draws the button. In the
//! question screen's draw list (0x758C9E). IDA has not made it a function. Refuses a caption longer
//! than its place. The menus and draw lists run its `entry` (`menu::table_calls`).

use crate::errors::Refused;
use crate::menu::owner::Tables;
use crate::strings::append_string_by_id::append_string_by_id;

// ---- game data ----

const CAPTION_LEN: usize = 0x100;
const FIRST: u32 = 0x56B;

/// Puts the answer's caption in the caption buffer.
pub fn yes_no_caption(
    so: crate::strings::append_string_by_id::StringOwners<'_>,
    tables: &Tables,
    buffers: &crate::screens::text_buffers_store::TextBuffers,
) -> Result<(), Refused> {
    let id = tables.pages.yes_no_answer.get().wrapping_add(FIRST);
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
    let tables = &menus.tables;
    yes_no_caption(so, tables, buffers)?;
    Ok(())
}
