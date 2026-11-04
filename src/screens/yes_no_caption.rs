//! `screens::yes_no_caption` (0x46FA5A): the "leave?" question's button shows the
//! answer.
//!
//! In:  nothing
//! Out: nothing (the original left eax and ebx = after the NUL; no caller reads them)
//!
//! String 0x56B + `g_yesNoAnswer` (0x56B "não", 0x56C "sim", by the language file) and a NUL into
//! `g_captionText`, from which `screens::draw_highlighted_caption` draws the button. In the
//! question screen's draw list (0x758C9E). IDA has not made it a function. Refuses a caption longer
//! than its place. The menus and draw lists run its `entry` (`menu::table_calls`); the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::menu::pages_store::MENU_PAGES;
use crate::strings::append_string_by_id::append_string_by_id;

// ---- game data ----

const CAPTION_LEN: usize = 0x100;
const FIRST: u32 = 0x56B;

// ---- Rust version ----

/// Puts the answer's caption in the caption buffer.
pub fn yes_no_caption() -> Result<(), Refused> {
    let id = MENU_PAGES.yes_no_answer.get().wrapping_add(FIRST);
    let mut text = Vec::new();
    append_string_by_id(id, &mut |bytes| text.extend_from_slice(bytes))?;
    // Converted: `rc2k_core::screens::captions::fit`.
    let text = rc2k_core::screens::captions::fit(text, CAPTION_LEN)?;
    let caption =
        crate::pointers::GamePtr::to(&crate::screens::text_buffers_store::TEXT_BUFFERS.caption);
    caption.write_bytes(&text, "the caption")?;
    Ok(())
}

/// The Rust version the typed tables run (what it leaves in registers is read by
/// none of its callers).
pub fn run() -> Result<(), Refused> {
    yes_no_caption()?;
    Ok(())
}
