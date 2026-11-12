//! `effects::burst_particles` (0x4D0146): up to 0x100 entries of the effects' ring started at
//! the place of the record (+0xC .. +0x14), each in a random direction on a sphere of a
//! random radius: a burst flag is set; the radius (the effects' random's low word times a
//! constant, plus a base, times a scale) is kept, and so are a kind ((its low 3 bits) times
//! 0x400) and ((its bit 16) + 8); then each entry taken from the ring's first free one (its
//! +0, while the used count is not 0xFFF): +8 the first kept value, +0x48 the second, the
//! place, a direction by two angles of the next random (the sine and cosine tables) times
//! the radius at +0x28 .. +0x34, +0x38 0x10000, +0x3C 0xFF000000, +0x44 3000 and +0x40 the
//! life factor / 3000.
//!
//! Takes: the record
//!
//! Callers: `effects::run_particles` and its dust step. Refuses a null record or ring (when an
//! entry is to be taken), before anything.

use crate::effects::blocks::{particle, set_particle};
use crate::effects::ring_store::Ring;
use crate::effects::run_particles::Particles;
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::math::x87_float as bh;
use crate::sound::owner::Sound;

const ENTRIES: u32 = 0x100;
const FULL: u32 = 0xFFF;
const LIFE: u32 = 0xBB8;

/// The marks' random: times 0x9D plus 0x47.
fn next_seed(effects: &crate::effects::owner::Effects) -> u32 {
    let setup = &effects.setup;
    let s = setup.work.seed.get().wrapping_mul(0x9D).wrapping_add(0x47);
    setup.work.seed.set(s);
    s
}

/// The entries the burst takes, from `g_marksRingFree` along +0.
fn entries_taken(particles: &Particles, ring: &Ring) -> Result<Vec<u32>, Refused> {
    Ok({
        let used: u32 = ring.pool.used.get();
        let count = if used <= FULL {
            (FULL - used).min(ENTRIES)
        } else {
            ENTRIES
        };
        let mut e: u32 = ring.pool.free.get();
        let mut out = Vec::new();
        for _ in 0..count {
            if e == 0 {
                break;
            }
            e = particle(particles, e, 0)?;
            out.push(e);
        }
        out
    })
}

/// `record` is the particle it bursts from.
pub fn burst_particles(
    engine_tables: &crate::scene::engine_tables_store::EngineTables,
    particles: &Particles,
    ring: &Ring,
    record: u32,
    effects: &crate::effects::owner::Effects,
    sound: &Sound,
) -> Result<(), Refused> {
    {
        if ring.pool.used.get() != FULL {
            if record == 0 {
                return Err(BadInput("a null record").into());
            }
            if entries_taken(particles, ring)?.contains(&0) || ring.pool.free.get() == 0 {
                return Err(BadInput("a null ring entry").into());
            }
        }
        sound.race.stage.get().random[1].set(1);
        let s = next_seed(effects);
        let r = s & 0xFFFF;
        let m = effects.frame.draw_work.uv_k.get();
        let [c, k] = effects
            .kinds
            .particle_tuning
            .burst_scale
            .each_ref()
            .map(super::super::session::PlainFloat::get);
        let radius = bh::stf((bh::int(r as i32) * bh::ld(m) + bh::ld(c)) * bh::ld(k));
        ring.pool.burst_radius.set(radius);
        ring.pool.kind.set((s & 7) * 0x400);
        ring.pool.nine.set((s.rotate_right(16) & 1) + 8);
        let sine =
            |n: u32| crate::scene::engine_tables_store::sine(engine_tables, n as usize).get();
        let cosine =
            |n: u32| crate::scene::engine_tables_store::cosine(engine_tables, n as usize).get();
        for _ in 0..ENTRIES {
            let used: u32 = ring.pool.used.get();
            if used == FULL {
                continue;
            }
            let e: u32 = particle(particles, ring.pool.free.get(), 0)?;
            ring.pool.free.set(e);
            ring.pool.used.set(used.wrapping_add(1));
            let put = |o: u32, v: u32| set_particle(particles, e, o, v);
            put(8, ring.pool.nine.get())?;
            put(0x48, ring.pool.kind.get())?;
            for o in [0xC, 0x10, 0x14] {
                put(o, x87::loaded(particle(particles, record, o)?).to_bits())?;
            }
            let s = next_seed(effects);
            let a = s.rotate_right(16) & 0xFFF;
            let b = s & 0xFFF;
            let radius = ring.pool.burst_radius.get();
            put(0x34, x87::mul(sine(a), radius).to_bits())?;
            put(
                0x28,
                x87::zero_times_plus_times(cosine(a), cosine(b), sine(b), radius, false).to_bits(),
            )?;
            put(
                0x2C,
                x87::zero_times_plus_times(cosine(a), sine(b), cosine(b), radius, true).to_bits(),
            )?;
            put(0x30, 0)?;
            put(0x38, 0x1_0000)?;
            put(0x3C, 0xFF00_0000)?;
            put(0x44, LIFE)?;
            put(
                0x40,
                x87::over_int(&effects.kinds.particle_tuning.burst_life, LIFE as i32).to_bits(),
            )?;
        }
    }
    Ok(())
}
