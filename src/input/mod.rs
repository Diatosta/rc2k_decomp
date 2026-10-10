//! Keyboard input.

use crate::framework::GameHook;

pub mod take_key;

pub static HOOKS: &[&dyn GameHook] = &[&take_key::HOOK];
