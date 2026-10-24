//! `net::clear_car_state` (0x463EB6): a car's block made ready for a network race: 51 of
//! its dwords (+0x64 .. +0x80, +0x2B8 .. +0x364) 0, +0x298 -1.
//!
//! In:  `block` (esi: the car's block)
//! Out: nothing (every register and flag kept: only `mov`s)
//! Caller: `net::reset_cars`.

/// The Rust version.
pub fn clear_car_state(block: u32) {
    // Converted: `rc2k_core::net::cars::CLEARED`.
    for (offset, v) in rc2k_core::net::cars::CLEARED {
        unsafe { ((block + offset) as *mut u32).write_unaligned(v) };
    }
}
