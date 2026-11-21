//! `effects::trail_particles`: eight kind-4 entries of the effects' particle ring (taken from the
//! ring's first free entry while the used count is not 0xFFF), started along the way the particle
//! came: each place is the particle's place moved a random share toward its last place, its motion
//! the (last - place) and the particle's own motion times the trail's factors, moved by
//! `effects::jitter_point`; size from the particle's motion y, life 300.
//!
//! Refuses a null particle or ring (an entry to take), before anything.

use crate::effects::blocks::{read_particle, update_particle};
use crate::effects::blocks_store::ParticleKind;
use crate::effects::jitter_point::jitter_point;
use crate::effects::particles_store::Particles;
use crate::effects::ring_store::Ring;
use crate::errors::{BadInput, Refused};
use crate::math::matrix3::{extended, rounded};
use crate::math::x87;
use crate::math::x87_float as bh;
use crate::race::owner_frame::Frame;

const FULL: u32 = 0xFFF;
const COUNT: u32 = 8;
/// The three axes (x, y, z); axis `n`'s two factors are `PARTICLE_TUNING.trail_axes[2 n]` and
/// `[2 n + 1]`.
const AXES: [usize; 3] = [0, 1, 2];
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
            e = read_particle(particles, e)?.before;
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
            let e = read_particle(particles, ring.pool.free.get())?.before;
            ring.pool.free.set(e);
            ring.pool.used.set(used.wrapping_add(1));
            update_particle(particles, e, |q| q.kind = ParticleKind::Trail as u32)?;
            let s = effects
                .setup
                .work
                .borrow()
                .seed
                .wrapping_mul(0x9D)
                .wrapping_add(0x47);
            effects.setup.work.borrow_mut().seed = s;
            let k1 = effects.frame.draw_work.borrow().uv_k;
            let k2 = effects.kinds.particle_tuning.trail_random_k;
            ring.pool.burst_radius.set(bh::stf(
                bh::int((s & 0xFFFF) as i32) * bh::ld(k1) * bh::ld(k2),
            ));
            let radius = ring.pool.burst_radius.get();
            for axis in AXES {
                let toward = x87::toward(
                    read_particle(particles, p)?.place(axis),
                    read_particle(particles, p)?.last_place(axis),
                    radius,
                )
                .to_bits();
                update_particle(particles, e, |q| q.set_place(axis, toward))?;
            }
            for axis in AXES {
                let tuning = &effects.kinds.particle_tuning;
                let motion = bh::st(
                    (bh::ld(read_particle(particles, p)?.last_place(axis))
                        - bh::ld(read_particle(particles, p)?.place(axis)))
                        * bh::ld(tuning.trail_axes[2 * axis])
                        + bh::ld(read_particle(particles, p)?.motion(axis))
                            * bh::ld(tuning.trail_axes[2 * axis + 1]),
                );
                update_particle(particles, e, |q| q.set_motion(axis, motion))?;
            }
            let own = read_particle(particles, e)?;
            let (moved, _) = jitter_point(
                [
                    extended(effects.kinds.particle_tuning.trail_spread),
                    extended(own.motion_z),
                    extended(own.motion_y),
                    extended(own.motion_x),
                ],
                effects,
            );
            update_particle(particles, e, |q| {
                q.motion_z = rounded(moved[0]);
                q.motion_y = rounded(moved[1]);
                q.motion_x = rounded(moved[2]);
            })?;
            let [a, b] = effects.kinds.particle_tuning.trail_size_k;
            let size = x87::times_plus_to_int(a, read_particle(particles, p)?.motion_y, b) as u32;
            let colour_step =
                x87::over_int(effects.kinds.particle_tuning.trail_life_k, LIFE as i32).to_bits();
            let ground = x87::loaded(&frame.matrices.get().view_place[13]).to_bits();
            update_particle(particles, e, |q| {
                q.size = size;
                q.colour = 0xFF00_0000;
                q.life = LIFE;
                q.colour_step = colour_step;
                q.ground = ground;
            })?;
        }
    }
    Ok(())
}
