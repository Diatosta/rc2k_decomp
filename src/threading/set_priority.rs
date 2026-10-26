//! `threading::set_priority` (0x410D39): `SetThreadPriority(GetCurrentThread(), eax)`,
//! every register preserved. Not hooked (a Win32 call and nothing else, section 10); the
//! audio thread's Rust version calls this.

pub fn set_priority(priority: i32) {
    crate::sync::set_priority(priority);
}
