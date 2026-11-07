//! `strings::text_length` (0x4022B8): the length of a NUL-terminated text.
//!
//! Takes the text. Returns its length without the NUL.
//! 14 call sites (the picture loader, the extension swap, path handling). There is no limit:
//! it runs until it finds a NUL.
//! Refuses a null pointer.

use crate::errors::BadInput;
use rc2k_core::strings::bytes::Bytes;

// The original returns from `cmp byte ptr [ecx + ebx], 0` on the NUL: the same compare again
// leaves the same flags.
// ---- Rust version ----

/// The length of the text at `at` (to its NUL), which must end inside its block.
pub fn nul_at(text: &(impl Bytes + ?Sized)) -> Result<u32, BadInput> {
    (0..)
        .map_while(|n| text.byte(n))
        .position(|b| b == 0)
        .map(|len| len as u32)
        .ok_or(BadInput("a text without its end in its block"))
}
