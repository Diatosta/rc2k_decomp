//! `system::packed_local_time` (0x410F99): the local date and time to the minute, packed
//! into one dword: year, 4 bits of month, 5 of day, 5 of hour, 6 of minute.
//!
//! Returns the stamp.
//! `GetLocalTime` fills the game's `SystemTime` (0x518988, a local here: nothing else read
//! it). The save files carry this stamp.
//!

pub fn pack(t: &rc2k_core::system::time::LocalTime) -> u32 {
    // Converted: `rc2k_core::system::time::pack`.
    rc2k_core::system::time::pack(t.year, t.month, t.day, t.hour, t.minute)
}

pub fn packed_local_time() -> u32 {
    pack(&crate::os::local_time())
}
