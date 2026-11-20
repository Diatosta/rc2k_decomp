//! `effects::add_drift_particle`: one entry of the marks' particle ring (the ring's first
//! free entry, while the used count is not 0xFFF) started from six values `v`: the kind;
//! `v[0]` .. `v[2]` the motion (the lift is not set); `v[3]`, `v[4]` and `v[5]` the place's
//! three coordinates, each stored as the place and as the last place (the second coordinate of
//! the last place less the drift lowering); the size 0x8000, the colour 0xFF000000, the life
//! 50000, and the ground taken from the view's place. With the ring full the six are dropped.
//!
//! Refuses a null ring (an entry to take), before anything.

use crate::effects::blocks::{read_particle, update_particle};
use crate::effects::blocks_store::ParticleKind;
use crate::effects::particles_store::Particles;
use crate::effects::ring_store::Ring;
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
    kind: ParticleKind,
    v: [[u8; 10]; 6],
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    {
        let used: u32 = ring.pool.used.get();
        if used == FULL {
            return Ok(());
        }
        let free: u32 = ring.pool.free.get();
        if free == 0 || read_particle(particles, free)?.before == 0 {
            return Err(BadInput("a null particle ring entry").into());
        }
        let e = read_particle(particles, free)?.before;
        ring.pool.free.set(e);
        ring.pool.used.set(used.wrapping_add(1));
        let lowered = x87::ext_less(v[4], &effects.kinds.particle_tuning.drift_lower).to_bits();
        let ground = x87::loaded(&frame.matrices.get().view_place[13]).to_bits();
        update_particle(particles, e, |p| {
            p.kind = kind as u32;
            p.motion_z = rounded(v[0]);
            p.motion_y = rounded(v[1]);
            p.motion_x = rounded(v[2]);
            p.z = rounded(v[3]);
            p.last_z = rounded(v[3]);
            p.y = rounded(v[4]);
            p.last_y = lowered;
            p.x = rounded(v[5]);
            p.last_x = rounded(v[5]);
            p.size = 0x8000;
            p.colour = 0xFF00_0000;
            p.life = LIFE;
            p.ground = ground;
        })?;
    }
    Ok(())
}
