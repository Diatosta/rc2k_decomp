//! `math::next_random` (0x4018B4): the game's random numbers.
//!
//! In:  nothing
//! Out: eax = the next number, ZF set when it is 0 (read by `race::nonzero_random`);
//!      everything else preserved
//! One step of the generator on its state `dword_4E0140` (`math::random_step`, 0x4018C8,
//! with 0 mixed in): the low byte is changed by three bits of the number (bit 22, then
//! bit 2 and bit 1 of the byte as it then is, each flipping bit 0), the whole is turned
//! right by one, and what is mixed in is `xor`ed on. 23 callers, and
//! `math::random_below` (73 more).
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use std::sync::atomic::{AtomicU32, Ordering};

// ---- Rust version ----

/// The generator's state: out of the image since 2026-10-24 (phase 5; it was
/// `g_randomState`, 0x4E0140, whose first value it keeps until `seed_random`).
pub static STATE: AtomicU32 = AtomicU32::new(0x9570_1114);

/// The generator loaded from `STATE`, `f` run on it, its state stored back.
pub fn with_random<R>(f: impl FnOnce(&mut rc2k_core::math::random::Random) -> R) -> R {
    let mut random = rc2k_core::math::random::Random {
        state: STATE.load(Ordering::Relaxed),
    };
    let value = f(&mut random);
    STATE.store(random.state, Ordering::Relaxed);
    value
}

/// The thread that owns the generator during a self-test that draws (0: none). Another
/// thread asking for a number meanwhile waits: a thread let through the pause (the log
/// writer, a pause timeout) would put the test's two sides on different numbers.
static OWNER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

fn this_thread() -> u32 {
    crate::sync::thread_id()
}

/// Waits while another thread owns the generator (`own_random`).
pub fn wait_for_random() {
    use std::sync::atomic::Ordering;
    loop {
        let owner = OWNER.load(Ordering::SeqCst);
        if owner == 0 || owner == this_thread() {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}

/// The numbers drawn so far (for tests that compare how many each side drew).
pub static DRAWS: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

pub fn next_random() -> u32 {
    wait_for_random();
    DRAWS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    with_random(rc2k_core::math::random::Random::draw)
}

#[cfg(test)]
mod tests {
    #[test]
    fn state_starts_as_the_games() {
        let d = crate::moved::image_bytes(0x4E0140, 4);
        assert_eq!(
            u32::from_le_bytes(d.try_into().unwrap()),
            super::STATE.load(std::sync::atomic::Ordering::Relaxed)
        );
    }
}
