//! `install::folder_key` (0x41534E): the folder page's key action (its list's +0x14): with
//! ah bit 3 (an edit's key), Enter (1, not while the edit holds 0x10 of `[edi]`) sets the
//! answer to 0 and leaves, Escape (0x1C) sets it to 1 and leaves. Any other key does
//! nothing.
//!
//! In:  al the key, ah its bits, edi the list
//! Out: every register kept; the flags of its last test (no caller reads them read back)
//! Caller `install::run_page` (the folder page). Shadow mode compares. Self-test: lists of
//! the test's against the original, every key and both bit-3 states.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::install::installer::{LIST_ANSWER, LIST_LEAVE, pb, rb};
use rc2k_core::install::items::FolderKey;

/// `key` as eax (al the key, ah its bits); whether it was taken (the original's zero).
pub fn folder_key(key: u32, list: u32) -> bool {
    // Converted: `rc2k_core::install::items::FolderKey`.
    let way = FolderKey::of(key);
    let answer = way.answer(|| rb(list));
    if let Some(answer) = answer {
        pb(LIST_ANSWER, answer);
        pb(LIST_LEAVE, 1);
    }
    match way {
        // test eax, eax
        FolderKey::NotEdited => key == 0,
        // test byte ptr [edi], 10h
        FolderKey::Back => answer.is_some(),
        // cmp al, 1Ch
        FolderKey::On => true,
        FolderKey::Other => false,
    }
}
