use crate::framework::GameHook;

pub mod find_thread_slot;
pub mod thread_slot;

pub static HOOKS: &[&dyn GameHook] = &[&find_thread_slot::HOOK];
