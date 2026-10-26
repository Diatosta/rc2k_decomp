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
//! `race::draw_frame_rate`. The rate is worked out in plain floats.

use crate::physics::tables_store::PHYSICS;
use crate::race::numbers_store::NUMBERS;
use crate::race::switches_store::FRAME_SWITCHES;
use crate::scene::draw_store::DRAW;
use crate::session::MACHINE;

/// The rate worked out in plain floats.
// Converted: `rc2k_core::race::frame_rate::frame_rate`.
fn frame_rate_plain() {
    let rate = rc2k_core::race::frame_rate::frame_rate(
        MACHINE.cpu_mhz.get() as i32,
        NUMBERS.thousand.get(),
        NUMBERS.sixteen.get(),
        DRAW.scratch.bits() as i32,
    );
    FRAME_SWITCHES.frame_rate.set_bits(rate.to_bits());
}

/// The Rust version: the eax the original leaves when it changes it (on the 16th call, the
/// time-stamp counter's low half, `dword_6214E8`); its edx and ebx (the counter's high
/// half, the cycles) are read by no caller.
pub unsafe fn measure_frame_rate() -> Option<u32> {
    // Converted: `rc2k_core::race::frame_rate::{count_frame, cycles_since}`.
    use rc2k_core::race::frame_rate::{count_frame, cycles_since};
    unsafe {
        let (count, due) = count_frame(PHYSICS.frame_rate_count.bits());
        PHYSICS.frame_rate_count.set(count);
        if !due {
            return None;
        }
        let now = crate::arch::_rdtsc() as u32;
        FRAME_SWITCHES.frame_now.set_bits(now);
        let last: u32 = FRAME_SWITCHES.frame_last.bits();
        if let Some(cycles) = cycles_since(now, last) {
            DRAW.scratch.set_bits(cycles);
            frame_rate_plain();
        }
        let now: u32 = FRAME_SWITCHES.frame_now.bits();
        FRAME_SWITCHES.frame_last.set_bits(now);
        Some(now)
    }
}
