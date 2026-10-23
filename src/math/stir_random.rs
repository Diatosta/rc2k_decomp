//! `math::stir_random` (0x40188B): stirs a number into the random numbers' state.
//!
//! In:  eax = the number
//! Out: every register preserved
//! One step of the generator on eax with the state mixed in; unless that is 0, it becomes
//! the state. One caller, `start`, after `sub_403131` (which asks about the volume and
//! ends with a `pop eax` that takes the esi it had saved: so what is stirred in is
//! whatever esi was at the process's start, not the volume's number).
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

// ---- Rust version ----

pub unsafe fn stir_random(number: u32) {
    crate::math::next_random::wait_for_random();
    crate::math::next_random::with_random(|r| r.stir(number));
}
