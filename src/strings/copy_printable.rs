//! `strings::copy_printable` (0x4017F2): copies a name, leaving out what cannot be shown.
//!
//! Takes the text (0: none), where to write, room, NUL included.
//! At most room - 1 characters are looked at, up to the text's end. One whose low seven
//! bits are under 0x20 is left out; the others are written, in lower case through
//! `g_lowerCase` unless `KEEP_CASE` is 1. A NUL ends what was written. Nine callers.
//! A room of 0 lets the original look at 2^32 - 1 characters: refused.

use crate::errors::BadInput;
use crate::strings::same_text::case_table;
use rc2k_core::strings::bytes::Bytes;

/// 1 = the case is kept (`dword_4E0144` in IDA,
/// 1 at the start). `input::joystick_found` sets it to 0 for a while.
pub static KEEP_CASE: crate::session::Dword = crate::session::Dword::new(1);

/// What is written for the text `text` (as far as the original reads: room - 1 characters
/// or the NUL, or the slice's end), with its NUL.
pub fn printable_from(text: &[u8], room: u32) -> Result<Vec<u8>, BadInput> {
    let looked_at: Vec<u8> = text
        .iter()
        .take((room as usize).saturating_sub(1))
        .take_while(|&&c| c != 0)
        .copied()
        .collect();
    let keep_case = KEEP_CASE.get() == 1;
    case_table()
        .printable(&looked_at, room as usize, keep_case)
        .ok_or(BadInput("no room for a name: the original runs on"))
}

/// What is written for the text `text`, with its NUL (converted:
/// `rc2k_core::strings::case::CaseTable::printable`). Reads as far as the original: room - 1
/// characters or the NUL; a text that ends first is refused.
pub fn printable(text: &impl Bytes, room: u32) -> Result<Vec<u8>, BadInput> {
    let mut looked_at = Vec::new();
    for n in 0..(room as usize).saturating_sub(1) {
        let c = text
            .byte(n)
            .ok_or(BadInput("a name to copy without its end in its block"))?;
        if c == 0 {
            break;
        }
        looked_at.push(c);
    }
    let keep_case = KEEP_CASE.get() == 1;
    case_table()
        .printable(&looked_at, room as usize, keep_case)
        .ok_or(BadInput("no room for a name: the original runs on"))
}
