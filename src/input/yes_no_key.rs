//! `input::yes_no_key` (0x407D71): says whether a key answers a yes/no question.
//!
//! Takes: the key, if `yes_no_key_given` is 1; otherwise it takes the pending key itself
//! The key becomes a character through `key_to_char_for_text` (the keyboard file's first
//! layout in place for the call when the layout mode (`layouts`) is 2), is made lower case with the lower-case
//! table (`g_lowerCase`), and is compared with the first character of strings 0xC3, 0xC4 and 0xC5 in
//! turn. Each string is built at the start of the shared text buffer without a NUL, so an
//! empty one is "compared" through whatever was there.
//! Two callers: `yes_no_for_key` and `drive::quit_question`.
//! Refuses a missing keyboard layout (before the key is taken) and what `key_to_char` does.

use crate::errors::BadInput;
use crate::input::key_to_char::{has_layout, key_to_char_for_text};
use crate::input::take_key::take_key;
use crate::strings::append_string_by_id::{StringOwners, push_string};
use crate::strings::text_from_ids::TEXT_LEN;

// ---- game data ----

/// (string id, answer)
use rc2k_core::input::keyboard::ANSWERS;
pub const YES: u32 = 1;
pub const NO: u32 = u32::MAX;
pub const THIRD: u32 = 2;

/// The answer a key gives. `eax` is the key word: the key in its low
/// byte, and 0 as a whole means no key.
pub fn answer_for_key(
    so: StringOwners<'_>,
    input: &crate::input::owner::Input,
    eax: u32,
) -> Result<u32, BadInput> {
    if eax == 0 {
        return Ok(0);
    }
    let character = key_to_char_for_text(input, eax);
    let lower = crate::strings::same_text::lower(character? as u8); // g_lowerCase
    // Converted: `rc2k_core::input::keyboard::answer`; each string built at the text
    // buffer's start without a NUL, as the original does (their lengths checked first).
    let mut strings = Vec::new();
    for (id, _) in ANSWERS {
        let mut string = Vec::new();
        push_string(so, &mut string, id)?;
        strings.push(string);
    }
    if strings.iter().any(|s| s.len() > TEXT_LEN) {
        return Err(BadInput("a string longer than the text buffer"));
    }
    Ok(rc2k_core::input::keyboard::answer(lower, |id| {
        let n = ANSWERS.iter().position(|&(i, _)| i == id).unwrap_or(0);
        let buffer = &so.strings.text.buffer.text;
        for (k, &b) in strings[n].iter().enumerate() {
            buffer.set(k, b);
        }
        buffer.get(0)
    }))
}

pub fn yes_no_key(
    so: StringOwners<'_>,
    given: u32,
    input: &crate::input::owner::Input,
) -> Result<u32, BadInput> {
    if !has_layout() {
        return Err(BadInput("no keyboard layout"));
    }
    let eax = if input.keyboard.yes_no_key_given.get() == 1 {
        given
    } else {
        take_key() as u32
    };
    answer_for_key(so, input, eax)
}
