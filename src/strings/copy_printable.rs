//! Copies a name, leaving out what cannot be shown.
//! At most room - 1 characters are looked at, up to the text's end. One whose low seven bits are
//! under 0x20 is left out; the others are written, in lower case through the language's lower-case
//! table unless the case is kept (`printable_from` and `printable` keep it, `printable_lowered`
//! does not). A NUL ends what was written. A room of 0 would look at 2^32 - 1 characters: refused.

use crate::errors::BadInput;
use crate::strings::same_text::case_table;
use rc2k_core::strings::bytes::Bytes;

/// What is written for the text `text` (room - 1 characters or up to the NUL, or the slice's end),
/// with its NUL.
pub fn printable_from(text: &[u8], room: u32) -> Result<Vec<u8>, BadInput> {
    printable_from_with(text, room, true)
}

/// `printable_from` with the letters in lower case (`input::joystick_found`'s controller names).
pub fn printable_lowered(text: &[u8], room: u32) -> Result<Vec<u8>, BadInput> {
    printable_from_with(text, room, false)
}

fn printable_from_with(text: &[u8], room: u32, keep_case: bool) -> Result<Vec<u8>, BadInput> {
    let looked_at: Vec<u8> = text
        .iter()
        .take((room as usize).saturating_sub(1))
        .take_while(|&&c| c != 0)
        .copied()
        .collect();
    case_table()
        .printable(&looked_at, room as usize, keep_case)
        .ok_or(BadInput("no room for a name: the original runs on"))
}

/// What is written for the text `text`, with its NUL
/// (`rc2k_core::strings::case::CaseTable::printable`). Reads room - 1 characters or up to the NUL;
/// A text that ends first is refused.
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
    case_table()
        .printable(&looked_at, room as usize, true)
        .ok_or(BadInput("no room for a name: the original runs on"))
}
