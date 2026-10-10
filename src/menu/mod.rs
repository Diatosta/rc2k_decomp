//! The menu system's state.

use crate::framework::GameHook;

pub mod add_text_length;
pub mod backspace_savegame_name;
pub mod backup_text;
pub mod begin_replay_name;
pub mod begin_savegame_name;
pub mod clear_text;
pub mod edit_replay_name;
pub mod edit_savegame_name;
pub mod go_back;
pub mod name_lists;
pub mod restore_text;
pub mod text_entry;

pub static HOOKS: &[&dyn GameHook] = &[
    &go_back::HOOK,
    &backup_text::HOOK,
    &restore_text::HOOK,
    &clear_text::HOOK,
    &add_text_length::HOOK,
    &backspace_savegame_name::HOOK,
    &edit_replay_name::HOOK,
    &edit_savegame_name::HOOK,
    &begin_savegame_name::HOOK,
    &begin_replay_name::HOOK,
];
