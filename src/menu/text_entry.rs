//! The state of the menu system's one-line text entry (the setup name, the driver name):
//! the length typed so far, the longest allowed, whether an entry is going on, and the copy
//! of the text as it was before, which Escape brings back.

use crate::framework::{game_addr, game_read, game_write, rva};
use crate::service::harness::Region;

pub const LENGTH_RVA: usize = rva(0x75556D); // dword_75556D: characters typed so far
pub const MAX_LENGTH_RVA: usize = rva(0x755571); // dword_755571
pub const ENTERING_RVA: usize = rva(0x7556A8); // dword_7556A8: 1 while a text is being typed
pub const BACKUP_RVA: usize = rva(0x7558A4); // unk_7558A4: the text before the entry started

/// The bytes the helpers copy and clear: a text field is 0x20 bytes with its NUL.
pub const TEXT_LEN: usize = 0x1F;

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
