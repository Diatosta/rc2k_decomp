//! `input::take_key` (0x407F95): takes the pending key: returns `byte_4F6ADC` and sets it to 0,
//! in one `xchg`.
//!
//! 65 callers. The
//! other flags of the original's `test al, al` (sign, parity) are not reproduced; carry and
//! zero are.

use crate::input::owner::input;
use std::sync::atomic::{AtomicU8, Ordering};

/// `byte_4F6ADC`: the first byte of `g_keysDown`.
pub fn pending_key() -> &'static AtomicU8 {
    input().devices.keys_down.atomic(0)
}

/// The pending key, or 0. There is none afterwards.
pub fn take_key() -> u8 {
    pending_key().swap(0, Ordering::SeqCst)
}
