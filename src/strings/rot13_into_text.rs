//! `strings::rot13_into_text` (0x4022C4): a text coded by rot13 (letters 13 on, `A` .. `Z`
//! and `a` .. `z` each in their own round; other bytes as they are) into the shared text
//! buffer (`text`, 0x93E430), with its NUL.
//!
//! In:  eax = the text
//! Out: eax = the text buffer; everything else kept (`pusha` .. `popa`; the original's
//!      `mov ah, 'z'` in the loop is lost in the `popa`)
//! Caller: `menu::check_cheat_name` (the fourth player's name, against the coded cheat
//! (every byte value, the round's edges), the buffer filled with 0xAA first and put back.
//! Hook removed 2026-10-21 (conversion phase 4): Rust callers call this directly; the
//! register use above is the original's, for reference.

use crate::errors::BadInput;
use crate::pointers::GamePtr;
use crate::strings::owner::strings;
use crate::strings::text_from_ids::{TEXT_LEN, text_at};
use rc2k_core::strings::bytes::Bytes;

/// One byte coded.
pub fn rot13(b: u8) -> u8 {
    rc2k_core::strings::format::rot13(b)
}

/// The Rust version: the text buffer's address.
pub fn rot13_into_text(from: GamePtr) -> Result<GamePtr, BadInput> {
    let source = from.bytes("a text to code")?;
    let mut coded = Vec::new();
    loop {
        let b = source
            .byte(coded.len())
            .ok_or(BadInput("a text without its end in its block"))?;
        coded.push(rot13(b));
        if b == 0 {
            break;
        }
        if coded.len() >= TEXT_LEN {
            return Err(BadInput("a text longer than the text buffer"));
        }
    }
    for (i, &b) in coded.iter().enumerate() {
        strings().text.buffer.text.set(i, b);
    }
    Ok(text_at())
}
