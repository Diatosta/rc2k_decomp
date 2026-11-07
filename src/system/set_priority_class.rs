//! `system::set_priority_class` (0x410B68): the process's priority class set to eax
//! (`SetPriorityClass(GetCurrentProcess(), eax)`), unless `dword_518388` is not 0.
//!
//! In:  eax = the class (`start`: 0x80, high, at the start; 0x20, normal, at the end)
//! Out: every register preserved (eax, ecx, edx pushed and popped; the rest Win32 keeps)
//! Two callers, both in `start`. The Win32 calls are made from Rust (the same in both
//! programs). Shadow mode runs the original alone (it changes the process). Self-test: the
//! original and the Rust version with the switch 0 and 1, from the class there and to
//! another, the class after each read back; the class and the switch put back.

// ---- game data ----

// ---- Rust version ----

pub fn set_priority_class(class: u32) {
    let system = crate::system::owner::system();
    if system.session.flags.keep_priority.get() == 0 {
        crate::os::set_priority_class(class);
    }
}
