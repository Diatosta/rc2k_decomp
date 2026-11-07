//! `physics::surface_noise` (0x43BC68): the ground's roughness under the work car from two
//! noise maps (`race::sample_noise`, esi `unk_A2AB34` and `unk_A2AB04`): each sample
//! scaled into its range (lo + (hi - lo) x it >> 16, `imul` / `shrd`: `qword_609344` and
//! `dword_60AD18` .. `1C`) and held to 0 ..= 0xFF (by the sign). The first into +0x900 and
//! +0x904; the second into +0x908, and at least `dword_60AD10` (unsigned) through
//! `dword_62400C`, / 256 into +0x280 (`unk_71A258`). Then with `dword_60934C` set +0x900
//! 0, else +0x904 0.
//!
//! In:  nothing
//! Out: eax the second sample (held), ebx its range, ecx and edx as left, esi
//!      `unk_A2AB04`; no flags read
//!
//! Caller: `sub_43B635` (0x43B714), every physics step.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::math::x87_run::{ext_div2, int_80};
use crate::race::frame_draw::sample_noise;
use crate::race::owner::Race;
use rc2k_core::physics::ground::{noise_level, rough_floor};

/// lo + (hi - lo) x sample >> 16, held to 0 ..= 0xFF.
fn scaled(sample: u32, lo: u32, hi: u32) -> u32 {
    noise_level(sample, lo, hi).0
}

/// The noise map `x` bytes into `g_noiseBytes`: its sample under the work car (what
/// `race::sample_noise` leaves in eax).
fn sample(
    shared: &crate::race::owner_shared::Shared,
    x: usize,
) -> Result<u32, crate::errors::BadInput> {
    let scene = crate::scene::owner::scene();
    sample_noise(shared, x, scene)
}

/// The Rust version.
// Converted: `rc2k_core::physics::ground::{noise_level, rough_floor}` (the share / 256
// stays x87).
pub fn surface_noise(
    race: &Race,
    scene: &crate::scene::owner::Scene,
) -> Result<(), crate::errors::BadInput> {
    {
        let first = scaled(
            sample(&race.shared, 0x50)?,
            race.course.stage_prefs.blit_cleared[0].get(),
            race.course.stage_prefs.blit_cleared[1].get(),
        );
        race.shared.car.surface_noise_a.set(first);
        race.shared.car.surface_noise_b.set(first);
        let second = scaled(
            sample(&race.shared, 0x20)?,
            race.course.stage_prefs.surface_scale[0].get(),
            race.course.stage_prefs.surface_scale[1].get(),
        );
        race.shared.car.surface_noise_second.set(second);
        let floor = rough_floor(second, race.course.stage_prefs.surface_floor.get());
        scene.drawing.scene.scratch.set(floor);
        race.shared.car.surface_blend.set_bits(ext_div2(
            int_80(floor as i32, false),
            race.course.numbers.k256.bits(),
            0x3F80_0000,
        ));
        if race.course.stage_prefs.noise_pick.get() != 0 {
            race.shared.car.surface_noise_a.set(0);
        } else {
            race.shared.car.surface_noise_b.set(0);
        }
    }
    Ok(())
}
