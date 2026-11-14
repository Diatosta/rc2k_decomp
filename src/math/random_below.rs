//! `math::random_below`: a random number under a limit.
//!
//! `next_random` modulo the limit. A limit of 0 is refused.

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
