//! `input::append_key_name` (0x407F39): a key's name, as the keyboard pages show it.
//!
//! In:  eax = the key (its low seven bits), ebx = the cursor
//! Out: ebx moved on; eax as `append_string_by_id` leaves it, or the key's low seven bits;
//!      everything else preserved (ecx is pushed and popped)
//!
//! The current keyboard layout (`g_keyboardLayout`, 0x4F07D4) has at +0x10 a table of one
//! string id a key (words); a key with an id gets that string, one without "0x" and two
//! hexadecimal digits. No NUL is written. Eleven callers.
//! Refuses no layout, or a layout without its table of names.

use crate::errors::{BadInput, Refused};
use crate::input::store::KEYBOARD_FILE;
use crate::pointers::GamePtr;
use crate::strings::append_string_by_id::append_string_by_id;
use crate::strings::hex_append::hex_append;
use std::slice::from_raw_parts;

// ---- Rust version ----

/// The name's bytes and the eax the original leaves.
pub unsafe fn append_key_name(key: u32) -> Result<(Vec<u8>, u32), Refused> {
    let key = key & 0x7F;
    unsafe {
        let layout: u32 = KEYBOARD_FILE.layout.get();
        if layout == 0 {
            return Err(BadInput("no keyboard layout").into());
        }
        let names: u32 = GamePtr::new(layout)
            .offset(0x10)
            .read("a keyboard layout's key names")?;
        if names == 0 {
            return Err(BadInput("a keyboard layout without its key names").into());
        }
        // Converted: `rc2k_core::input::keyboard::key_name` (asks for the one entry).
        let id: u16 = GamePtr::new(names)
            .offset(key * 2)
            .read("a key's name id")?;
        let name = rc2k_core::input::keyboard::key_name(key, |_| id);
        let mut out = Vec::new();
        match name {
            rc2k_core::input::keyboard::KeyName::String(id) => {
                let eax = append_string_by_id(id, &mut |p, n| {
                    out.extend_from_slice(from_raw_parts(p, n))
                })?;
                Ok((out, eax))
            }
            rc2k_core::input::keyboard::KeyName::Hex(key) => {
                out.extend_from_slice(b"0x");
                out.extend(hex_append(key, 2)?);
                Ok((out, key))
            }
        }
    }
}
