//! The state of the menu system's one-line text entry (the setup name, the driver name):
//! the length typed so far, the longest allowed, whether an entry is going on, and the copy
//! of the text as it was before, which Escape brings back.

use crate::menu::owner::Menus;

pub use rc2k_core::menu::text_entry::{Entry, TEXT_LEN, Text};

pub fn length(menus: &Menus) -> i32 {
    menus.screens.loop_flags.text_length.get() as i32
}

pub fn set_length(menus: &Menus, length: i32) {
    menus.screens.loop_flags.text_length.set(length as u32);
}

/// Where the backup copy of the text is.
pub fn backup(menus: &'static Menus) -> crate::memory::place::BytePlace {
    crate::memory::place::BytePlace::from(&menus.play.slots.names.text_backup)
}

/// The entry's state, as the game holds it.
pub fn entry(menus: &Menus) -> Entry {
    Entry {
        length: menus.screens.loop_flags.text_length.get() as i32,
        entering: menus.screens.entry.typing.is_set(),
        backup: std::array::from_fn(|i| menus.play.slots.names.text_backup.get(i)),
    }
}

/// The entry's state stored back.
pub fn put_entry(menus: &Menus, e: &Entry) {
    set_length(menus, e.length);
    menus.screens.entry.typing.set(e.entering);
    for (i, &byte) in e.backup.iter().enumerate() {
        menus.play.slots.names.text_backup.set(i, byte);
    }
}

/// The entry's state changed by `f` (read, changed, written back whole).
pub fn with_entry<R>(menus: &Menus, f: impl FnOnce(&mut Entry) -> R) -> R {
    let mut e = entry(menus);
    let r = f(&mut e);
    put_entry(menus, &e);
    r
}

/// A text field's text.
pub fn field(
    at: impl Into<crate::memory::place::BytePlace>,
) -> Result<Text, crate::errors::BadInput> {
    let bytes = at.into().read(TEXT_LEN, "a text field")?;
    Ok(std::array::from_fn(|i| bytes[i]))
}

pub fn put_field(
    at: impl Into<crate::memory::place::BytePlace>,
    text: &Text,
) -> Result<(), crate::errors::BadInput> {
    at.into().write(text, "a text field")
}

const BACKED_UP: &std::ffi::CStr = c"Backed up name=";

/// Starts an entry on the text field `field`, as the saved game and replay screens do it:
/// the text is copied to the backup, "Backed up name=<backup>" goes to the log if `log` is
/// set, the field is cleared, the length is 0 and an entry is going on.
pub fn begin_logged(
    menus: &'static Menus,
    at: impl Into<crate::memory::place::BytePlace>,
    log: bool,
) -> Result<(), crate::errors::BadInput> {
    let at = at.into();
    let mut text = field(at)?;
    with_entry(menus, |e| e.begin(&mut text));
    put_field(at, &text)?;
    if log {
        crate::logfile::log_write::log_nul(BACKED_UP, false, 1);
        crate::logfile::log_write::log_at(backup(menus), true, 1);
    }
    Ok(())
}
