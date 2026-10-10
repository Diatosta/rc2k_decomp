//! Keyboard input.

use crate::framework::GameHook;

pub mod key_to_char;
pub mod take_key;

pub static HOOKS: &[&dyn GameHook] = &[&take_key::HOOK, &key_to_char::HOOK];
