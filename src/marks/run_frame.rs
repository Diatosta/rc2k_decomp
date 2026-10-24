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
use crate::marks::store::{Slot, game_read, game_write};
use crate::math::x87;

const FRAME_CYCLES_RVA: Slot = Slot::at(0x765114); // dword_765114
/// (where the share goes, its factor): the draw, the ring, the whole.
const SHARES: [(Slot, Slot); 3] = [
    (Slot::at(0x765134), Slot::at(0x93C8C4)),
    (Slot::at(0x76511C), Slot::at(0x93C8C8)),
    (Slot::at(0x765120), Slot::at(0x93C8CC)),
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

unsafe fn share(at: usize, start: u32) {
    unsafe {
        let (to, k) = SHARES[at];
        let took = rdtsc().wrapping_sub(start) as i32;
        game_write(
            to,
            x87::int_over_int_times(took, game_read(FRAME_CYCLES_RVA), game_read(k)),
        );
    }
}

/// The Rust version (see the header).
pub unsafe fn run_frame() -> Result<(), Refused> {
    use crate::marks::*;
    use crate::scene::set_render_state::set_render_state;
    unsafe {
        swap_mask::swap_mask()?;
        setup::tick_clock();
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
