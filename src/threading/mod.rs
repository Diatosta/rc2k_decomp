pub(crate) mod find_thread_slot;
pub mod thread_slot;

use crate::framework::GameHook;

pub static HOOKS: &[&dyn GameHook] = &[&find_thread_slot::HOOK];
