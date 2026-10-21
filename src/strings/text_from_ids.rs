//! `strings::text_from_ids` (0x40229B): builds the text of `count` consecutive string ids in
//! the shared text buffer and NUL-terminates it.
//!
//! In:  eax = first id, ecx = count (at least 1)
//! Out: ebx = the text buffer; everything else preserved
//! The next id is whatever `append_string_by_id` leaves in eax, plus 1. That is the previous id
//! for string-table ids, but a string pointer for the pointer-table branches, so a run only
//! makes sense over string-table ids. A count of 0 wraps and never ends: refused, as is a text
//! longer than the buffer (the callers pass the refusal on).
//! Hook removed 2026-10-21 (conversion phase 4): Rust callers call this directly; the
//! register use above is the original's, for reference.

use crate::errors::BadInput;
use crate::framework::{game_ptr, rva};
use crate::strings::append_string_by_id::append_string_by_id;
use std::slice::from_raw_parts;

// ---- game data ----

const TEXT_RVA: usize = rva(0x93E430); // text: shared text buffer, up to the kerning table
pub const TEXT_LEN: usize = 0x93E830 - 0x93E430;

pub fn text() -> *mut u8 {
    game_ptr(TEXT_RVA)
}

// ---- Rust version ----

/// The bytes the original leaves in the text buffer, including the NUL (converted:
/// `rc2k_core::strings::table::text_from_ids`).
pub unsafe fn text_from_ids(first_id: u32, count: u32) -> Result<Vec<u8>, BadInput> {
    if count == 0 {
        return Err(BadInput(
            "count 0: the original's counter wraps and it never ends",
        ));
    }
    rc2k_core::strings::table::text_from_ids(first_id, count, TEXT_LEN, |id, out| unsafe {
        append_string_by_id(id, &mut |p, n| out.extend_from_slice(from_raw_parts(p, n)))
    })
    .ok_or(BadInput(
        "text longer than the text buffer: the original writes past it",
    ))
}
