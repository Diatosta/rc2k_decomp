//! `menu::backup_text` (0x46D5CC): copies the 0x1F bytes of a text field to the text entry's
//! backup, before the field is edited.
//!
//! In:  edi = the text field
//! Out: every register preserved
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::menu::text_entry::{field, with_entry};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

// ---- Rust version ----

pub unsafe fn backup_text(at: *const u8) {
    // Converted: `rc2k_core::menu::text_entry::Entry::back_up`.
    let text = unsafe { field(at) };
    with_entry(|e| e.back_up(&text));
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { backup_text(r.edi as *const u8) };
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
