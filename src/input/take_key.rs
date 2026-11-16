//! Takes the pending key: gives it and sets it to 0 in one atomic swap.

use crate::input::owner::Input;
use std::sync::atomic::{AtomicU8, Ordering};

/// The first byte of the key table.
pub fn pending_key(input: &Input) -> &AtomicU8 {
    input.devices.keys_down.atomic(0)
}

/// The pending key, or 0. There is none afterwards.
pub fn take_key(input: &Input) -> u8 {
    pending_key(input).swap(0, Ordering::SeqCst)
}
