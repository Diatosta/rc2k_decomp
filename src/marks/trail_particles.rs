//! `marks::trail_particles` (0x4CFFE3): eight entries of the marks' particle ring
//! (`g_marksRingFree`'s +0, each while `g_marksRingUsed` is not 0xFFF) started along the
//! way the particle at esi came: for each, kind 4; `g_marksBurstRadius` = the marks'
//! random's low word times `flt_8EB969` times `flt_93C648`; the place (+0xC .. +0x14) the
//! particle's place moved that share toward its last place (+0x18 .. +0x20); the motion
//! (+0x28, +0x2C, +0x34) (last - place) times `flt_93C64C` / 54 / 5C plus the particle's
//! motion times `flt_93C650` / 58 / 60, moved by `marks::jitter_point` (spread
//! `flt_93C664`); +0x38 the particle's +0x2C times `flt_93C668` plus `flt_93C66C`
//! (`fistp`), +0x3C 0xFF000000, +0x44 300, +0x40 `flt_93C670` / 300, +0x24 the view's
//! +0x34.
//!
//! In:  esi the particle
//! Out: nothing (no caller reads a register it leaves but esi, which it keeps)
//!
//! Caller: the ring's update (code at 0x4D033F, kind 0 particles moving up). Refuses a null
//! particle or ring (an entry to take), before anything.

use crate::errors::{BadInput, Refused};
use crate::marks::blocks::RING_POOL;
use crate::marks::frame_state::DRAW_WORK;
use crate::marks::jitter_point::jitter_point;
use crate::marks::run_particles::PARTICLE_TUNING;
use crate::marks::work_store::MARKS_WORK;
use crate::math::x87;
use crate::pointers::GamePtr;
use crate::race::matrices_store::MATRICES;
use crate::scene::matrix3::{extended, rounded};

const FULL: u32 = 0xFFF;
const COUNT: u32 = 8;
/// Per axis (place offset, last offset, motion offset); its two factors are
/// `PARTICLE_TUNING.trail_axes[2 n]` and `[2 n + 1]`.
const AXES: [(u32, u32, u32); 3] = [(0xC, 0x18, 0x28), (0x10, 0x1C, 0x2C), (0x14, 0x20, 0x34)];
const LIFE: u32 = 300;

fn get(a: u32) -> Result<u32, BadInput> {
    GamePtr::new(a).read("a game field")
}

/// The entries it takes, from `g_marksRingFree` along +0.
fn entries_taken() -> Result<Vec<u32>, Refused> {
    Ok({
        let used: u32 = RING_POOL.used.get();
        let n = if used <= FULL {
            (FULL - used).min(COUNT)
        } else {
            COUNT
        };
        let mut e: u32 = RING_POOL.free.get();
        let mut out = Vec::new();
        for _ in 0..n {
            if e == 0 {
                break;
            }
            e = get(e)?;
            out.push(e);
        }
        out
    })
}

/// The Rust version (see the header): `p` the particle it trails (esi).
pub fn trail_particles(p: u32) -> Result<(), Refused> {
    {
        if RING_POOL.used.get() != FULL {
            if p == 0 {
                return Err(BadInput("a null particle").into());
            }
            if RING_POOL.free.get() == 0 || entries_taken()?.contains(&0) {
                return Err(BadInput("a null particle ring entry").into());
            }
        }
        for _ in 0..COUNT {
            let used: u32 = RING_POOL.used.get();
            if used == FULL {
                continue;
            }
            let e = get(RING_POOL.free.get())?;
            RING_POOL.free.set(e);
            RING_POOL.used.set(used.wrapping_add(1));
            let put = |o: u32, v: u32| GamePtr::new(e + o).write::<u32>(v, "a mark's field");
            put(8, 4)?;
            let s = MARKS_WORK.seed.get().wrapping_mul(0x9D).wrapping_add(0x47);
            MARKS_WORK.seed.set(s);
            let k1 = DRAW_WORK.uv_k.bits();
            let k2 = PARTICLE_TUNING.trail_random_k.bits();
            RING_POOL
                .burst_radius
                .set_bits(x87::int_times_times((s & 0xFFFF) as i32, k1, k2));
            let radius: u32 = RING_POOL.burst_radius.bits();
            for (place, last, _) in AXES {
                put(place, x87::toward(get(p + place)?, get(p + last)?, radius))?;
            }
            for (n, (place, last, motion)) in AXES.into_iter().enumerate() {
                put(
                    motion,
                    x87::diff_times_plus_times(
                        get(p + last)?,
                        get(p + place)?,
                        PARTICLE_TUNING.trail_axes[2 * n].bits(),
                        get(p + motion)?,
                        PARTICLE_TUNING.trail_axes[2 * n + 1].bits(),
                    ),
                )?;
            }
            let (moved, _) = jitter_point([
                extended(PARTICLE_TUNING.trail_spread.bits()),
                extended(get(e + 0x34)?),
                extended(get(e + 0x2C)?),
                extended(get(e + 0x28)?),
            ]);
            put(0x34, rounded(moved[0]))?;
            put(0x2C, rounded(moved[1]))?;
            put(0x28, rounded(moved[2]))?;
            let [a, b] = PARTICLE_TUNING.trail_size_k.each_ref().map(|f| f.bits());
            put(0x38, x87::times_plus_to_int(a, get(p + 0x2C)?, b) as u32)?;
            put(0x3C, 0xFF00_0000)?;
            put(0x44, LIFE)?;
            put(
                0x40,
                x87::over_int(PARTICLE_TUNING.trail_life_k.bits(), LIFE as i32),
            )?;
            put(0x24, x87::loaded(MATRICES.view_place[13].bits()))?;
        }
    }
    Ok(())
}
