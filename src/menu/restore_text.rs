//! `menu::restore_text` (0x46D5E3): copies the text entry's backup back into a text field and
//! ends it with a NUL: 0x1F bytes, then a 0 at +0x1F.
//!
//! In:  edi = the text field
//! Out: every register preserved
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::menu::text_entry::{TEXT_LEN, entry as text_entry};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

// ---- Rust version ----

pub unsafe fn restore_text(at: *mut u8) {
    // Converted: `rc2k_core::menu::text_entry::Entry::restored`.
    let text = text_entry().restored();
    unsafe { (at as *mut [u8; TEXT_LEN + 1]).write_unaligned(text) };
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { restore_text(r.edi as *mut u8) };
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
