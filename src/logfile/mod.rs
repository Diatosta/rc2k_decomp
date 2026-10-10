use crate::framework::GameHook;

pub mod log_write;
pub mod write_line;

pub static HOOKS: &[&dyn GameHook] = &[&write_line::HOOK, &log_write::HOOK];
