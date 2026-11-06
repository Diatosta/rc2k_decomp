//! `race::measure_frame_rate` (0x42009E): every 16th call (`dword_613838` counting, back to 0
//! at 0x10) the time-stamp counter (`rdtsc`) into `dword_6214E8`; with the cycles since the
//! last time (`dword_6214EC`, into `dword_62400C`) above 0 (signed), the frame rate
//! `flt_6214F0` = the CPU's clock `dword_5183E8` x `flt_5C5D18` x `flt_5C5D18` x
//! `flt_5C5D08` / the cycles (x87, as the original); the time kept for the next.
//!
//! In:  nothing
//! Out: unchanged but on the 16th: eax and edx the counter, ebx the cycles; no flags read
//!
//! Caller: `sub_4547AE` (0x4547FD, `race::show_frame`); `flt_6214F0` is shown by
//! `race::draw_frame_rate`. The rate is worked out in plain floats. Fixed (2026-10-30,
//! `wiki/Fixed-bugs.md`): the cycles from the whole counter, not its signed low halves.

use crate::physics::tables_store::PHYSICS;
use crate::race::switches_store::FRAME_SWITCHES;
use crate::scene::draw_store::DRAW;
use crate::session::MACHINE;
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};

/// The whole counter at the last measure (`dword_6214EC` holds its low half).
static LAST: AtomicU64 = AtomicU64::new(0);

/// The rate worked out in plain floats.
// Converted: `rc2k_core::race::frame_rate::frame_rate`.
fn frame_rate_plain(cycles: u64) {
    let rate = rc2k_core::race::frame_rate::frame_rate(
        MACHINE.cpu_mhz.get() as i32,
        crate::race::owner::race().course.numbers.thousand.get(),
        crate::race::owner::race().course.numbers.sixteen.get(),
        cycles,
    );
    FRAME_SWITCHES.frame_rate.set_bits(rate.to_bits());
}

/// The Rust version: the eax the original leaves when it changes it (on the 16th call, the
/// time-stamp counter's low half, `dword_6214E8`); its edx and ebx (the counter's high
/// half, the cycles) are read by no caller.
pub fn measure_frame_rate() -> Option<u32> {
    // Converted: `rc2k_core::race::frame_rate::{count_frame, cycles_since}`.
    use rc2k_core::race::frame_rate::{count_frame, cycles_since};
    let (count, due) = count_frame(PHYSICS.frame_rate_count.bits());
    PHYSICS.frame_rate_count.set(count);
    if !due {
        return None;
    }
    let whole = crate::arch::rdtsc();
    let now = whole as u32;
    FRAME_SWITCHES.frame_now.set_bits(now);
    if let Some(cycles) = cycles_since(whole, LAST.swap(whole, Relaxed)) {
        DRAW.scratch.set_bits(cycles as u32);
        frame_rate_plain(cycles);
    }
    FRAME_SWITCHES.frame_last.set_bits(now);
    Some(now)
}
