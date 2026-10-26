//! `strings::text_length` (0x4022B8): the length of a NUL-terminated text.
//!
//! In:  ebx = the text
//! Out: ecx = its length without the NUL; the flags of the last compare (zero set, carry
//!      clear); everything else preserved
//! 14 call sites (the picture loader, the extension swap, path handling). There is no limit:
//! it runs until it finds a NUL.
//! Private copy (0x0C bytes, nothing to relocate): the loop comes back to the third byte,
//! inside what the patch overwrites.
//! Refuses a null pointer.

use crate::errors::BadInput;
use crate::pointers::GamePtr;
use rc2k_core::strings::bytes::Bytes;

// The original returns from `cmp byte ptr [ecx + ebx], 0` on the NUL: the same compare again
// leaves the same flags.
// ---- Rust version ----

pub fn text_length(text: *const u8) -> Result<u32, BadInput> {
    if text.is_null() {
        return Err(BadInput("no text"));
    }
    nul_at(crate::pointers::game_address(text), "a text")
}

/// The length of the text at `at` (to its NUL), which must end inside its block.
pub fn nul_at(at: u32, what: &str) -> Result<u32, BadInput> {
    let bytes = GamePtr::new(at).bytes(what)?;
    (0..)
        .map_while(|n| bytes.byte(n))
        .position(|b| b == 0)
        .map(|len| len as u32)
        .ok_or(BadInput("a text without its end in its block"))
}
