//! Keyboard input.

use crate::framework::GameHook;

pub mod append_key_name;
pub mod key_to_char;
pub mod take_key;
pub mod yes_no_for_key;
pub mod yes_no_key;

pub static HOOKS: &[&dyn GameHook] = &[
    &take_key::HOOK,
    &key_to_char::HOOK,
    &append_key_name::HOOK,
    &yes_no_key::HOOK,
    &yes_no_for_key::HOOK,
];
