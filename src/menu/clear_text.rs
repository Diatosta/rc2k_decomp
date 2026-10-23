//! `menu::clear_text` (0x467DC9): zeroes the 0x1F bytes of a text field and sets the text
//! entry's length to 0.
//!
//! In:  edi = the text field
//! Out: every register preserved
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::menu::text_entry::{Text, put_field, with_entry};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

// ---- Rust version ----

pub unsafe fn clear_text(at: *mut u8) {
    // Converted: `rc2k_core::menu::text_entry::Entry::clear` (the field is not read).
    let mut text = Text::default();
    with_entry(|e| e.clear(&mut text));
    unsafe { put_field(at, &text) };
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { clear_text(r.edi as *mut u8) };
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
