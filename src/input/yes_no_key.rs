//! `input::yes_no_key` (0x407D71): says whether a key answers a yes/no question.
//!
//! In:  eax = the key, if `dword_4F7FD8` is 1; otherwise it takes the pending key itself
//! Out: eax = 1 for yes, 0xFFFFFFFF for no, 2 for the third answer, 0 for anything else or
//!      no key; everything else preserved
//! The key becomes a character through `key_to_char` (with the layout `dword_4F07F8` put in
//! place for the call when `byte_4F07D2` is 2), is made lower case with the table at
//! `byte_4F0BC8`, and is compared with the first character of strings 0xC3, 0xC4 and 0xC5 in
//! turn. Each string is built at the start of the shared text buffer without a NUL, so an
//! empty one is "compared" through whatever was there.
//! Two callers: `yes_no_for_key` and `sub_445069`.
//! Refuses a missing keyboard layout (before the key is taken) and what `key_to_char` does.

use crate::errors::BadInput;
use crate::input::key_to_char::{has_layout, key_to_char};
use crate::input::store::{KEYBOARD, KEYBOARD_FILE};
use crate::input::take_key::take_key;
use crate::strings::append_string_by_id::push_string;
use crate::strings::text_from_ids::{TEXT_LEN, text};

// ---- game data ----

/// (string id, answer)
use rc2k_core::input::keyboard::ANSWERS;
pub const YES: u32 = 1;
pub const NO: u32 = u32::MAX;

// ---- Rust version ----

/// The answer a key gives. `eax` is the register as the original has it: the key in its low
/// byte, and 0 as a whole means no key.
pub unsafe fn answer_for_key(eax: u32) -> Result<u32, BadInput> {
    if eax == 0 {
        return Ok(0);
    }
    unsafe {
        let own_layout = KEYBOARD_FILE.mode.get() == 2;
        let layout = KEYBOARD_FILE.layout.get();
        if own_layout {
            KEYBOARD_FILE
                .layout
                .set(KEYBOARD_FILE.layouts[0].dead_keys.addr());
        }
        let character = key_to_char(eax);
        if own_layout {
            KEYBOARD_FILE.layout.set(layout);
        }
        let lower = crate::strings::same_text::lower(character? as u8); // g_lowerCase
        // Converted: `rc2k_core::input::keyboard::answer`; each string built at the text
        // buffer's start without a NUL, as the original does (their lengths checked first).
        let mut strings = Vec::new();
        for (id, _) in ANSWERS {
            let mut string = Vec::new();
            push_string(&mut string, id)?;
            strings.push(string);
        }
        if strings.iter().any(|s| s.len() > TEXT_LEN) {
            return Err(BadInput("a string longer than the text buffer"));
        }
        Ok(rc2k_core::input::keyboard::answer(lower, |id| {
            let n = ANSWERS.iter().position(|&(i, _)| i == id).unwrap_or(0);
            std::ptr::copy_nonoverlapping(strings[n].as_ptr(), text(), strings[n].len());
            *text()
        }))
    }
}

pub unsafe fn yes_no_key(given: u32) -> Result<u32, BadInput> {
    if !has_layout() {
        return Err(BadInput("no keyboard layout"));
    }
    unsafe {
        let eax = if KEYBOARD.yes_no_key_given.get() == 1 {
            given
        } else {
            take_key() as u32
        };
        answer_for_key(eax)
    }
}
