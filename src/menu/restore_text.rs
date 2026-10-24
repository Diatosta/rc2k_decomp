//! `menu::restore_text` (0x46D5E3): copies the text entry's backup back into a text field and
//! ends it with a NUL: 0x1F bytes, then a 0 at +0x1F.
//!
//! In:  edi = the text field
//! Out: every register preserved

use crate::menu::text_entry::{TEXT_LEN, entry as text_entry};

// ---- Rust version ----

pub unsafe fn restore_text(at: *mut u8) {
    // Converted: `rc2k_core::menu::text_entry::Entry::restored`.
    let text = text_entry().restored();
    unsafe { (at as *mut [u8; TEXT_LEN + 1]).write_unaligned(text) };
}
