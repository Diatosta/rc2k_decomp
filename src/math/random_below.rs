//! `math::random_below` (0x4018A2): a random number under a limit.
//!
//! Takes: the limit
//! Returns: `next_random` modulo the limit
//! 73 callers. A limit of 0 makes the original's `div` trap: refused.

use crate::errors::BadInput;

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
