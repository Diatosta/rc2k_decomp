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

use crate::framework::{Regs, game_ptr, rva};
use std::cell::Cell;
use std::sync::atomic::{AtomicU8, Ordering};

// ---- game data ----

pub const PENDING_KEY_RVA: usize = rva(0x4F6ADC); // byte_4F6ADC

pub fn pending_key() -> &'static AtomicU8 {
    unsafe { AtomicU8::from_ptr(game_ptr(PENDING_KEY_RVA)) }
}

// ---- Rust version ----

/// The pending key, or 0. There is none afterwards.
pub fn take_key() -> u8 {
    if let Some(key) = REPLAYED.get() {
        return key;
    }
    pending_key().swap(0, Ordering::SeqCst)
}

// ---- replaying a key, for shadow comparisons of functions that take one ----
//
// The key arrives from another thread at any moment, so a caller of `take_key` cannot be
// compared by running it twice on "the same" pending key. Instead the original runs first,
// the key it was given is noted here, and the Rust version is then run with that key
// replayed instead of the real one.

thread_local! {
    /// The key the hook last handed out on this thread.
    static LAST_TAKEN: Cell<Option<u8>> = const { Cell::new(None) };
    /// While set, `take_key` returns this and leaves the real pending key alone.
    static REPLAYED: Cell<Option<u8>> = const { Cell::new(None) };
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    crate::fatal::raise::raise_test_if_armed(r);
    let key = take_key();
    LAST_TAKEN.set(Some(key));
    r.eax = (r.eax & !0xFF) | key as u32;
    r.set_zero(key == 0);
    r.set_carry(false);
    Ok(())
}
