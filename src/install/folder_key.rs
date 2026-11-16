//! The folder page's key action: with bit 3 of the key's bits set (an edit's key), Enter sets
//! the answer to 1 and leaves, Escape (unless the list stays) sets it to 0 and leaves. Any other key does nothing.
//! Called by `install::run_page`.

use crate::memory::list_handle::ListMenu;
use rc2k_core::install::items::FolderKey;

/// `key` (the key in its low byte, its bits in the next); whether it was taken.
pub fn folder_key(
    list_state: &crate::menu::list_state::ListState,
    key: u32,
    list: ListMenu,
) -> bool {
    let way = FolderKey::of(key);
    let answer = way.answer(|| list.flags());
    if let Some(answer) = answer {
        list_state.list_answer.set(answer);
        list_state.list_leave.set(1);
    }
    match way {
        FolderKey::NotEdited => key == 0,
        FolderKey::Back => answer.is_some(),
        FolderKey::On => true,
        FolderKey::Other => false,
    }
}
