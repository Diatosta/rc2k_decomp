//! A key for the text being edited (`net::begin_edit`), if any: Escape puts the kept text back
//! and ends the editing, Enter ends it, 0x6E clears the text, backspace drops the last
//! character; any other key becomes a character appended (up to 0x3F) and taken off again if the
//! text is then wider than allowed. Returns how the editing stands: still going, finished with
//! Enter (or nothing was being edited), or cancelled with Escape.

use crate::errors::Refused;
use crate::memory::cells::{StoreBytes, copy_between, store_value};
use crate::net::owner::Net;
use crate::net::store::Editing;
use crate::text::draw_text::GameCanvas;
use crate::text::measure_text::measure_text_at;
use rc2k_core::net::editing::{self, EditKey};
/// How a key left the editing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditOutcome {
    /// The text is still being edited.
    Editing,
    /// Enter ended it, or no text was being edited: the screen takes the text.
    Done,
    /// Escape ended it: the old text is back and the screen takes nothing.
    Cancelled,
}

pub fn edit_key(
    net: &Net,
    edit: &Editing,
    key: u8,
    gfx: &'static crate::gfx::owner::Gfx,
) -> Result<EditOutcome, Refused> {
    // The logic is in `rc2k_core::net::editing` (`EditKey`, `last_char`, `takes`,
    // `end_for_char`, `too_wide`).
    let mut cancelled = false;
    if edit.edited.get() != editing::NOT_EDITING {
        let text = edit
            .edit_text
            .get()
            .ok_or(crate::errors::BadInput("no edited text"))?
            .text(net);
        let words = StoreBytes(text);
        let queued = crate::input::typed_text::take();
        // A character: appended (up to 0x3F) and taken off again when the text is then too wide.
        let append = |typed: u8| -> Result<(), Refused> {
            if editing::takes(typed)
                && let Some(at) = editing::end_for_char(&words)?
            {
                // The text up to the character's place and the character, measured as a
                // value; the character is taken off again (a NUL over it) when the text is
                // then too wide.
                let mut shown = vec![0; at];
                text.load(0, &mut shown)?;
                shown.extend_from_slice(&[typed, 0]);
                let width = {
                    measure_text_at(
                        edit.edit_font.get(),
                        &shown,
                        at as i32,
                        &mut GameCanvas::new(gfx),
                    )
                }?
                .width;
                let kept = if editing::too_wide(width, edit.edit_width.get()) {
                    [0, 0]
                } else {
                    [typed, 0]
                };
                text.store(at, &kept)?;
            }
            Ok(())
        };
        for &c in &queued.chars {
            append(c)?;
        }
        match EditKey::of(key) {
            EditKey::Escape => {
                copy_between(
                    &edit.text_before_edit,
                    0,
                    text,
                    0,
                    editing::TEXT_LEN,
                    "an edited text",
                )?;
                edit.edited.set(editing::NOT_EDITING);
                cancelled = true;
            }
            EditKey::Enter => edit.edited.set(editing::NOT_EDITING),
            EditKey::Clear => text.store(0, &[0; editing::TEXT_LEN])?,
            EditKey::Backspace => {
                if let Some(last) = editing::last_char(&words)? {
                    store_value(text, last, 0u8)?;
                }
            }
            // Printable keys come as typed text, not as keys.
            EditKey::Other => {}
        }
    }
    Ok(if cancelled {
        EditOutcome::Cancelled
    } else if edit.edited.get() == editing::NOT_EDITING {
        EditOutcome::Done
    } else {
        EditOutcome::Editing
    })
}
