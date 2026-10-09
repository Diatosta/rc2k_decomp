use crate::framework::GameHook;

pub mod resolve_path;

pub static HOOKS: &[&dyn GameHook] = &[&resolve_path::HOOK];
