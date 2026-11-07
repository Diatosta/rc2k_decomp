//! `settings::parse_number` (0x411623): reads a number out of a settings text.
//!
//! In:  eax = where it starts (0 stays 0)
//! Out: eax = the number; everything else preserved
//! An optional `+` or `-`; then hexadecimal after `$` or `0x` (digits, A .. F, a .. f),
//! decimal otherwise; up to the first character that is no digit. No overflow check: the
//! number wraps. Eight callers.
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

// ---- Rust version ----

pub fn parse_number(text: &(impl rc2k_core::strings::bytes::Bytes + ?Sized)) -> u32 {
    // Converted: `rc2k_core::files::settings::number`.
    let number = rc2k_core::files::settings::number(text, 0).map_err(crate::errors::BadInput::from);
    number.unwrap_or_else(|e| crate::fatal::raise::end("settings::parse_number", e))
}
