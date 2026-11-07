//! `input::yes_no_for_key` (0x407E27): `yes_no_key` for a key the caller already has.
//!
//! Takes: the key
//! Out: eax = 1 for yes, 0xFFFFFFFF for no, 2 for the third answer, else 0; everything else
//!      preserved
//! Sets `dword_4F7FD8` to 1 around the call and puts back what it held. One caller,
//! `menu::handle_yes_no`.

use crate::input::key_to_char::has_layout;
use crate::input::yes_no_key::answer_for_key;

// ---- Rust version ----

pub fn yes_no_for_key(eax: u32) -> Result<u32, crate::errors::BadInput> {
    if !has_layout() {
        return Err(crate::errors::BadInput("no keyboard layout"));
    }
    answer_for_key(eax)
}
