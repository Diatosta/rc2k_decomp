//! `effects::drift_offsets` (0x4C1BA9): two whole-number offsets drifting by a wandering pace.
//! When `drift_wait` ticks of the scene clock have passed since `drift_since` (unsigned), new
//! targets (`drift_target`) by the effects' random (its low 12 bits and the next 12, less
//! 0x7FF, times the `target_scale`s), a new wait ((its top byte + 0x40) times 5), the time
//! kept, and a second random's halves times 0x100 in `drift_halves`. Then each pace
//! (`drift_pace`) eased toward its target (`x87::ease`, by `ease`) and added to its offset
//! (the view's x and z, rounded).
//!
//! One caller, `effects::run_rain` (the effects' frame).
//!
//! the x87 version stays for the Rust callers in unconverted code; the offsets' `fistp`
//! stays the x87's).

use crate::effects::rain::Weather;
use crate::math::x87;
use crate::math::x87_float as bh;
use crate::race::owner::Race;

/// The marks' random: times 0x9D plus 0x47.
fn next_seed(effects: &crate::effects::owner::Effects) -> u32 {
    let setup = &effects.setup;
    let s = setup.work.seed.get().wrapping_mul(0x9D).wrapping_add(0x47);
    setup.work.seed.set(s);
    s
}

pub fn drift_offsets(race: &Race, weather: &Weather, effects: &crate::effects::owner::Effects) {
    let (r, k) = (&weather.rain, &weather.rain_rules);
    {
        let since = race
            .shared
            .physics_tables
            .scene_clock
            .bits()
            .wrapping_sub(r.drift_since.get());
        if since >= r.drift_wait.get() {
            let s = next_seed(effects);
            let x = (s & 0xFFF).wrapping_sub(0x7FF);
            r.drift_target[0].set(x87::int_times(x as i32, k.target_scale[0]));
            let y = ((s >> 12) & 0xFFF).wrapping_sub(0x7FF);
            r.drift_target[2].set(x87::int_times(y as i32, k.target_scale[1]));
            r.drift_wait.set(((s >> 24) + 0x40) * 5);
            r.drift_since
                .set(race.shared.physics_tables.scene_clock.bits());
            let s = next_seed(effects);
            r.drift_halves[0].set((s & 0xFFFF) << 8);
            r.drift_halves[2].set((s >> 16) << 8);
        }
        for (n, at) in [0, 2].into_iter().enumerate() {
            let pace = x87::ease(&r.drift_target[at], &r.drift_pace[at], k.ease[n].to_bits());
            r.drift_pace[at].set_bits(pace.to_bits());
            let offset = bh::to_int(bh::int(r.view[at].get() as i32) + bh::ld(pace.to_bits()));
            r.view[at].set(offset as u32);
        }
    }
}
