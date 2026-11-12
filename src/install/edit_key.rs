//! `install::edit_key` (0x4148F7): a key while an edit item (kind 4) is being edited.
//! - bit 8 of the key word set: its low byte is the character itself;
//! - Escape (1): the edit ended (record bit 0 off) and the text put back from
//!   `script_path` (its most bytes and two); the key word gets bit 3;
//! - Enter (0x1C): the edit ended; a text of nothing but spaces and control bytes is cleared
//!   (its most bytes and two) and the record's default text (`[record+2]`, a string id)
//!   put in; the key word gets bit 3;
//! - 0x6E: the text cleared; Backspace (0xE): its last byte cut;
//! - the four arrows: nothing;
//! - any other: the key's character (`input::key_to_char`, with the keyboard file's first layout
//!   while the layout mode is 2) added if it is not below a space and there is room
//!   (the most bytes), and cut again if the text is then wider than `[record+6]`
//!   (`text::measure_text` with the font of the record's kind: `fonts`).
//!
//! The key word is the key in its low byte and its bits in the next. What comes back is
//! an `EditOutcome`: the word as the caller goes on reading it, and whether the key was a
//! plain edit (not Escape or Enter, the keys that end the edit). Caller `install::page_key`.

use crate::errors::Refused;
use crate::install::installer::{
    measure_text, push_string_by_id, rb, store_text, text_with_nul, wb,
};
use crate::install::vars::InstallerPages;
use crate::menu::list_handle::{EditField, ListItem};
use crate::place::BytePlace;
use rc2k_core::install::edits::{self, EDITING, EditKey};

/// The font of an edit record's kind.
pub fn edit_font(
    record: EditField,
) -> Result<crate::gfx::typefaces::FontId, crate::errors::BadInput> {
    let kind = record.font_kind() as u32;
    let font = InstallerPages::kind_font(5 + 8 * kind) as u32;
    crate::install::vars::InstallerPages::font_of(font)
}

/// How many bytes the text takes with its NUL (the offset past the NUL).
fn past_nul(text: BytePlace) -> u32 {
    let mut n = 0;
    loop {
        let b = rb(text.add(n));
        n = n.wrapping_add(1);
        if b < 1 {
            return n;
        }
    }
}

/// Where the text lies in its record, the base of the limit the measure takes (the width a
/// measuring font finds does not depend on it).
fn measure_limit(text: BytePlace) -> Result<u32, crate::errors::BadInput> {
    match text {
        BytePlace::Store(_, at) | BytePlace::In(_, at) => Ok(at as u32),
        // An edit item's text is always in the installer's pages or paths.
        BytePlace::At(_) => Err(crate::errors::BadInput(
            "an edit item's text outside the installer's data",
        )),
    }
}

fn clear(text: BytePlace, most: u32) {
    for n in 0..most + 2 {
        wb(text.add(n), 0);
    }
}

/// What `edit_key` tells its caller.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EditOutcome {
    /// The key word the caller goes on reading: the key itself, with bit 3 of its bits set
    /// when the edit ended; after an edit that took a character, the original leaves the
    /// field's allowed width here (the callers read only the low byte as a key).
    pub key_word: u32,
    /// True for a key that was only an edit, false for Escape and Enter (the edit ended).
    pub plain_edit: bool,
}

/// `key`: the key and its bits (see above).
pub fn edit_key(
    install: &'static crate::install::owner::Install,
    key: u32,
    item: ListItem,
) -> Result<EditOutcome, Refused> {
    let text = item.text();
    let record = item.edit_field();
    let most = crate::install::folders::field_most(install, text, record.most()) as u32;
    let plain = |key_word| EditOutcome {
        key_word,
        plain_edit: true,
    };
    let ended = EditOutcome {
        key_word: key | 0x800,
        plain_edit: false,
    };
    let backspace = || {
        if rb(text) != 0 {
            wb(text.add(past_nul(text).wrapping_sub(2)), 0);
        }
    };
    // Converted: `rc2k_core::install::edits` (`EditKey`, `blank`, `takes`, `too_wide`).
    let ch = if key & 0x100 != 0 {
        key as u8
    } else {
        match EditKey::of(key as u8) {
            EditKey::Escape => {
                record.set_flags(record.flags() & !EDITING);
                let backup = &install.paths.script_path;
                for n in 0..most + 2 {
                    wb(text.add(n), rb(BytePlace::from(backup).add(n)));
                }
                return Ok(ended);
            }
            EditKey::Enter => {
                record.set_flags(record.flags() & !EDITING);
                let words = text.bytes("an edited text")?;
                let (blank, _) = edits::blank(&words)?;
                if blank {
                    clear(text, most);
                    // The folder's default by the system's rules (`install::folders`).
                    use crate::install::folders::{Folders, Native};
                    match <Native as Folders>::first_folder() {
                        Some(first) if crate::install::folders::is_folder_field(install, text) => {
                            crate::install::run_installer::put_typed(install, &first);
                        }
                        _ => {
                            let mut default = Vec::new();
                            push_string_by_id(&mut default, record.default_text() as u32);
                            store_text(text, &default);
                        }
                    }
                }
                return Ok(ended);
            }
            EditKey::Clear => {
                clear(text, most);
                return Ok(plain(key));
            }
            EditKey::Backspace => {
                backspace();
                return Ok(plain(key));
            }
            EditKey::Arrow => return Ok(plain(key)),
            EditKey::Other => {
                let out = crate::input::key_to_char::key_to_char_for_text(key);
                out? as u8
            }
        }
    };
    if ch < 0x20 {
        return Ok(plain(key));
    }
    let end = past_nul(text);
    let length = end.wrapping_sub(1) as usize;
    if !edits::takes(ch, length, most as u8) {
        return Ok(plain(key));
    }
    wb(text.add(end.wrapping_sub(1)), ch);
    wb(text.add(end), 0);
    // The measure takes the text's last place as its limit.
    let (width, _) = measure_text(
        edit_font(record)?,
        &text_with_nul(text),
        most.wrapping_add(measure_limit(text)?).wrapping_add(1),
    )?;
    let widest = crate::install::folders::field_width(install, text, record.width());
    if edits::too_wide(width, widest) {
        backspace();
    }
    Ok(plain(widest as u32))
}
