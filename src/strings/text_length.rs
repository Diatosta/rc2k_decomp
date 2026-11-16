//! The length of a NUL-terminated text, without the NUL. There is no limit: it runs until it finds
//! a NUL. Refuses a null pointer.

use crate::errors::BadInput;
use rc2k_core::strings::bytes::Bytes;

/// The length of the text at `at` (to its NUL), which must end inside its block.
pub fn nul_at(text: &(impl Bytes + ?Sized)) -> Result<u32, BadInput> {
    (0..)
        .map_while(|n| text.byte(n))
        .position(|b| b == 0)
        .map(|len| len as u32)
        .ok_or(BadInput("a text without its end in its block"))
}
