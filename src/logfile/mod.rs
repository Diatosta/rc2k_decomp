use crate::framework::GameHook;

pub mod write_line;

pub static HOOKS: &[&dyn GameHook] = &[&write_line::HOOK];
