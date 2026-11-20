//! `effects::trail_particles`: eight kind-4 entries of the effects' particle ring (taken from the
//! ring's first free entry while the used count is not 0xFFF), started along the way the particle
//! came: each place is the particle's place moved a random share toward its last place, its motion
//! the (last - place) and the particle's own motion times the trail's factors, moved by
//! `effects::jitter_point`; size from the particle's motion y, life 300.
//!
//! Refuses a null particle or ring (an entry to take), before anything.

use crate::effects::blocks::{particle, set_particle};
use crate::effects::jitter_point::jitter_point;
use crate::effects::ring_store::Ring;
use crate::effects::run_particles::Particles;
use crate::errors::{BadInput, Refused};
use crate::math::matrix3::{extended, rounded};
use crate::math::x87;
use crate::math::x87_float as bh;
use crate::race::owner_frame::Frame;

const FULL: u32 = 0xFFF;
const COUNT: u32 = 8;
/// Per axis (place, last place and motion byte offsets in a particle); its two factors are
/// `PARTICLE_TUNING.trail_axes[2 n]` and `[2 n + 1]`.
const AXES: [(u32, u32, u32); 3] = [(0xC, 0x18, 0x28), (0x10, 0x1C, 0x2C), (0x14, 0x20, 0x34)];
const LIFE: u32 = 300;

/// The entries it takes, from the ring's first free one along the links.
fn entries_taken(particles: &Particles, ring: &Ring) -> Result<Vec<u32>, Refused> {
    Ok({
        let used: u32 = ring.pool.used.get();
        let n = if used <= FULL {
            (FULL - used).min(COUNT)
        } else {
            COUNT
        };
        let mut e: u32 = ring.pool.free.get();
        let mut out = Vec::new();
        for _ in 0..n {
            if e == 0 {
                break;
            }
            e = particle(particles, e, 0)?;
            out.push(e);
        }
        out
    })
}

/// `p` is the particle it trails.
pub(super) fn trail_particles(
    frame: &Frame,
    particles: &Particles,
    ring: &Ring,
    p: u32,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    {
        if ring.pool.used.get() != FULL {
            if p == 0 {
                return Err(BadInput("a null particle").into());
            }
            if ring.pool.free.get() == 0 || entries_taken(particles, ring)?.contains(&0) {
                return Err(BadInput("a null particle ring entry").into());
            }
        }
        for _ in 0..COUNT {
            let used: u32 = ring.pool.used.get();
            if used == FULL {
                continue;
            }
            let e = particle(particles, ring.pool.free.get(), 0)?;
            ring.pool.free.set(e);
            ring.pool.used.set(used.wrapping_add(1));
            let put = |o: u32, v: u32| set_particle(particles, e, o, v);
            put(8, 4)?;
            let s = effects
                .setup
                .work
                .seed
                .get()
                .wrapping_mul(0x9D)
                .wrapping_add(0x47);
            effects.setup.work.seed.set(s);
            let k1 = effects.frame.draw_work.uv_k.get();
            let k2 = effects.kinds.particle_tuning.trail_random_k.get();
            ring.pool.burst_radius.set(bh::stf(
                bh::int((s & 0xFFFF) as i32) * bh::ld(k1) * bh::ld(k2),
            ));
            let radius = ring.pool.burst_radius.get();
            for (place, last, _) in AXES {
                put(
                    place,
                    x87::toward(
                        particle(particles, p, place)?,
                        particle(particles, p, last)?,
                        radius,
                    )
                    .to_bits(),
                )?;
            }
            for (n, (place, last, motion)) in AXES.into_iter().enumerate() {
                put(
                    motion,
                    bh::st(
                        (bh::ld(particle(particles, p, last)?)
                            - bh::ld(particle(particles, p, place)?))
                            * bh::ld(effects.kinds.particle_tuning.trail_axes[2 * n].get())
                            + bh::ld(particle(particles, p, motion)?)
                                * bh::ld(effects.kinds.particle_tuning.trail_axes[2 * n + 1].get()),
                    ),
                )?;
            }
            let (moved, _) = jitter_point(
                [
                    extended(effects.kinds.particle_tuning.trail_spread.get()),
                    extended(particle(particles, e, 0x34)?),
                    extended(particle(particles, e, 0x2C)?),
                    extended(particle(particles, e, 0x28)?),
                ],
                effects,
            );
            put(0x34, rounded(moved[0]))?;
            put(0x2C, rounded(moved[1]))?;
            put(0x28, rounded(moved[2]))?;
            let [a, b] = effects
                .kinds
                .particle_tuning
                .trail_size_k
                .each_ref()
                .map(crate::memory::session::PlainFloat::get);
            put(
                0x38,
                x87::times_plus_to_int(a, particle(particles, p, 0x2C)?, b) as u32,
            )?;
            put(0x3C, 0xFF00_0000)?;
            put(0x44, LIFE)?;
            put(
                0x40,
                x87::over_int(&effects.kinds.particle_tuning.trail_life_k, LIFE as i32).to_bits(),
            )?;
            put(
                0x24,
                x87::loaded(&frame.matrices.get().view_place[13]).to_bits(),
            )?;
        }
    }
    Ok(())
}
