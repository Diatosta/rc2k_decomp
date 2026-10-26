//! `math::random_below` (0x4018A2): a random number under a limit.
//!
//! In:  eax = the limit
//! Out: eax = `next_random` modulo the limit; everything else preserved
//! 73 callers. A limit of 0 makes the original's `div` trap: refused.
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::errors::BadInput;

// ---- Rust version ----

pub fn random_below(limit: u32) -> Result<u32, BadInput> {
    if limit == 0 {
        return Err(BadInput(
            "a random number under 0: the original divides by it",
        ));
    }
    crate::math::next_random::wait_for_random();
    crate::math::next_random::DRAWS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    crate::math::next_random::with_random(|r| r.below(limit))
        .ok_or(BadInput("unreachable: the limit is not 0"))
}
