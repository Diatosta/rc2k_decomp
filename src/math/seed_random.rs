//! `math::seed_random` (0x401864): how the random numbers start.
//!
//! In:  nothing
//! Out: every register preserved
//! One step of the generator on the start's time (`dword_5183A4`: hour, minute, second,
//! hundredths) with its date mixed in (`dword_5183A0`: day, month, year); unless that is
//! 0, it is turned left by the day of the week (`byte_51839F`) plus 8 and becomes the
//! state. One caller, `start`, right after the time is taken.
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

// ---- game data ----

// ---- Rust version ----

pub unsafe fn seed_random() {
    crate::math::next_random::wait_for_random();
    use crate::bridge::Global;
    static TIME: Global<u32> = Global::at(0x51_83A4);
    static DATE: Global<u32> = Global::at(0x51_83A0);
    static WEEKDAY: Global<u8> = Global::at(0x51_839F);
    let clock = rc2k_core::math::random::StartClock {
        time: TIME.get(),
        date: DATE.get(),
        weekday: WEEKDAY.get(),
    };
    crate::math::next_random::with_random(|r| r.seed(clock));
}
