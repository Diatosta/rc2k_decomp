use crate::framework::GameHook;

pub mod find_thread_slot;
pub mod request_quit;
pub mod start_slot_thread;
pub mod thread_slot;

pub static HOOKS: &[&dyn GameHook] = &[
    &find_thread_slot::HOOK,
    &start_slot_thread::HOOK,
    &request_quit::HOOK,
];
