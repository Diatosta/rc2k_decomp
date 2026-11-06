//! `install::edit_key` (0x4148F7): a key while an edit item (kind 4) is being edited.
//! - bit 8 of the key word set: its low byte is the character itself;
//! - Escape (1): the edit ended (record bit 0 off) and the text put back from
//!   `byte_95EAF8` (its most bytes and two); ah gets bit 3;
//! - Enter (0x1C): the edit ended; a text of nothing but spaces and control bytes is cleared
//!   (its most bytes and two) and the record's default text (`[record+2]`, a string id)
//!   put in; ah gets bit 3;
//! - 0x6E: the text cleared; Backspace (0xE): its last byte cut;
//! - the four arrows: nothing;
//! - any other: the key's character (`input::key_to_char`, with `dword_4F07F8` as the
//!   layout while `byte_4F07D2` is 2) added if it is not below a space and there is room
//!   (the most bytes), and cut again if the text is then wider than `[record+6]`
//!   (`text::measure_text` with the font of the record's kind: `byte_51D52D`, `dword_51D4C4`).
//!
//! The key word is the key in its low byte and its bits in the next; what comes back is
//! that word as the caller reads it, and whether the key was a plain edit (not Escape or
//! Enter, the keys that set bit 3). Caller `install::page_key`.

use crate::errors::Refused;
use crate::install::installer::{append_string_by_id, measure_text, rb, wb};
use crate::install::vars::INSTALLER_PATHS;
use crate::install::vars::InstallerPages;
use crate::menu::list_handle::{EditField, ListItem};
use crate::place::Place;
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
fn past_nul(text: Place) -> u32 {
    let mut n = 0;
    loop {
        let b = rb(text.add(n));
        n = n.wrapping_add(1);
        if b < 1 {
            return n;
        }
    }
}

fn clear(text: Place, most: u32) {
    for n in 0..most + 2 {
        wb(text.add(n), 0);
    }
}

/// `key`: the key and its bits (see above). Returns the key word as the caller reads it
/// and whether the key was a plain edit: true, but for Escape and Enter (bit 3 set).
pub fn edit_key(key: u32, item: ListItem) -> Result<(u32, bool), Refused> {
    let text = item.text();
    let record = item.edit_field();
    let most = crate::install::folders::field_most(text, record.most()) as u32;
    let with_bit_3 = (key | 0x800, false);
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
                let backup = &INSTALLER_PATHS.script_path;
                for n in 0..most + 2 {
                    wb(text.add(n), rb(Place::from(backup).add(n)));
                }
                return Ok(with_bit_3);
            }
            EditKey::Enter => {
                record.set_flags(record.flags() & !EDITING);
                let words = text.game_ptr().bytes("an edited text")?;
                let (blank, _) = edits::blank(&words)?;
                if blank {
                    clear(text, most);
                    // The folder's default by the system's rules (`install::folders`).
                    use crate::install::folders::{Folders, Native};
                    match <Native as Folders>::first_folder() {
                        Some(first) if crate::install::folders::is_folder_field(text) => {
                            crate::install::run_installer::put_typed(&first);
                        }
                        _ => {
                            append_string_by_id(record.default_text() as u32, text);
                        }
                    }
                }
                return Ok(with_bit_3);
            }
            EditKey::Clear => {
                clear(text, most);
                return Ok((key, true));
            }
            EditKey::Backspace => {
                backspace();
                return Ok((key, true));
            }
            EditKey::Arrow => return Ok((key, true)),
            EditKey::Other => {
                let out = crate::input::key_to_char::key_to_char_for_text(key);
                out? as u8
            }
        }
    };
    if ch < 0x20 {
        return Ok((key, true));
    }
    let end = past_nul(text);
    let length = end.wrapping_sub(1) as usize;
    if !edits::takes(ch, length, most as u8) {
        return Ok((key, true));
    }
    wb(text.add(end.wrapping_sub(1)), ch);
    wb(text.add(end), 0);
    // The measure takes the text's last place as its limit.
    let (width, _) = measure_text(
        edit_font(record)?,
        text,
        most.wrapping_add(text.game()).wrapping_add(1),
    )?;
    let widest = crate::install::folders::field_width(text, record.width());
    if edits::too_wide(width, widest) {
        backspace();
    }
    Ok((widest as u32, true))
}
