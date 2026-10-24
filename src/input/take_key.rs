//! `input::take_key` (0x407F95): takes the pending key: returns `byte_4F6ADC` and sets it to 0,
//! in one `xchg`.
//!
//! In:  nothing
//! Out: al = the key, 0 if there was none (the rest of eax is preserved); ZF = (al == 0)
//! 65 callers. Shadow mode runs the original alone: taking the key twice would lose it. The
//! hook notes the key it hands out, so that callers can be shadow-compared (see
//! `noting_taken_key`). The
//! other flags of the original's `test al, al` (sign, parity) are not reproduced; carry and
//! zero are.

use crate::framework::{game_ptr, rva};
use std::sync::atomic::{AtomicU8, Ordering};

// ---- game data ----

pub const PENDING_KEY_RVA: usize = rva(0x4F6ADC); // byte_4F6ADC

pub fn pending_key() -> &'static AtomicU8 {
    unsafe { AtomicU8::from_ptr(game_ptr(PENDING_KEY_RVA)) }
}

// ---- Rust version ----

/// The pending key, or 0. There is none afterwards.
pub fn take_key() -> u8 {
    pending_key().swap(0, Ordering::SeqCst)
}
