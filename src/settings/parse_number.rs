//! `settings::parse_number` (0x411623): reads a number out of a settings text.
//!
//! In:  eax = where it starts (0 stays 0)
//! Out: eax = the number; everything else preserved
//! An optional `+` or `-`; then hexadecimal after `$` or `0x` (digits, A .. F, a .. f),
//! decimal otherwise; up to the first character that is no digit. No overflow check: the
//! number wraps. Eight callers.
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::bridge::{Raw, read_through};

// ---- Rust version ----

pub unsafe fn parse_number(text: *const u8) -> u32 {
    if text.is_null() {
        return 0;
    }
    // Converted: `rc2k_core::files::settings::number`.
    let words = unsafe { Raw::new(text) };
    read_through(rc2k_core::files::settings::number(&words, 0))
}
