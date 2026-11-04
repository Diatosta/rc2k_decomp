//! `strings::append_string_by_id` (0x401FA3): appends the text for a string id.
//!
//! In:  eax = id, ebx = cursor
//! Out: ebx = new cursor; eax = the string pointer (pointer-table branches) or the id itself
//!      (string-table branch)
//! Part of the original lives in a separate chunk at 0x402010.

use crate::errors::BadInput;
use crate::pointers::GamePtr;
use crate::strings::expand_entry::expand_entry;
use crate::strings::string_table::{ptr_high, ptr_low};
use rc2k_core::strings::bytes::Bytes;
use rc2k_core::strings::table::Source;

// ---- Rust version ----

/// `emit` receives each run of bytes to append. Returns the `eax` the original leaves behind.
pub fn append_string_by_id(id: u32, emit: &mut impl FnMut(&[u8])) -> Result<u32, BadInput> {
    // Converted: `rc2k_core::strings::table::Source`.
    let fault = crate::menu::event_flow::Fault::default();
    let source = Source::of(id, || fault.ok(ptr_low(id)) != 0);
    fault.check()?;
    let direct = match source {
        Source::Made => Some(GamePtr::new(ptr_high(id)?)),
        Source::Pointer => Some(crate::settings::read_driver_names::name_at(ptr_low(id)?)),
        Source::Table => None,
    };
    match direct {
        Some(p) => {
            // The text runs to its NUL inside one block of the game's memory.
            let text = p.bytes("a string")?;
            let Some(len) = (0..).map_while(|n| text.byte(n)).position(|b| b == 0) else {
                return Err(BadInput("a string without its end in its block"));
            };
            emit(&p.read_bytes(len, "a string")?);
            Ok(p.addr())
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
