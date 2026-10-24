//! The eight fields of car block 0 that `drive::share_all_cars` keeps around its work:
//! (block field, keeping place) pairs, in the original's order.

/// (car block 0's field, its keeping place), IDA addresses.
pub const PAIRS: [(usize, usize); 8] = [
    (0x70FA70, 0x64165C),
    (0x70FA90, 0x641660),
    (0x70FA94, 0x641664),
    (0x70FA98, 0x641668),
    (0x70FAA4, 0x64166C),
    (0x70FA9C, 0x641670),
    (0x70FAA0, 0x641674),
    (0x70FADC, 0x641678),
];

/// Copies each pair's `from` into its `to` (addresses), in order.
pub unsafe fn move_pairs(pairs: &[(u32, u32)]) {
    for &(from, to) in pairs {
        unsafe { (to as *mut u32).write_unaligned((from as *const u32).read_unaligned()) };
    }
}
