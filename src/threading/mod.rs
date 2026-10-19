use crate::framework::GameHook;

pub mod create_event;
pub mod create_mutex;
pub mod find_thread_slot;
pub mod request_quit;
pub mod set_priority;
pub mod shutdown_thread;
pub mod start_slot_thread;
pub mod start_threads;
pub mod thread_slot;
pub mod wait_for_object;
pub mod wait_for_threads;

pub static HOOKS: &[&dyn GameHook] = &[
    &find_thread_slot::HOOK,
    &start_slot_thread::HOOK,
    &request_quit::HOOK,
    &start_threads::HOOK,
    &create_event::HOOK,
    &create_mutex::HOOK,
    &wait_for_object::HOOK,
    &wait_for_threads::HOOK,
    &shutdown_thread::HOOK,
];
