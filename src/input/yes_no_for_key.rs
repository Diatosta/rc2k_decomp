//! `yes_no_key` for a key the caller already has.

use crate::input::key_to_char::has_layout;
use crate::input::yes_no_key::answer_for_key;

pub fn yes_no_for_key(
    so: crate::strings::append_string_by_id::StringOwners<'_>,
    input: &crate::input::owner::Input,
    key_word: u32,
) -> Result<u32, crate::errors::BadInput> {
    if !has_layout(input) {
        return Err(crate::errors::BadInput("no keyboard layout"));
    }
    answer_for_key(so, input, key_word)
}
