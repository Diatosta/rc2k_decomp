//! The menu system's state.

use crate::framework::GameHook;

pub mod go_back;

pub static HOOKS: &[&dyn GameHook] = &[&go_back::HOOK];
