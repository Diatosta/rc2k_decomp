//! The state of the menu system's one-line text entry (the setup name, the driver name):
//! the length typed so far, the longest allowed, whether an entry is going on, and the copy
//! of the text as it was before, which Escape brings back.

use crate::framework::{game_addr, game_read, game_write, rva};
use crate::service::harness::Region;

pub const LENGTH_RVA: usize = rva(0x75556D); // dword_75556D: characters typed so far
pub const MAX_LENGTH_RVA: usize = rva(0x755571); // dword_755571
pub const ENTERING_RVA: usize = rva(0x7556A8); // dword_7556A8: 1 while a text is being typed
pub const BACKUP_RVA: usize = rva(0x7558A4); // unk_7558A4: the text before the entry started

pub use rc2k_core::menu::text_entry::{Entry, TEXT_LEN, Text};

pub fn length() -> i32 {
    unsafe { game_read(LENGTH_RVA) }
}

pub fn set_length(length: i32) {
    unsafe { game_write(LENGTH_RVA, length) }
}

pub fn backup() -> *mut u8 {
    game_addr(BACKUP_RVA) as *mut u8
}

pub fn backup_region() -> Region {
    (backup() as u32, TEXT_LEN)
}

/// The entry's state, as the game holds it.
pub fn entry() -> Entry {
    unsafe {
        Entry {
            length: length(),
            entering: game_read(ENTERING_RVA),
            backup: game_read(BACKUP_RVA),
        }
    }
}

/// The entry's state stored back.
pub fn put_entry(e: &Entry) {
    unsafe {
        set_length(e.length);
        game_write(ENTERING_RVA, e.entering);
        game_write(BACKUP_RVA, e.backup);
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
pub unsafe fn field(at: *const u8) -> Text {
    unsafe { (at as *const Text).read_unaligned() }
}

pub unsafe fn put_field(at: *mut u8, text: &Text) {
    unsafe { (at as *mut Text).write_unaligned(*text) }
}

/// Writes a NUL-terminated text to the game's log at level 1.
pub unsafe fn log_l1(text: *const u8, newline: bool) {
    unsafe { crate::logfile::log_write::log_write(text, newline, 1) };
}

const BACKED_UP_RVA: usize = rva(0x4DE1D2); // aBackedUpName: "Backed up name="

/// Starts an entry on the text field `field`, as the saved game and replay screens do it:
/// the text is copied to the backup, "Backed up name=<backup>" goes to the log if `log` is
/// set, the field is cleared, the length is 0 and an entry is going on.
pub unsafe fn begin_logged(at: *mut u8, log: bool) {
    // Converted: `rc2k_core::menu::text_entry::Entry::begin`.
    unsafe {
        let mut text = field(at);
        with_entry(|e| e.begin(&mut text));
        put_field(at, &text);
        if log {
            log_l1(game_addr(BACKED_UP_RVA) as *const u8, false);
            log_l1(backup(), true);
        }
    }
}
