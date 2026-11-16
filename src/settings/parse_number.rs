//! `settings::parse_number`: reads a number out of a settings text.
//!
//! Returns the number. An optional `+` or `-`; then hexadecimal after `$` or `0x` (digits, A .. F,
//! a .. f), decimal otherwise; up to the first character that is no digit. No overflow check: the
//! number wraps.

pub fn parse_number(text: &(impl rc2k_core::strings::bytes::Bytes + ?Sized)) -> u32 {
    // Converted: `rc2k_core::files::settings::number`.
    let number = rc2k_core::files::settings::number(text, 0).map_err(crate::errors::BadInput::from);
    number.unwrap_or_else(|e| crate::fatal::raise::end("settings::parse_number", e))
}
