//! `effects::add_drift_particle`: one entry of the marks' particle ring (the ring's first
//! free entry, while the used count is not 0xFFF) started from six values `v`: the kind;
//! `v[0]` .. `v[2]` the motion (the lift is not set); `v[3]`, `v[4]` and `v[5]` the place's
//! three coordinates, each stored as the place and as the last place (the second coordinate of
//! the last place less the drift lowering); the size 0x8000, the colour 0xFF000000, the life
//! 50000, and the ground taken from the view's place. With the ring full the six are dropped.
//!
//! Refuses a null ring (an entry to take), before anything.

use crate::effects::blocks::{particle, set_particle};
use crate::effects::ring_store::Ring;
use crate::effects::run_particles::Particles;
use crate::errors::{BadInput, Refused};
use crate::math::matrix3::rounded;
use crate::math::x87;
use crate::race::owner_frame::Frame;

const FULL: u32 = 0xFFF;
const LIFE: u32 = 0xC350;

pub(super) fn add_drift_particle(
    frame: &Frame,
    particles: &Particles,
    ring: &Ring,
    kind: u32,
    v: [[u8; 10]; 6],
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    {
        let used: u32 = ring.pool.used.get();
        if used == FULL {
            return Ok(());
        }
        let free: u32 = ring.pool.free.get();
        if free == 0 || particle(particles, free, 0)? == 0 {
            return Err(BadInput("a null particle ring entry").into());
        }
        let e = particle(particles, free, 0)?;
        ring.pool.free.set(e);
        ring.pool.used.set(used.wrapping_add(1));
        let put = |o: u32, x: u32| set_particle(particles, e, o, x);
        put(8, kind)?;
        put(0x34, rounded(v[0]))?;
        put(0x2C, rounded(v[1]))?;
        put(0x28, rounded(v[2]))?;
        put(0x14, rounded(v[3]))?;
        put(0x20, rounded(v[3]))?;
        put(0x10, rounded(v[4]))?;
        put(
            0x1C,
            x87::ext_less(v[4], &effects.kinds.particle_tuning.drift_lower).to_bits(),
        )?;
        put(0xC, rounded(v[5]))?;
        put(0x18, rounded(v[5]))?;
        put(0x38, 0x8000)?;
        put(0x3C, 0xFF00_0000)?;
        put(0x44, LIFE)?;
        put(
            0x24,
            x87::loaded(&frame.matrices.get().view_place[13]).to_bits(),
        )?;
    }
    Ok(())
}
