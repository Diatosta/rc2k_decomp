//! `SetThreadPriority(GetCurrentThread(), priority)`.
//! A Win32 call and nothing else; the audio thread calls this.

pub fn set_priority(priority: i32) {
    crate::sync::set_priority(priority);
}
