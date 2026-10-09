use crate::framework::GameHook;

pub mod find_record;
pub mod record;

pub static HOOKS: &[&dyn GameHook] = &[&find_record::HOOK];
