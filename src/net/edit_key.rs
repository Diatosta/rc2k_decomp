//! A key for the text being edited (`net::begin_edit`), if any: Escape puts the kept text back
//! and ends the editing, Enter ends it, 0x6E clears the text, backspace drops the last
//! character; any other key becomes a character appended (up to 0x3F) and taken off again if the
//! text is then wider than allowed. Returns whether no text is being edited (any more).

use crate::cells::{StoreBytes, copy_between, store_value};
use crate::errors::Refused;
use crate::net::owner::Net;
use crate::net::store::Editing;
use crate::text::draw_text::GameCanvas;
use crate::text::measure_text::measure_text_at;
use rc2k_core::net::editing::{self, EditKey};
pub fn edit_key(
    input: &crate::input::owner::Input,
    net: &Net,
    edit: &Editing,
    key: u8,
    gfx: &'static crate::gfx::owner::Gfx,
) -> Result<bool, Refused> {
    // The logic is in `rc2k_core::net::editing` (`EditKey`, `last_char`, `takes`,
    // `end_for_char`, `too_wide`).
    if edit.edited.get() != editing::NOT_EDITING {
        let text = crate::net::begin_edit::EditedText::edited(edit)?.text(net);
        let words = StoreBytes(text);
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
            }
            EditKey::Enter => edit.edited.set(editing::NOT_EDITING),
            EditKey::Clear => text.store(0, &[0; editing::TEXT_LEN])?,
            EditKey::Backspace => {
                if let Some(last) = editing::last_char(&words)? {
                    store_value(text, last, 0u8)?;
                }
            }
            EditKey::Other => {
                let typed = crate::input::key_to_char::key_to_char(input, key as u32)? as u8;
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
            }
        }
    }
    Ok(edit.edited.get() == u32::MAX)
}
