//! `race::measure_frame_rate` (0x42009E): every 16th call (`frame_rate_count` counting, back to 0 at
//! 0x10) the time-stamp counter (`rdtsc`) into `frame_now`; with the cycles since the last time
//! (`frame_last`, into `scratch`) above 0 (signed), the frame rate `frame_rate` = the CPU's
//! clock `dword_5183E8` x `thousand` x `thousand` x `sixteen` / the cycles (x87, as the
//! original); the time kept for the next.
//!
//! Returns: unchanged but on the 16th: and the counter, the cycles
//!
//! Caller: 0x4547AE (0x4547FD, `race::show_frame`); `frame_rate` is shown by
//! `race::draw_frame_rate`. The rate is worked out in plain floats. Fixed (see
//! `wiki/Fixed-bugs.md`): the cycles from the whole counter, not its signed low halves.

use crate::race::owner::Race;
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};

/// The whole counter at the last measure (`frame_last` holds its low half).
static LAST: AtomicU64 = AtomicU64::new(0);

/// The rate worked out in plain floats.
// Converted: `rc2k_core::race::frame_rate::frame_rate`.
fn frame_rate_plain(race: &Race, cycles: u64, system: &crate::system::owner::System) {
    let course = &race.course;
    let rate = rc2k_core::race::frame_rate::frame_rate(
        system.session.machine.cpu_mhz.get() as i32,
        course.numbers.thousand.get(),
        course.numbers.sixteen.get(),
        cycles,
    );
    race.shared.switches.frame_rate.set(rate);
}

pub fn measure_frame_rate(
    race: &Race,
    scene: &crate::scene::owner::Scene,
    system: &crate::system::owner::System,
) -> Option<u32> {
    let shared = &race.shared;
    // Converted: `rc2k_core::race::frame_rate::{count_frame, cycles_since}`.
    use rc2k_core::race::frame_rate::{count_frame, cycles_since};
    let (count, due) = count_frame(shared.physics_tables.frame_rate_count.get());
    shared.physics_tables.frame_rate_count.set(count);
    if !due {
        return None;
    }
    let whole = crate::arch::rdtsc();
    let now = whole as u32;
    shared.switches.frame_now.set(now);
    if let Some(cycles) = cycles_since(whole, LAST.swap(whole, Relaxed)) {
        scene.drawing.scene.scratch.set(cycles as u32);
        frame_rate_plain(race, cycles, system);
    }
    shared.switches.frame_last.set(now);
    Some(now)
}
