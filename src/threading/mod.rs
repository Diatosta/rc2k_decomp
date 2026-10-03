mod find_thread_slot;
pub mod thread_slot;

use crate::framework::Hook;

pub static HOOKS: &[Hook] = &[Hook {
    name: "threading::find_thread_slot",
    rva: 0x410CC5 - 0x400000,
    stats: &find_thread_slot::STATS,
    install: find_thread_slot::install,
    set_enabled: find_thread_slot::set_enabled,
    self_test: None,
}];