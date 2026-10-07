use crate::framework::GameHook;

mod resolve_path;

pub static HOOKS: &[&dyn GameHook] = &[&resolve_path::HOOK];
