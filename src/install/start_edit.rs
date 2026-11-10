//! `install::start_edit` (0x4148D6): an edit item (kind 4) begins to be edited: its record's
//! bit 0 set, its text cut at its most bytes (`[record+1]`: a NUL there) and that and two
//! bytes more kept in `byte_95EAF8` (for Escape, `install::edit_key`).
//!
//! Takes the item.
//! Callers `install::item_enter`, `install::next_edit`.

use crate::menu::list_handle::ListItem;

pub fn start_edit(install: &'static crate::install::owner::Install, item: ListItem) {
    let record = item.edit_field();
    let text = item.text();
    let most = crate::install::folders::field_most(text, record.most());
    // Converted: `rc2k_core::install::edits::start_edit`, on the text's `most` + 2 bytes.
    let at = text.game_ptr();
    let mut words = at
        .read_bytes(most as usize + 2, "an edited text")
        .unwrap_or_else(|e| crate::install::installer::refuse(e));
    let flags = rc2k_core::install::edits::start_edit(record.flags(), most, &mut words);
    at.write_bytes(&words, "an edited text")
        .unwrap_or_else(|e| crate::install::installer::refuse(e));
    record.set_flags(flags);
    crate::place::BytePlace::from(&install.paths.script_path)
        .write(&words, "the edit's backup")
        .unwrap_or_else(|e| crate::install::installer::refuse(e));
}
