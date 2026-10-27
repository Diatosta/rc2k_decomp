//! `marks::add_drift_particle` (0x4CFF57): one entry of the marks' particle ring
//! (`g_marksRingFree`'s +0, while `g_marksRingUsed` is not 0xFFF) started from the six
//! values on the x87 stack: +8 eax (the kind), st(0) .. st(2) at +0x34, +0x2C, +0x28 (the
//! motion), st(3) at +0x14 and +0x20, st(4) at +0x10 and (less `flt_93C644`) +0x1C, st(5)
//! at +0xC and +0x18 (the place, now and last); +0x38 0x8000, +0x3C 0xFF000000, +0x44
//! 50000, +0x24 the view's +0x34 (`flt_5EA3CC`). With the ring full the six are dropped.
//!
//! In:  st(0) .. st(5) as above (all popped), eax the kind
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `sub_4CFB49` (every eighth marks step, a drift puff by the view). Refuses a null
//! ring (an entry to take), before anything.

use crate::errors::{BadInput, Refused};
use crate::marks::blocks::{RING_POOL, particle, set_particle};
use crate::marks::run_particles::PARTICLE_TUNING;
use crate::math::x87;
use crate::race::matrices_store::MATRICES;
use crate::scene::matrix3::rounded;

const FULL: u32 = 0xFFF;
const LIFE: u32 = 0xC350;

/// The Rust version (see the header).
pub fn add_drift_particle(kind: u32, v: [[u8; 10]; 6]) -> Result<(), Refused> {
    {
        let used: u32 = RING_POOL.used.get();
        if used == FULL {
            return Ok(());
        }
        let free: u32 = RING_POOL.free.get();
        if free == 0 || particle(free, 0)? == 0 {
            return Err(BadInput("a null particle ring entry").into());
        }
        let e = particle(free, 0)?;
        RING_POOL.free.set(e);
        RING_POOL.used.set(used.wrapping_add(1));
        let put = |o: u32, x: u32| set_particle(e, o, x);
        put(8, kind)?;
        put(0x34, rounded(v[0]))?;
        put(0x2C, rounded(v[1]))?;
        put(0x28, rounded(v[2]))?;
        put(0x14, rounded(v[3]))?;
        put(0x20, rounded(v[3]))?;
        put(0x10, rounded(v[4]))?;
        put(0x1C, x87::ext_sub(v[4], PARTICLE_TUNING.drift_lower.bits()))?;
        put(0xC, rounded(v[5]))?;
        put(0x18, rounded(v[5]))?;
        put(0x38, 0x8000)?;
        put(0x3C, 0xFF00_0000)?;
        put(0x44, LIFE)?;
        put(0x24, x87::loaded(MATRICES.view_place[13].bits()))?;
    }
    Ok(())
}
