//! `marks::run_frame` (0x4D1549): the marks' frame drawn: `marks::swap_mask`,
//! `marks::tick_clock`; render states 0x15 = 4, 0x1C = 0, 0x13 = 5, 0x14 = 6, 0x1B = 1,
//! 3 = 1; `marks::run_rain`, `marks::run_snow`; `marks::draw_frame` (through the stub at
//! 0x4C4E28); `marks::ring_frame`; render state 0x14 = 2; `marks::flare_frame`; render
//! state 3 = 1; `marks::draw_mark_slots`, `marks::run_particles`; `marks::stage_record`,
//! `marks::draw_glow`, `scene::depth_buffer_on`, `scene::depth_test_on`. Three shares of
//! the frame's time (`rdtsc` over `dword_765114`, times `flt_93C8C4` / `C8` / `CC`) into
//! `flt_765134` (the draw), `flt_76511C` (the ring) and `flt_765120` (the whole).
//!
//! In:  nothing
//! Out: nothing (no caller reads the registers it leaves: none after 0x446408, 0x456472)
//!
//! Caller: the race's frame (0x456464 ..). Its calls the Rust versions, in the order above.

use crate::errors::Refused;
use crate::marks::tuning::MARKS_FRAME;
use crate::math::x87;
use crate::session::Float;
/// Where the share goes (its factor is `FRAME_K.times`): the draw, the ring, the whole.
static SHARES: [&Float; 3] = [
    &MARKS_FRAME.draw_share,
    &MARKS_FRAME.ring_share,
    &MARKS_FRAME.whole_share,
];
const STATES: [(u32, u32); 6] = [
    (4, 0x15),
    (0, 0x1C),
    (5, 0x13),
    (6, 0x14),
    (1, 0x1B),
    (1, 3),
];

fn rdtsc() -> u32 {
    unsafe { core::arch::x86::_rdtsc() as u32 }
}

fn share(at: usize, start: u32) {
    let to = SHARES[at];
    let took = rdtsc().wrapping_sub(start) as i32;
    to.set_bits(x87::int_over_int_times(
        took,
        MARKS_FRAME.frame_cycles.bits() as i32,
        FRAME_K.times[at].bits(),
    ));
}

/// The Rust version (see the header).
pub fn run_frame() -> Result<(), Refused> {
    use crate::marks::*;
    use crate::scene::set_render_state::set_render_state;
    {
        swap_mask::swap_mask()?;
        setup::tick_clock()?;
        let whole = rdtsc();
        for (value, state) in STATES {
            set_render_state(value, state);
        }
        run_rain::run_rain()?;
        run_snow::run_snow()?;
        let start = rdtsc();
        share(0, start);
        draw_frame::draw_frame()?;
        let start = rdtsc();
        ring_frame::ring_frame()?;
        share(1, start);
        set_render_state(2, 0x14);
        flare_frame::flare_frame()?;
        set_render_state(1, 3);
        draw_mark_slots::draw_mark_slots()?;
        run_particles::run_particles()?;
        share(2, whole);
        stage_record::stage_record()?;
        draw_glow::draw_glow()?;
        crate::scene::depth_buffer_on::depth_buffer(true);
        crate::scene::depth_test_on::depth_test_on();
    }
    Ok(())
}

/// The frame's time constants (IDA 0x93C8C0 .. 0x93C8D0, the image's layout; phase 7, typed state).
#[repr(C)]
pub struct FrameK {
    /// 0x93C8C0: `flt_93C8C0`: the marks' share factor (`frame_marks`)
    pub share_k: Float,
    /// 0x93C8C4: `flt_93C8C4`, `flt_93C8C8`, `flt_93C8CC`: the frame's time factors (`run_frame`)
    pub times: [Float; 3],
}

/// FrameK's first values.
pub static FRAME_K: FrameK = FrameK {
    share_k: Float::from_bits(0x42C80000),
    times: [
        Float::from_bits(0x42C80000),
        Float::from_bits(0x42C80000),
        Float::from_bits(0x42C80000),
    ],
};

impl FrameK {
    pub const START: usize = 0x93C8C0;
    pub const LEN: usize = 0x10;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        &FRAME_K as *const FrameK as usize
    }
}

#[cfg(test)]
mod frame_k_tests {
    use super::*;
    use core::mem::offset_of;

    fn bytes<T>(v: &T) -> Vec<u8> {
        let p = v as *const T as *const u8;
        (0..size_of::<T>())
            .map(|i| unsafe { p.add(i).read() })
            .collect()
    }

    #[test]
    fn frame_k_starts_as_the_games() {
        assert_eq!(size_of::<FrameK>(), FrameK::LEN);
        assert!(
            bytes(&FRAME_K) == crate::moved::image_bytes(FrameK::START, FrameK::LEN),
            "FRAME_K starts differently from the image"
        );
        assert_eq!(offset_of!(FrameK, share_k), 0x93C8C0 - FrameK::START);
        assert_eq!(offset_of!(FrameK, times), 0x93C8C4 - FrameK::START);
    }
}
