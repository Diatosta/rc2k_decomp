use crate::framework::GameHook;

mod maybe_str_append;

pub static HOOKS: &[&dyn GameHook] = &[&maybe_str_append::HOOK];
