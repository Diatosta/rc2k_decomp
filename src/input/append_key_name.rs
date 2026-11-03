//! `input::append_key_name` (0x407F39): a key's name, as the keyboard pages show it.
//!
//! In:  eax = the key (its low seven bits), ebx = the cursor
//! Out: ebx moved on; eax as `append_string_by_id` leaves it, or the key's low seven bits;
//!      everything else preserved (ecx is pushed and popped)
//!
//! The current keyboard layout (`g_keyboardLayout`, 0x4F07D4: an address in the original, a
//! `LayoutId` handle here) has at +0x10 a table of one
//! string id a key (words); a key with an id gets that string, one without "0x" and two
//! hexadecimal digits. No NUL is written. Eleven callers.
//! Refuses no layout, or a layout without its table of names.

use crate::errors::{BadInput, Refused};
use crate::input::store::{LayoutTable, current_layout, layout_table};
use crate::strings::append_string_by_id::append_string_by_id;
use crate::strings::hex_append::hex_append;
use rc2k_core::strings::bytes::Bytes;

// ---- Rust version ----

/// The name's bytes and the eax the original leaves.
pub fn append_key_name(key: u32) -> Result<(Vec<u8>, u32), Refused> {
    let key = key & 0x7F;
    let layout = current_layout()?.ok_or(BadInput("no keyboard layout"))?;
    let Some(names) = layout_table(layout, LayoutTable::KeyNames)? else {
        return Err(BadInput("a keyboard layout without its key names").into());
    };
    // Converted: `rc2k_core::input::keyboard::key_name` (asks for the one entry).
    let at = key as usize * 2;
    let (Some(low), Some(high)) = (names.byte(at), names.byte(at + 1)) else {
        return Err(BadInput("a key's name id past its table").into());
    };
    let id = u16::from_le_bytes([low, high]);
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
