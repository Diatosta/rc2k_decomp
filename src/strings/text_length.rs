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

// The original returns from `cmp byte ptr [ecx + ebx], 0` on the NUL: the same compare again
// leaves the same flags.
// ---- Rust version ----

pub unsafe fn text_length(text: *const u8) -> Result<u32, BadInput> {
    if text.is_null() {
        return Err(BadInput("no text"));
    }
    Ok(
        unsafe { std::ffi::CStr::from_ptr(text as *const std::ffi::c_char) }
            .to_bytes()
            .len() as u32,
    )
}
