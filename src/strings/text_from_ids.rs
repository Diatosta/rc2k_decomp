//! `strings::text_from_ids` (0x40229B): builds the text of `count` consecutive string ids in
//! the shared text buffer and NUL-terminates it.
//!
//! Returns the text buffer. The next id is whatever `append_string_by_id` leaves in eax, plus 1.
//! That is the previous id for string-table ids, but a string pointer for the pointer-table
//! branches, so a run only makes sense over string-table ids. A count of 0 wraps and never ends:
//! refused, as is a text longer than the buffer (the callers pass the refusal on).

use crate::errors::BadInput;
use crate::strings::append_string_by_id::append_string_by_id;
use crate::strings::owner::strings;

// ---- game data ----

/// The shared text buffer's length (`strings::text_buffer`).
pub const TEXT_LEN: usize = 0x400;

/// The game's text buffer as a game pointer.
pub fn text_at() -> crate::pointers::GamePtr {
    crate::pointers::GamePtr::to(&strings().text.buffer.text)
}

// ---- Rust version ----

/// The bytes the original leaves in the text buffer, including the NUL (converted:
/// `rc2k_core::strings::table::text_from_ids`).
pub fn text_from_ids(first_id: u32, count: u32) -> Result<Vec<u8>, BadInput> {
    if count == 0 {
        return Err(BadInput(
            "count 0: the original's counter wraps and it never ends",
        ));
    }
    let fault = crate::menu::event_flow::Fault::default();
    let text = rc2k_core::strings::table::text_from_ids(first_id, count, TEXT_LEN, |id, out| {
        fault.ok(append_string_by_id(id, &mut |bytes| {
            out.extend_from_slice(bytes);
        }))
    });
    fault.check()?;
    text.ok_or(BadInput(
        "text longer than the text buffer: the original writes past it",
    ))
}
