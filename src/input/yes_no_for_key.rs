//! `input::yes_no_for_key` (0x407E27): `yes_no_key` for a key the caller already has.
//!
//! Takes: the key
//! Sets `yes_no_key_given` to 1 around the call and puts back what it held. One caller,
//! `menu::handle_yes_no`.

use crate::input::key_to_char::has_layout;
use crate::input::yes_no_key::answer_for_key;

pub fn yes_no_for_key(
    so: crate::strings::append_string_by_id::StringOwners<'_>,
    input: &crate::input::owner::Input,
    eax: u32,
) -> Result<u32, crate::errors::BadInput> {
    if !has_layout() {
        return Err(crate::errors::BadInput("no keyboard layout"));
    }
    answer_for_key(so, input, eax)
}
