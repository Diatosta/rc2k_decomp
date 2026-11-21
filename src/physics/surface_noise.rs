//! The ground's roughness under the work car from two noise maps (`race::sample_noise`): each
//! sample is scaled into its range (lo + (hi - lo) x it >> 16: `precipitation_range` and `surface_scale`)
//! and held to 0 ..= 0xFF. The first goes into `surface_noise_a` and `surface_noise_b`; the second
//! into `surface_noise_second`, and at least `surface_floor` (unsigned), / 256, into
//! `surface_blend`. Then with `noise_pick` set `surface_noise_a` becomes 0, else `surface_noise_b`.

use crate::math::x87::{ext_div2, int_80};
use crate::race::frame_draw::sample_noise;
use crate::race::numbers_store::Numbers;
use crate::race::owner_shared::RaceShared;
use crate::race::prefs_store::StagePrefsRef;
use rc2k_core::physics::ground::{noise_level, rough_floor};

/// lo + (hi - lo) x sample >> 16, held to 0 ..= 0xFF.
fn scaled(sample: u32, lo: u32, hi: u32) -> u32 {
    noise_level(sample, lo, hi).0
}

/// The noise map `x` bytes into the noise bytes: its sample under the work car (what
/// `race::sample_noise` returns).
fn sample(
    shared: &crate::race::owner_shared::RaceShared,
    x: usize,
) -> Result<u32, crate::errors::BadInput> {
    sample_noise(shared.buffers.engine_tables(), shared.car(), x)
}

// Core: `rc2k_core::physics::ground::{noise_level, rough_floor}` (the share / 256 stays x87).
pub fn surface_noise(
    stage_prefs: StagePrefsRef<'_>,
    numbers: &Numbers,
    shared: &RaceShared,
) -> Result<(), crate::errors::BadInput> {
    {
        let first = scaled(
            sample(shared, 0x50)?,
            stage_prefs.read(|p| p.precipitation_range[0]),
            stage_prefs.read(|p| p.precipitation_range[1]),
        );
        shared.car().surface_noise_a.set(first);
        shared.car().surface_noise_b.set(first);
        let second = scaled(
            sample(shared, 0x20)?,
            stage_prefs.read(|p| p.surface_scale[0]),
            stage_prefs.read(|p| p.surface_scale[1]),
        );
        shared.car().surface_noise_second.set(second);
        let floor = rough_floor(second, stage_prefs.read(|p| p.surface_floor));
        shared
            .car()
            .surface_blend
            .set(ext_div2(int_80(floor as i32, false), numbers.k256, 1.0f32));
        if stage_prefs.read(|p| p.noise_pick) != 0 {
            shared.car().surface_noise_a.set(0);
        } else {
            shared.car().surface_noise_b.set(0);
        }
    }
    Ok(())
}
