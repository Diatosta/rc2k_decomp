//! `effects::run_frame` (0x4D1549): the marks' frame drawn: `effects::swap_mask`,
//! `effects::tick_clock`; render states 0x15 = 4, 0x1C = 0, 0x13 = 5, 0x14 = 6, 0x1B = 1,
//! 3 = 1; `effects::run_rain`, `effects::run_snow`; `effects::draw_frame` (through the stub at
//! 0x4C4E28); `effects::ring_frame`; render state 0x14 = 2; `effects::flare_frame`; render
//! state 3 = 1; `effects::draw_mark_slots`, `effects::run_particles`; `effects::stage_record`,
//! `effects::draw_glow`, `scene::depth_buffer_on`, `scene::depth_test_on`. Three shares of
//! the frame's time (`rdtsc` over `dword_765114`, times `flt_93C8C4` / `C8` / `CC`) into
//! `flt_765134` (the draw), `flt_76511C` (the ring) and `flt_765120` (the whole).
//!
//! Caller: the race's frame (0x456464 ..). Its calls the Rust versions, in the order above.

use crate::errors::Refused;
use crate::math::x87;
use crate::race::owner::Race;
use crate::session::Float;
/// Where the share goes (its factor is `FRAME_K.times`): the draw, the ring, the whole.
static SHARES: [&Float; 3] = [
    &crate::effects::owner::EFFECTS.setup.frame.draw_share,
    &crate::effects::owner::EFFECTS.setup.frame.ring_share,
    &crate::effects::owner::EFFECTS.setup.frame.whole_share,
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
    crate::arch::rdtsc() as u32
}

fn share(at: usize, start: u32, effects: &crate::effects::owner::Effects) {
    let to = SHARES[at];
    let took = rdtsc().wrapping_sub(start) as i32;
    to.set_bits(x87::int_over_int_times(
        took,
        effects.setup.frame.frame_cycles.bits() as i32,
        FRAME_K.times[at].to_bits(),
    ));
}

/// The Rust version (see the header).
pub fn run_frame(race: &Race) -> Result<(), Refused> {
    let gfx = crate::gfx::owner::gfx();
    let scene = crate::scene::owner::scene();
    use crate::effects::*;
    use crate::scene::set_render_state::set_render_state;
    let effects = owner::effects();
    let (weather, particles, ring) = (&effects.weather, &effects.particles, &effects.ring);
    let paths = &effects.paths;
    {
        swap_mask::swap_mask(&effects.masks, scene, gfx, effects)?;
        setup::tick_clock(&race.shared, effects)?;
        let whole = rdtsc();
        for (value, state) in STATES {
            set_render_state(value, state);
        }
        run_rain::run_rain(race, weather, scene, effects)?;
        run_snow::run_snow(race, weather, scene, effects)?;
        let start = rdtsc();
        share(0, start, effects);
        draw_frame::draw_frame(race, effects)?;
        let start = rdtsc();
        ring_frame::ring_frame(
            &race.frame,
            &effects.masks,
            weather,
            particles,
            ring,
            scene,
            effects,
        )?;
        share(1, start, effects);
        set_render_state(2, 0x14);
        flare_frame::flare_frame(paths, scene, effects)?;
        set_render_state(1, 3);
        draw_mark_slots::draw_mark_slots(race, paths, scene, effects)?;
        run_particles::run_particles(&race.frame, particles, ring, effects)?;
        share(2, whole, effects);
        stage_record::stage_record(&race.frame)?;
        draw_glow::draw_glow(scene, effects)?;
        crate::scene::depth_buffer_on::depth_buffer(true);
        crate::scene::depth_test_on::depth_test_on();
    }
    Ok(())
}

/// The frame's time constants (IDA 0x93C8C0 .. 0x93C8D0, the image's layout; phase 7, typed state).
#[repr(C)]
pub struct FrameK {
    /// 0x93C8C0: `flt_93C8C0`: the marks' share factor (`frame_marks`)
    pub share_k: f32,
    /// 0x93C8C4: `flt_93C8C4`, `flt_93C8C8`, `flt_93C8CC`: the frame's time factors (`run_frame`)
    pub times: [f32; 3],
}

/// FrameK's first values.
pub static FRAME_K: FrameK = FrameK {
    share_k: f32::from_bits(0x42C80000),
    times: [
        f32::from_bits(0x42C80000),
        f32::from_bits(0x42C80000),
        f32::from_bits(0x42C80000),
    ],
};

#[cfg(test)]
impl FrameK {
    pub const START: usize = 0x93C8C0;
    pub const LEN: usize = 0x10;
}

#[cfg(test)]
mod frame_k_tests {
    use super::*;
    use core::mem::offset_of;

    fn bytes<T>(v: &T) -> Vec<u8> {
        let p = std::ptr::from_ref::<T>(v).cast::<u8>();
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
