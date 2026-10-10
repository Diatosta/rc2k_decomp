use crate::framework::GameHook;

pub mod numbered_path;
pub mod read_file;
pub mod resolve_path;

pub static HOOKS: &[&dyn GameHook] = &[&resolve_path::HOOK, &numbered_path::HOOK, &read_file::HOOK];
