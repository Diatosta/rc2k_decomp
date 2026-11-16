//! Appends `digits` uppercase hex digits of a value, most significant first. More than 8 digits is
//! refused (the only callers ask for 2 and 8).

use crate::errors::BadInput;

/// The low `digits` hex digits of `value`, most significant first.
pub fn hex_append(value: u32, digits: u32) -> Result<Vec<u8>, BadInput> {
    rc2k_core::strings::format::hex(value, digits).ok_or(BadInput(
        "more than 8 hex digits: not worked out in the original",
    ))
}
