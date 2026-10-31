//! The hook and self-test machinery for functions that only change game variables. Shadow
//! mode cannot simply run such a function twice, so it runs the Rust version, notes what it
//! left, puts the variables back, runs the original and compares.

use std::slice::{from_raw_parts, from_raw_parts_mut};

/// A piece of game memory a function may change: (address, bytes).
pub type Region = (u32, usize);

pub fn capture(regions: &[Region]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for &(addr, len) in regions {
        bytes.extend_from_slice(unsafe {
            from_raw_parts(crate::pointers::host(addr).cast_const(), len)
        });
    }
    bytes
}

pub fn restore(regions: &[Region], mut bytes: &[u8]) {
    for &(addr, len) in regions {
        unsafe { from_raw_parts_mut(crate::pointers::host(addr), len) }
            .copy_from_slice(&bytes[..len]);
        bytes = &bytes[len..];
    }
}
