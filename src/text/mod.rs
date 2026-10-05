use crate::framework::GameHook;

pub mod reset_line_state;

pub static HOOKS: &[&dyn GameHook] = &[&reset_line_state::HOOK];
