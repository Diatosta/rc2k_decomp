//! `net::edit_key` (0x416E43): a key for the text being edited (`net::begin_edit`), if one
//! is (`dword_52A4B8` not -1): Escape (1) puts the kept text back and ends the editing,
//! Enter (0x1C) ends it, 0x6E clears the text's 64 bytes, backspace (0x0E) drops its last
//! character; any other key is made a character (`input::key_to_char`), which, if it is one
//! (0x20 or more), goes at the text's end (if it has fewer than 0x3F) with a NUL after it,
//! and is taken off again if the text is then wider (`text::measure_text` with the editing's
//! font, signed) than allowed.
//!
//! In:  `key` (al)
//! Out: whether no text is edited (any more): the original's zero flag of `cmp
//!      dword_52A4B8, -1`, which its callers read (`lq -s 0x416E43`: ZF); every register
//!      kept. `input::key_to_char` and `text::measure_text` see only the key's byte and the
//!      text (the measure's x changes no width it reads).
//! Callers: `net::setting_key`, `net::chat_key`.

use crate::errors::{BadInput, Refused};
use crate::net::lobby::movsb;
use crate::net::owner::net;
use crate::pointers::GamePtr;
use crate::text::draw_text::GameCanvas;
use crate::text::measure_text::measure_text_at;
use rc2k_core::net::editing::{self, EditKey};

fn set_byte(at: GamePtr, v: u8) -> Result<(), BadInput> {
    at.write(v, "an edited text")
}

/// The Rust version.
pub fn edit_key(key: u8) -> Result<bool, Refused> {
    let net = net();
    // Converted: `rc2k_core::net::editing` (`EditKey`, `last_char`, `takes`,
    // `end_for_char`, `too_wide`).
    if net.vars.edited.get() != editing::NOT_EDITING {
        let text = crate::net::begin_edit::EditedText::edited()?.place();
        let words = text.bytes("an edited text")?;
        match EditKey::of(key) {
            EditKey::Escape => {
                movsb(
                    text,
                    &crate::install::owner::install().paths.script_path,
                    editing::TEXT_LEN as u32,
                )?;
                net.vars.edited.set(editing::NOT_EDITING);
            }
            EditKey::Enter => net.vars.edited.set(editing::NOT_EDITING),
            EditKey::Clear => text.fill(0, editing::TEXT_LEN, "an edited text")?,
            EditKey::Backspace => {
                if let Some(last) = editing::last_char(&words)? {
                    set_byte(text.offset(last as u32), 0)?;
                }
            }
            EditKey::Other => {
                let typed = crate::input::key_to_char::key_to_char(key as u32)? as u8;
                if editing::takes(typed)
                    && let Some(at) = editing::end_for_char(&words)?
                {
                    let at = at as u32;
                    text.offset(at).write(typed as u16, "an edited text")?;
                    let width = {
                        measure_text_at(
                            net.vars.edit_font.get().game_ptr(),
                            text,
                            at as i32,
                            &mut GameCanvas,
                        )
                    }?
                    .width;
                    if editing::too_wide(width, net.vars.edit_width.get()) {
                        set_byte(text.offset(at), 0)?;
                    }
                }
            }
        }
    }
    Ok(net.vars.edited.get() == u32::MAX)
}
