use crate::framework::GameHook;

mod write_line;

pub static HOOKS: &[&dyn GameHook] = &[&write_line::HOOK];
