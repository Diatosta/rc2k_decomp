//! `race::measure_frame_rate`: every 16th call (`frame_rate_count` counting, back to 0 at 0x10)
//! reads the time-stamp counter (`rdtsc`) into `frame_now`; with the cycles since the last time
//! (`frame_last`, into `scratch`) above 0 (signed), the frame rate `frame_rate` is the CPU's
//! clock `Machine.cpu_mhz` x `thousand` x `thousand` x `sixteen` / the cycles; the time is kept
//! for the next.
//!
//! Caller: `race::show_frame`; `frame_rate` is shown by `race::draw_frame_rate`. The rate is worked
//! out in plain floats. Fixed ("The frame rate never measured below some 28 frames a second" in
//! `wiki/Fixed-bugs.md`): the cycles from the whole counter, not its signed low halves.

use crate::race::owner::Race;
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};

/// The whole counter at the last measure (`frame_last` holds its low half).
static LAST: AtomicU64 = AtomicU64::new(0);

/// The rate worked out in plain floats.
// Logic in `rc2k_core::race::frame_rate::frame_rate`.
fn frame_rate_plain(race: &Race, cycles: u64, system: &crate::system::owner::System) {
    let course = &race.course;
    let rate = rc2k_core::race::frame_rate::frame_rate(
        system.machine().cpu_mhz.get() as i32,
        course.numbers.get().thousand.get(),
        course.numbers.get().sixteen.get(),
        cycles,
    );
    race.shared.switches.get().frame_rate.set(rate);
}

pub fn measure_frame_rate(
    race: &Race,
    scene: &crate::scene::owner::Scene,
    system: &crate::system::owner::System,
) -> Option<u32> {
    let shared = &race.shared;
    // Logic in `rc2k_core::race::frame_rate::{count_frame, cycles_since}`.
    use rc2k_core::race::frame_rate::{count_frame, cycles_since};
    let (count, due) = count_frame(shared.physics_tables.get().frame_rate_count.get());
    shared.physics_tables.get().frame_rate_count.set(count);
    if !due {
        return None;
    }
    let whole = crate::platform::arch::rdtsc();
    let now = whole as u32;
    shared.switches.get().frame_now.set(now);
    if let Some(cycles) = cycles_since(whole, LAST.swap(whole, Relaxed)) {
        scene.drawing.scene.get().scratch.set(cycles as u32);
        frame_rate_plain(race, cycles, system);
    }
    shared.switches.get().frame_last.set(now);
    Some(now)
}
