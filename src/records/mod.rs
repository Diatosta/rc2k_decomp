use crate::framework::GameHook;

mod find_record;
mod record;

pub static HOOKS: &[&dyn GameHook] = &[&find_record::HOOK];
