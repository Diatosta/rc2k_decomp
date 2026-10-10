//! Operating system services other than files.

use crate::framework::GameHook;

pub mod packed_local_time;

pub static HOOKS: &[&dyn GameHook] = &[&packed_local_time::HOOK];
