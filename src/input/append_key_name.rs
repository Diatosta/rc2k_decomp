//! A key's name, as the keyboard pages show it.
//!
//! The current layout's table gives one string id per key; a key with an id gets that string,
//! one without gets "0x" and two hexadecimal digits. No NUL is written. Refuses no layout, or a
//! layout without its table of names.

use crate::errors::{BadInput, Refused};
use crate::input::layouts::layouts;
use crate::strings::append_string_by_id::append_string_by_id;
use crate::strings::hex_append::hex_append;

/// The name's bytes and the value the caller reads back.
pub fn append_key_name(
    so: crate::strings::append_string_by_id::StringOwners<'_>,
    key: u32,
) -> Result<(Vec<u8>, u32), Refused> {
    let key = key & 0x7F;
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
            let eax = append_string_by_id(so, id, &mut |bytes| out.extend_from_slice(bytes))?;
            Ok((out, eax))
        }
        rc2k_core::input::keyboard::KeyName::Hex(key) => {
            out.extend_from_slice(b"0x");
            out.extend(hex_append(key, 2)?);
            Ok((out, key))
        }
    }
}
