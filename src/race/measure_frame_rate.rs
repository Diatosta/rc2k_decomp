//! `race::measure_frame_rate`: every 16th call (`frame_rate_count` counting, back to 0 at 0x10)
//! reads a clock in nanoseconds into `frame_now` (its low half); with the time since the last
//! measure (`frame_last`, into `scratch`) above 0 (signed), the frame rate `frame_rate` is
//! `thousand` x `thousand` x `thousand` x `sixteen` / the nanoseconds; the time is kept for the
//! next. (The original counted the processor's cycles with `rdtsc` and needed its clock in MHz,
//! timed for a quarter of a second at every start.)
//!
//! The rate is worked out in plain floats. Fixed ("The frame rate never measured below some 28
//! frames a second" in `wiki/Fixed-bugs.md`): the time from the whole count, not its signed low
//! halves.

use crate::race::owner::Race;
use std::sync::OnceLock;
use std::sync::atomic::Ordering::Relaxed;
use std::time::Instant;

/// Nanoseconds since the first measure.
fn nanos() -> u64 {
    static FIRST: OnceLock<Instant> = OnceLock::new();
    FIRST.get_or_init(Instant::now).elapsed().as_nanos() as u64
}

pub fn measure_frame_rate(race: &Race, scene: &crate::scene::owner::Scene) {
    let shared = &race.shared;
    // Logic in `rc2k_core::race::frame_rate::{count_frame, nanos_since, frame_rate}`.
    use rc2k_core::race::frame_rate::{count_frame, frame_rate, nanos_since};
    let (count, due) = count_frame(shared.run().frame_rate_count);
    shared.run_mut().frame_rate_count = count;
    if !due {
        return;
    }
    let whole = nanos();
    let now = whole as u32;
    shared.switches.borrow_mut().frame_now = now;
    if let Some(time) = nanos_since(whole, race.frame.last_frame_nanos.swap(whole, Relaxed)) {
        scene.draw_work_mut().scratch = time as u32;
        let numbers = race.course.numbers;
        let rate = frame_rate(numbers.thousand.get(), numbers.sixteen.get(), time);
        shared.switches.borrow_mut().frame_rate = rate;
    }
    shared.switches.borrow_mut().frame_last = now;
}
