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
//! ring (an entry to take), before anything. Thunk by `six_in_hook!`. Self-test: a ring
//! of the test's, the original and the Rust version at several fillings and values (some
//! unrounded).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::rva;
use crate::marks::blocks::{RING_FREE_RVA, RING_USED_RVA};
use crate::marks::store::{Slot, game_read, game_write};
use crate::math::x87;
use crate::scene::matrix3::rounded;

const FULL: u32 = 0xFFF;
const LOWER_RVA: Slot = Slot::at(0x93C644); // flt_93C644
const VIEW_RVA: usize = rva(0x5EA3CC); // flt_5EA3CC: the view's place at +0x30
const LIFE: u32 = 0xC350;

/// The Rust version (see the header).
pub unsafe fn add_drift_particle(kind: u32, v: [[u8; 10]; 6]) -> Result<(), Refused> {
    unsafe {
        let used: u32 = game_read(RING_USED_RVA);
        if used == FULL {
            return Ok(());
        }
        let free: u32 = game_read(RING_FREE_RVA);
        if free == 0 || (free as *const u32).read_unaligned() == 0 {
            return Err(BadInput("a null particle ring entry").into());
        }
        let e = (free as *const u32).read_unaligned();
        game_write(RING_FREE_RVA, e);
        game_write(RING_USED_RVA, used.wrapping_add(1));
        let put = |o: u32, x: u32| ((e + o) as *mut u32).write_unaligned(x);
        put(8, kind);
        put(0x34, rounded(v[0]));
        put(0x2C, rounded(v[1]));
        put(0x28, rounded(v[2]));
        put(0x14, rounded(v[3]));
        put(0x20, rounded(v[3]));
        put(0x10, rounded(v[4]));
        put(0x1C, x87::ext_sub(v[4], game_read(LOWER_RVA)));
        put(0xC, rounded(v[5]));
        put(0x18, rounded(v[5]));
        put(0x38, 0x8000);
        put(0x3C, 0xFF00_0000);
        put(0x44, LIFE);
        put(0x24, x87::loaded(game_read(VIEW_RVA + 0x34)));
    }
    Ok(())
}
