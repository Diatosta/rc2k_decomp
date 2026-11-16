//! Builds the text of `count` consecutive string ids in the shared text buffer and NUL-terminates
//! it. The next id is whatever `append_string_by_id` returns, plus 1. That is the previous id for
//! string-table ids, but a string pointer for the pointer-table branches, so a run only makes sense
//! over string-table ids. A count of 0 wraps and never ends: refused, as is a text longer than the
//! buffer (the callers pass the refusal on).

use crate::errors::BadInput;
use crate::strings::append_string_by_id::{StringOwners, append_string_by_id};

/// The shared text buffer's length (`strings::text_buffer`).
pub const TEXT_LEN: usize = 0x400;

/// The game's text buffer.
pub fn text_at(
    strings: &'static crate::strings::owner::Strings,
) -> crate::strings::text_buffer::TextBuf {
    crate::strings::text_buffer::TextBuf::Text(&strings.text.buffer.text)
}

/// The bytes left in the text buffer, including the NUL
/// (`rc2k_core::strings::table::text_from_ids`).
pub fn text_from_ids(o: StringOwners<'_>, first_id: u32, count: u32) -> Result<Vec<u8>, BadInput> {
    if count == 0 {
        return Err(BadInput(
            "count 0: the original's counter wraps and it never ends",
        ));
    }
    let fault = crate::menu::event_flow::Fault::default();
    let text = rc2k_core::strings::table::text_from_ids(first_id, count, TEXT_LEN, |id, out| {
        fault.ok(append_string_by_id(o, id, &mut |bytes| {
            out.extend_from_slice(bytes);
        }))
    });
    fault.check()?;
    text.ok_or(BadInput(
        "text longer than the text buffer: the original writes past it",
    ))
}
