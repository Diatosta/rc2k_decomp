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

use crate::errors::Refused;
use crate::framework::Regs;
use crate::install::installer::{LIST_ANSWER, LIST_LEAVE, logic, pb, rb};
use crate::race::viewer::regs_entry;
use rc2k_core::install::items::FolderKey;

pub fn folder_key(entry: &Regs) -> Regs {
    let mut r = *entry;
    // Converted: `rc2k_core::install::items::FolderKey`.
    let key = FolderKey::of(r.eax);
    let answer = key.answer(|| rb(r.edi));
    match key {
        FolderKey::NotEdited => {
            // test eax, eax
            let v = r.eax;
            logic(&mut r, v);
        }
        // test byte [edi], 10h
        FolderKey::Back => {
            r.set_zero(answer.is_some());
            r.set_carry(false);
        }
        // cmp al, 1Ch: equal
        FolderKey::On => {
            r.set_zero(true);
            r.set_carry(false);
        }
        // cmp al, 1Ch: not equal
        FolderKey::Other => {
            let v = (r.eax as u8).wrapping_sub(0x1C) as u32;
            logic(&mut r, v);
        }
    }
    if let Some(answer) = answer {
        pb(LIST_ANSWER, answer);
        pb(LIST_LEAVE, 1);
    }
    r
}

fn rust(r: &Regs) -> Result<Regs, Refused> {
    Ok(folder_key(r))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &rust)
}
