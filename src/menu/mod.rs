//! The menu system's state.

use crate::framework::GameHook;

pub mod add_text_length;
pub mod backup_text;
pub mod clear_text;
pub mod go_back;
pub mod restore_text;
pub mod text_entry;

pub static HOOKS: &[&dyn GameHook] = &[
    &go_back::HOOK,
    &backup_text::HOOK,
    &restore_text::HOOK,
    &clear_text::HOOK,
    &add_text_length::HOOK,
];
