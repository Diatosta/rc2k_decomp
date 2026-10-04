use crate::framework::GameHook;

mod hex_append;
mod str_append;
mod utils;
mod str_append_indexed;

pub static HOOKS: &[&dyn GameHook] = &[&str_append::HOOK, &hex_append::HOOK, &str_append_indexed::HOOK];
