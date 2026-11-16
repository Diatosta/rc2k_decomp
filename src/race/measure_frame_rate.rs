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
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};
use std::time::Instant;

/// The whole count at the last measure (`frame_last` holds its low half).
static LAST: AtomicU64 = AtomicU64::new(0);

/// Nanoseconds since the first measure.
fn nanos() -> u64 {
    static FIRST: OnceLock<Instant> = OnceLock::new();
    FIRST.get_or_init(Instant::now).elapsed().as_nanos() as u64
}

pub fn measure_frame_rate(race: &Race, scene: &crate::scene::owner::Scene) -> Option<u32> {
    let shared = &race.shared;
    // Logic in `rc2k_core::race::frame_rate::{count_frame, nanos_since, frame_rate}`.
    use rc2k_core::race::frame_rate::{count_frame, frame_rate, nanos_since};
    let (count, due) = count_frame(shared.physics_tables.get().frame_rate_count.get());
    shared.physics_tables.get().frame_rate_count.set(count);
    if !due {
        return None;
    }
    let whole = nanos();
    let now = whole as u32;
    shared.switches.get().frame_now.set(now);
    if let Some(time) = nanos_since(whole, LAST.swap(whole, Relaxed)) {
        scene.drawing.scene.get().scratch.set(time as u32);
        let numbers = race.course.numbers.get();
        let rate = frame_rate(numbers.thousand.get(), numbers.sixteen.get(), time);
        shared.switches.get().frame_rate.set(rate);
    }
    shared.switches.get().frame_last.set(now);
    Some(now)
}
