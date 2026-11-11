//! `net::edit_key` (0x416E43): a key for the text being edited (`net::begin_edit`), if one
//! is (`edited` not -1): Escape (1) puts the kept text back and ends the editing,
//! Enter (0x1C) ends it, 0x6E clears the text's 64 bytes, backspace (0x0E) drops its last
//! character; any other key is made a character (`input::key_to_char`), which, if it is one
//! (0x20 or more), goes at the text's end (if it has fewer than 0x3F) with a NUL after it,
//! and is taken off again if the text is then wider (`text::measure_text` with the editing's
//! font, signed) than allowed.
//!
//! Takes `key`. Returns whether no text is edited (any more); the original's callers read this
//! as the zero flag of the compare of the edited item with -1. `input::key_to_char` and
//! `text::measure_text` see only the key's byte and the text (the measure's x changes no width
//! it reads).
//! Callers: `net::setting_key`, `net::chat_key`.

use crate::errors::Refused;
use crate::install::owner::Install;
use crate::net::owner::Net;
use crate::pointers::{StoreBytes, copy_between, store_value};
use crate::text::draw_text::GameCanvas;
use crate::text::measure_text::measure_text_at;
use rc2k_core::net::editing::{self, EditKey};
pub fn edit_key(net: &Net, install: &Install, key: u8) -> Result<bool, Refused> {
    // Converted: `rc2k_core::net::editing` (`EditKey`, `last_char`, `takes`,
    // `end_for_char`, `too_wide`).
    if net.edit().edited.get() != editing::NOT_EDITING {
        let text = crate::net::begin_edit::EditedText::edited(net)?.text(net, install);
        let words = StoreBytes(text);
        match EditKey::of(key) {
            EditKey::Escape => {
                copy_between(
                    &install.paths.script_path,
                    0,
                    text,
                    0,
                    editing::TEXT_LEN,
                    "an edited text",
                )?;
                net.edit().edited.set(editing::NOT_EDITING);
            }
            EditKey::Enter => net.edit().edited.set(editing::NOT_EDITING),
            EditKey::Clear => text.store(0, &[0; editing::TEXT_LEN])?,
            EditKey::Backspace => {
                if let Some(last) = editing::last_char(&words)? {
                    store_value(text, last, 0u8)?;
                }
            }
            EditKey::Other => {
                let typed = crate::input::key_to_char::key_to_char(key as u32)? as u8;
                if editing::takes(typed)
                    && let Some(at) = editing::end_for_char(&words)?
                {
                    // The text up to the character's place and the character, measured as a
                    // value; the original writes the character and a NUL, then takes the
                    // character off again (a NUL over it) when the text is too wide.
                    let mut shown = vec![0; at];
                    text.load(0, &mut shown)?;
                    shown.extend_from_slice(&[typed, 0]);
                    let width = {
                        measure_text_at(
                            net.edit().edit_font.get(),
                            &shown,
                            at as i32,
                            &mut GameCanvas,
                        )
                    }?
                    .width;
                    let kept = if editing::too_wide(width, net.edit().edit_width.get()) {
                        [0, 0]
                    } else {
                        [typed, 0]
                    };
                    text.store(at, &kept)?;
                }
            }
        }
    }
    Ok(net.edit().edited.get() == u32::MAX)
}
