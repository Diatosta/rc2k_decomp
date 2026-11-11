//! `install::folder_key` (0x41534E): the folder page's key action (its list's +0x14): with
//! bit 3 of the key's bits set (an edit's key), Enter (1, not while the edit holds 0x10 of the
//! list's first byte) sets the
//! answer to 0 and leaves, Escape (0x1C) sets it to 1 and leaves. Any other key does
//! nothing.
//!
//! Takes the key, its bits, list.
//! Caller `install::run_page` (the folder page).

use crate::menu::list_handle::ListMenu;
use rc2k_core::install::items::FolderKey;

/// `key` (the key in its low byte, its bits in the next); whether it was taken.
pub fn folder_key(
    install: &'static crate::install::owner::Install,
    key: u32,
    list: ListMenu,
) -> bool {
    // Converted: `rc2k_core::install::items::FolderKey`.
    let way = FolderKey::of(key);
    let answer = way.answer(|| list.flags());
    if let Some(answer) = answer {
        install.pages.list_answer.set(answer);
        install.pages.list_leave.set(1);
    }
    match way {
        FolderKey::NotEdited => key == 0,
        FolderKey::Back => answer.is_some(),
        FolderKey::On => true,
        FolderKey::Other => false,
    }
}
