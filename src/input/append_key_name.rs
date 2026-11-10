//! `input::append_key_name` (0x407F39): a key's name, as the keyboard pages show it.
//!
//! Takes: the key (its low seven bits), the cursor
//!
//! The current keyboard layout (`input::layouts`; `g_keyboardLayout`, 0x4F07D4, in the
//! original) has a table of one string id a key (words); a key with an id gets that string,
//! one without "0x" and two hexadecimal digits. No NUL is written. Eleven callers.
//! Refuses no layout, or a layout without its table of names.

use crate::errors::{BadInput, Refused};
use crate::input::layouts::layouts;
use crate::strings::append_string_by_id::append_string_by_id;
use crate::strings::hex_append::hex_append;

/// The name's bytes and the eax the original leaves.
pub fn append_key_name(key: u32) -> Result<(Vec<u8>, u32), Refused> {
    let key = key & 0x7F;
    // Converted: `rc2k_core::input::keyboard::key_name` (asks for the one entry).
    let id = {
        let layouts = layouts();
        let layout = layouts
            .layout(layouts.current())
            .ok_or(BadInput("no keyboard layout"))?;
        layout.key_name_id(key)?
    };
    let name = rc2k_core::input::keyboard::key_name(key, |_| id);
    let mut out = Vec::new();
    match name {
        rc2k_core::input::keyboard::KeyName::String(id) => {
            let eax = append_string_by_id(id, &mut |bytes| out.extend_from_slice(bytes))?;
            Ok((out, eax))
        }
        rc2k_core::input::keyboard::KeyName::Hex(key) => {
            out.extend_from_slice(b"0x");
            out.extend(hex_append(key, 2)?);
            Ok((out, key))
        }
    }
}
