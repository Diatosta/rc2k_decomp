use crate::framework::GameHook;

mod hex_append;
mod maybe_str_append;
mod utils;

pub static HOOKS: &[&dyn GameHook] = &[&maybe_str_append::HOOK, &hex_append::HOOK];
