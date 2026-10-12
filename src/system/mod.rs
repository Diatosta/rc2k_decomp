//! Operating system services other than files.

use crate::framework::GameHook;

pub mod find_entry;
pub mod packed_local_time;
pub mod sleep;
pub mod tick_count;
pub mod zero_memory;

pub static HOOKS: &[&dyn GameHook] = &[
    &packed_local_time::HOOK,
    &tick_count::HOOK,
    &sleep::HOOK,
    &zero_memory::HOOK,
    &find_entry::HOOK,
];
