//! Says whether a key answers a yes/no question: the pending key. The key becomes a character (`key_to_char_for_text`), made lower case, and is
//! compared with the first character of strings 0xC3, 0xC4 and 0xC5 in turn. An empty string is
//! compared with the first byte of the one before it (the first with the shared text buffer's). Refuses a missing keyboard layout (before the key is taken) and what
//! `key_to_char` refuses.

use crate::errors::BadInput;
use crate::input::key_to_char::{has_layout, key_to_char_for_text};
use crate::input::take_key::take_key;
use crate::strings::append_string_by_id::{StringOwners, push_string};
use crate::strings::text_from_ids::TEXT_LEN;

/// (string id, answer)
use rc2k_core::input::keyboard::ANSWERS;
pub const YES: u32 = 1;
pub const NO: u32 = u32::MAX;
pub const THIRD: u32 = 2;

/// The answer a key gives: the key is in the key word's low byte, and 0 as a whole means no
/// key.
pub(super) fn answer_for_key(
    so: StringOwners<'_>,
    input: &crate::input::owner::Input,
    key_word: u32,
) -> Result<u32, BadInput> {
    if key_word == 0 {
        return Ok(0);
    }
    let character = key_to_char_for_text(input, key_word);
    let lower = crate::strings::same_text::lower(character? as u8);
    let mut firsts = Vec::new();
    for (id, _) in ANSWERS {
        let mut string = Vec::new();
        push_string(so, &mut string, id)?;
        if string.len() > TEXT_LEN {
            return Err(BadInput("a string longer than the text buffer"));
        }
        firsts.push(string.first().copied());
    }
    // An empty answer reads what the shared text buffer's first byte held.
    let previous = so.strings.text.buffer.text.get(0);
    Ok(rc2k_core::input::keyboard::answer(lower, previous, |id| {
        let n = ANSWERS.iter().position(|&(i, _)| i == id).unwrap_or(0);
        firsts[n]
    }))
}

pub fn yes_no_key(
    so: StringOwners<'_>,
    input: &crate::input::owner::Input,
) -> Result<u32, BadInput> {
    if !has_layout(input) {
        return Err(BadInput("no keyboard layout"));
    }
    let key_word = take_key(input) as u32;
    answer_for_key(so, input, key_word)
}
