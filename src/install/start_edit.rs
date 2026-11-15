//! An edit item (kind 4) begins to be edited: its record's bit 0 is set, its text cut at its most
//! bytes and kept in `script_path` (for Escape, `install::edit_key`). Called by
//! `install::item_enter`, `install::next_edit`.

use crate::memory::list_handle::ListItem;

pub fn start_edit(install: &'static crate::install::owner::Install, item: ListItem) {
    let record = item.edit_field(install);
    let text = item.text(install);
    let most = crate::install::folders::field_most(install, text, record.most());
    let mut words = text
        .read(most as usize + 2, "an edited text")
        .unwrap_or_else(|e| crate::install::installer::refuse(e));
    let flags = rc2k_core::install::edits::start_edit(record.flags(), most, &mut words);
    text.write(&words, "an edited text")
        .unwrap_or_else(|e| crate::install::installer::refuse(e));
    record.set_flags(flags);
    crate::memory::place::BytePlace::from(&install.paths.script_path)
        .write(&words, "the edit's backup")
        .unwrap_or_else(|e| crate::install::installer::refuse(e));
}
