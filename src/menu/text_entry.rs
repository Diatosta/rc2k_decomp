//! The state of the menu system's one-line text entry (the setup name, the driver name):
//! the length typed so far, the longest allowed, whether an entry is going on, and the copy
//! of the text as it was before, which Escape brings back.

use crate::menu::loop_store::LOOP_FLAGS;
use crate::menu::names_store::NAME_BUFFERS;
use crate::menu::option_work_store::OPTION_WORK;

pub use rc2k_core::menu::text_entry::{Entry, TEXT_LEN, Text};

pub fn length() -> i32 {
    LOOP_FLAGS.text_length.get() as i32
}

pub fn set_length(length: i32) {
    LOOP_FLAGS.text_length.set(length as u32)
}

/// Where the backup copy of the text is.
pub fn backup() -> u32 {
    NAME_BUFFERS.text_backup.addr()
}

/// The entry's state, as the game holds it.
pub fn entry() -> Entry {
    Entry {
        length: length(),
        entering: OPTION_WORK.typing.get(),
        backup: std::array::from_fn(|i| NAME_BUFFERS.text_backup.get(i)),
    }
}

/// The entry's state stored back.
pub fn put_entry(e: &Entry) {
    set_length(e.length);
    OPTION_WORK.typing.set(e.entering);
    for (i, &byte) in e.backup.iter().enumerate() {
        NAME_BUFFERS.text_backup.set(i, byte);
    }
}

/// The entry's state changed by `f` (read, changed, written back whole).
pub fn with_entry<R>(f: impl FnOnce(&mut Entry) -> R) -> R {
    let mut e = entry();
    let r = f(&mut e);
    put_entry(&e);
    r
}

/// A text field's text.
pub fn field(at: u32) -> Result<Text, crate::errors::BadInput> {
    crate::pointers::GamePtr::new(at).read::<Text>("a text field")
}

pub fn put_field(at: u32, text: &Text) -> Result<(), crate::errors::BadInput> {
    crate::pointers::GamePtr::new(at).write::<Text>(*text, "a text field")
}

/// Writes a NUL-terminated text to the game's log at level 1.
pub unsafe fn log_l1(text: *const u8, newline: bool) {
    unsafe { crate::logfile::log_write::log_write(text, newline, 1) };
}

const BACKED_UP_RVA: &std::ffi::CStr = c"Backed up name="; // aBackedUpName: "Backed up name="

/// Starts an entry on the text field `field`, as the saved game and replay screens do it:
/// the text is copied to the backup, "Backed up name=<backup>" goes to the log if `log` is
/// set, the field is cleared, the length is 0 and an entry is going on.
pub fn begin_logged(at: u32, log: bool) -> Result<(), crate::errors::BadInput> {
    // Converted: `rc2k_core::menu::text_entry::Entry::begin`.
    let mut text = field(at)?;
    with_entry(|e| e.begin(&mut text));
    put_field(at, &text)?;
    if log {
        unsafe {
            log_l1(BACKED_UP_RVA.as_ptr() as *const u8, false);
            log_l1(crate::pointers::host(backup()) as *const u8, true);
        }
    }
    Ok(())
}
