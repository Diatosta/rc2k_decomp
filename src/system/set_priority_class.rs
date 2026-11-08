//! `system::set_priority_class` (0x410B68): the process's priority class set to eax
//! (`SetPriorityClass(GetCurrentProcess(), eax)`), unless `dword_518388` is not 0.
//!
//! Two callers, both in `start`. The Win32 calls are made from Rust.

// ---- game data ----

// ---- Rust version ----

pub fn set_priority_class(class: u32) {
    let system = crate::system::owner::system();
    if system.session.flags.keep_priority.get() == 0 {
        crate::os::set_priority_class(class);
    }
}
