//! `effects::burst_particles`: up to 0x100 entries of the effects' ring started at the place of the
//! record (its three place coordinates), each in a random direction on a sphere of a random radius:
//! a burst flag is set; the radius (the effects' random's low word times a constant, plus a base,
//! times a scale) is kept, and so are a kind ((its low 3 bits) times 0x400) and ((its bit 16) + 8);
//! then each entry taken from the ring's first free one (its link to the next, while the used count
//! is not 0xFFF): the kind the first kept value, the colours' place the second, the place, the
//! motion a direction by two angles of the next random (the sine and cosine tables) times the
//! radius with the lift 0, the size 0x10000, the colour 0xFF000000, the life 3000 and the colour's
//! step the life factor / 3000.
//!
//! Refuses a null record or ring (when an entry is to be taken), before anything.

use crate::effects::blocks::{read_particle, update_particle};
use crate::effects::particles_store::Particles;
use crate::effects::ring_store::Ring;
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

/// The entries the burst takes, from the ring's first free one along the links to the next.
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
            e = read_particle(particles, e)?.before;
            out.push(e);
        }
        out
    })
}

/// `record` is the particle it bursts from.
pub(super) fn burst_particles(
    engine_tables: &crate::math::engine_tables::EngineTables,
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
        sound.race.stage.get().borrow_mut().random[1] = 1;
        let s = next_seed(effects);
        let r = s & 0xFFFF;
        let m = effects.frame.draw_work.borrow().uv_k;
        let [c, k] = effects
            .kinds
            .particle_tuning
            .burst_scale
            .each_ref()
            .map(crate::memory::session::PlainFloat::get);
        let radius = bh::stf((bh::int(r as i32) * bh::ld(m) + bh::ld(c)) * bh::ld(k));
        ring.pool.burst_radius.set(radius);
        ring.pool.kind.set((s & 7) * 0x400);
        ring.pool.nine.set((s.rotate_right(16) & 1) + 8);
        let sine = |n: u32| crate::math::engine_tables::sine(engine_tables, n as usize).get();
        let cosine = |n: u32| crate::math::engine_tables::cosine(engine_tables, n as usize).get();
        for _ in 0..ENTRIES {
            let used: u32 = ring.pool.used.get();
            if used == FULL {
                continue;
            }
            let e: u32 = read_particle(particles, ring.pool.free.get())?.before;
            ring.pool.free.set(e);
            ring.pool.used.set(used.wrapping_add(1));
            let (kind, colours_place) = (ring.pool.nine.get(), ring.pool.kind.get());
            update_particle(particles, e, |p| {
                p.kind = kind;
                p.colours_place = colours_place;
            })?;
            for axis in 0..3 {
                let from = x87::loaded(read_particle(particles, record)?.place(axis)).to_bits();
                update_particle(particles, e, |p| p.set_place(axis, from))?;
            }
            let s = next_seed(effects);
            let a = s.rotate_right(16) & 0xFFF;
            let b = s & 0xFFF;
            let radius = ring.pool.burst_radius.get();
            let motion_z = x87::mul(sine(a), radius).to_bits();
            let motion_x =
                x87::zero_times_plus_times(cosine(a), cosine(b), sine(b), radius, false).to_bits();
            let motion_y =
                x87::zero_times_plus_times(cosine(a), sine(b), cosine(b), radius, true).to_bits();
            let colour_step =
                x87::over_int(&effects.kinds.particle_tuning.burst_life, LIFE as i32).to_bits();
            update_particle(particles, e, |p| {
                p.motion_z = motion_z;
                p.motion_x = motion_x;
                p.motion_y = motion_y;
                p.lift = 0;
                p.size = 0x1_0000;
                p.colour = 0xFF00_0000;
                p.life = LIFE;
                p.colour_step = colour_step;
            })?;
        }
    }
    Ok(())
}
