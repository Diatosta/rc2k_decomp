//! The local date and time to the minute, packed into one dword: year, 4 bits of month, 5 of
//! day, 5 of hour, 6 of minute. The save files carry this stamp.

pub fn pack(t: &rc2k_core::system::time::LocalTime) -> u32 {
    rc2k_core::system::time::pack(t.year, t.month, t.day, t.hour, t.minute)
}

pub fn packed_local_time() -> u32 {
    pack(&crate::platform::os::local_time())
}
