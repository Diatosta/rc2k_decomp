//! `marks::burst_particles` (0x4D0146): up to 0x100 entries of the marks' ring started at
//! the place of the record at esi (+0xC .. +0x14), each in a random direction on a sphere
//! of a random radius: `dword_5165E8` 1; the radius (the marks' random's low word times
//! `flt_8EB969`, plus `flt_93C674`, times `flt_93C678`) into `flt_93A5E4`, a kind ((its low
//! 3 bits) times 0x400) into `dword_93A5D4` and ((its bits 16) + 8) into `dword_93A5D0`;
//! then each entry taken from `g_marksRingFree` (its +0, while `g_marksRingUsed` is not
//! 0xFFF): +8 `dword_93A5D0`, +0x48 `dword_93A5D4`, the place, a direction by two angles
//! of the next random (`g_sines` / `g_cosines`) times the radius at +0x28 .. +0x34, +0x38
//! 0x10000, +0x3C 0xFF000000, +0x44 3000 and +0x40 `flt_93C67C` / 3000.
//!
//! In:  esi the record
//! Out: nothing (its caller reads no register after it)
//!
//! Callers: `sub_4CFB49` and code at 0x4D032B. Refuses a null record or ring (when an entry
//! is to be taken), before anything.

use crate::errors::{BadInput, Refused};
use crate::marks::blocks::RING_POOL;
use crate::marks::frame_state::DRAW_WORK;
use crate::marks::run_particles::PARTICLE_TUNING;
use crate::marks::work_store::MARKS_WORK;
use crate::math::x87;
use crate::pointers::GamePtr;

const ENTRIES: u32 = 0x100;
const FULL: u32 = 0xFFF;
const LIFE: u32 = 0xBB8;

/// The marks' random: times 0x9D plus 0x47.
fn next_seed() -> u32 {
    let s = MARKS_WORK.seed.get().wrapping_mul(0x9D).wrapping_add(0x47);
    MARKS_WORK.seed.set(s);
    s
}

/// The entries the burst takes, from `g_marksRingFree` along +0.
fn entries_taken() -> Result<Vec<u32>, Refused> {
    Ok({
        let used: u32 = RING_POOL.used.get();
        let count = if used <= FULL {
            (FULL - used).min(ENTRIES)
        } else {
            ENTRIES
        };
        let mut e: u32 = RING_POOL.free.get();
        let mut out = Vec::new();
        for _ in 0..count {
            if e == 0 {
                break;
            }
            e = GamePtr::new(e).read::<u32>("a mark's field")?;
            out.push(e);
        }
        out
    })
}

/// The Rust version (see the header): `record` the particle it bursts from (esi).
pub fn burst_particles(record: u32) -> Result<(), Refused> {
    {
        if RING_POOL.used.get() != FULL {
            if record == 0 {
                return Err(BadInput("a null record").into());
            }
            if entries_taken()?.contains(&0) || RING_POOL.free.get() == 0 {
                return Err(BadInput("a null ring entry").into());
            }
        }
        crate::sound::store::STAGE_SOUNDS.random[1].set(1);
        let s = next_seed();
        let r = s & 0xFFFF;
        let m = DRAW_WORK.uv_k.bits();
        let [c, k] = PARTICLE_TUNING.burst_scale.each_ref().map(|f| f.bits());
        let radius = x87::int_times_plus_times(r as i32, m, c, k);
        RING_POOL.burst_radius.set_bits(radius);
        RING_POOL.kind.set((s & 7) * 0x400);
        RING_POOL.nine.set((s.rotate_right(16) & 1) + 8);
        let sine = |n: u32| crate::scene::engine_tables_store::sine(n as usize).bits();
        let cosine = |n: u32| crate::scene::engine_tables_store::cosine(n as usize).bits();
        for _ in 0..ENTRIES {
            let used: u32 = RING_POOL.used.get();
            if used == FULL {
                continue;
            }
            let e: u32 = GamePtr::new(RING_POOL.free.get()).read::<u32>("a mark's field")?;
            RING_POOL.free.set(e);
            RING_POOL.used.set(used.wrapping_add(1));
            let put = |o: u32, v: u32| GamePtr::new(e + o).write::<u32>(v, "a mark's field");
            put(8, RING_POOL.nine.get())?;
            put(0x48, RING_POOL.kind.get())?;
            for o in [0xC, 0x10, 0x14] {
                put(
                    o,
                    x87::loaded(GamePtr::new(record + o).read::<u32>("a mark's field")?),
                )?;
            }
            let s = next_seed();
            let a = s.rotate_right(16) & 0xFFF;
            let b = s & 0xFFF;
            let radius: u32 = RING_POOL.burst_radius.bits();
            put(0x34, x87::mul(sine(a), radius))?;
            put(
                0x28,
                x87::zero_times_plus_times(cosine(a), cosine(b), sine(b), radius, false),
            )?;
            put(
                0x2C,
                x87::zero_times_plus_times(cosine(a), sine(b), cosine(b), radius, true),
            )?;
            put(0x30, 0)?;
            put(0x38, 0x1_0000)?;
            put(0x3C, 0xFF00_0000)?;
            put(0x44, LIFE)?;
            put(
                0x40,
                x87::over_int(PARTICLE_TUNING.burst_life.bits(), LIFE as i32),
            )?;
        }
    }
    Ok(())
}
