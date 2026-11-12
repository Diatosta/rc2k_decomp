//! `strings::append_string_by_id` (0x401FA3): appends the text for a string id.
//!
//! Takes the id and the cursor. Returns the new cursor and the string pointer (pointer-table
//! branches) or the id itself (string-table branch).
//! Part of the original lives in a separate chunk at 0x402010.

use crate::errors::BadInput;
use crate::strings::expand_entry::expand_entry;
use crate::strings::string_table::{driver_name_slot, driver_name_text, made_text};
use rc2k_core::strings::table::Source;

/// What the original leaves in its result register for a text kept in a variable: the text's
/// address, which as the next string id (`text_from_ids`) is past every table entry. Any id
/// past the table does the same, nothing being appended for it.
pub const AFTER_VARIABLE_TEXT: u32 = 0x8000_0000;

/// `emit` receives each run of bytes to append. Returns the value the original leaves in its result register.
pub fn append_string_by_id(id: u32, emit: &mut impl FnMut(&[u8])) -> Result<u32, BadInput> {
    append_string_by_id_in(crate::owners::boundary(), id, emit)
}

/// `append_string_by_id` for a caller that holds the owners.
pub fn append_string_by_id_in(
    o: crate::owners::Owners<'static>,
    id: u32,
    emit: &mut impl FnMut(&[u8]),
) -> Result<u32, BadInput> {
    // Converted: `rc2k_core::strings::table::Source`.
    let fault = crate::menu::event_flow::Fault::default();
    let source = Source::of(id, || fault.ok(driver_name_slot(o.strings, id)) != 0);
    fault.check()?;
    let direct = match source {
        Source::Made => Some(made_text(o.race, o.menus, o.install, id)?),
        Source::Pointer => Some(driver_name_text(
            o.strings,
            driver_name_slot(o.strings, id)?,
        )?),
        Source::Table => None,
    };
    match direct {
        Some(text) => {
            emit(&text);
            Ok(AFTER_VARIABLE_TEXT)
        }
        None => {
            expand_entry(id, emit);
            Ok(id)
        }
    }
}

/// Appends the text of string `id` to `out`.
pub fn push_string(out: &mut Vec<u8>, id: u32) -> Result<(), BadInput> {
    append_string_by_id(id, &mut |bytes| out.extend_from_slice(bytes))?;
    Ok(())
}

/// `push_string` for a caller that holds the owners.
pub fn push_string_in(
    o: crate::owners::Owners<'static>,
    out: &mut Vec<u8>,
    id: u32,
) -> Result<(), BadInput> {
    append_string_by_id_in(o, id, &mut |bytes| out.extend_from_slice(bytes))?;
    Ok(())
}
