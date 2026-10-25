//! `marks::drift_offsets` (0x4C1BA9): two whole-number offsets drifting by a wandering
//! pace. When `dword_8BC40D` ticks of `g_sceneClock` have passed since `dword_8BC409`
//! (unsigned), new targets by the marks' random (its low 12 bits and the next 12, less
//! 0x7FF, times `flt_8C5FD1` / `flt_8C5FD5`), a new wait ((its top byte + 0x40) times 5),
//! the time kept, and a second random's halves times 0x100 in `dword_8BC429` /
//! `dword_8BC431`. Then each pace eased toward its target (`x87::ease`, by `flt_8C5FD9`
//! / `flt_8C5FDD`) and added to its offset (`dword_8BDD7D`, `dword_8BDD85`, rounded).
//!
//! In:  nothing
//! Out: nothing (its caller reads no register after it)
//!
//! One caller, `sub_4C16F9` (the marks' frame).
//!
//! the x87 version stays for the Rust callers in unconverted code; the offsets' `fistp`
//! stays the x87's).

use crate::marks::rain::{RAIN, RAIN_RULES};
use crate::marks::setup::SEED_RVA;
use crate::marks::store::{game_read, game_write};
use crate::math::x87;
use crate::physics::tables_store::PHYSICS;

/// The marks' random: times 0x9D plus 0x47.
unsafe fn next_seed() -> u32 {
    unsafe {
        let s = game_read::<u32>(SEED_RVA)
            .wrapping_mul(0x9D)
            .wrapping_add(0x47);
        game_write(SEED_RVA, s);
        s
    }
}

/// The Rust version (see the header).
pub unsafe fn drift_offsets() {
    let (r, k) = (&RAIN, &RAIN_RULES);
    unsafe {
        let since = PHYSICS.scene_clock.bits().wrapping_sub(r.drift_since.get());
        if since >= r.drift_wait.get() {
            let s = next_seed();
            let x = (s & 0xFFF).wrapping_sub(0x7FF);
            r.drift_target[0].set_bits(x87::int_times(x as i32, k.target_scale[0].bits()));
            let y = ((s >> 12) & 0xFFF).wrapping_sub(0x7FF);
            r.drift_target[2].set_bits(x87::int_times(y as i32, k.target_scale[1].bits()));
            r.drift_wait.set(((s >> 24) + 0x40) * 5);
            r.drift_since.set(PHYSICS.scene_clock.bits());
            let s = next_seed();
            r.drift_halves[0].set((s & 0xFFFF) << 8);
            r.drift_halves[2].set((s >> 16) << 8);
        }
        for (n, at) in [0, 2].into_iter().enumerate() {
            let pace = x87::ease(
                r.drift_target[at].bits(),
                r.drift_pace[at].bits(),
                k.ease[n].bits(),
            );
            r.drift_pace[at].set_bits(pace);
            let offset = x87::int_plus_to_int(r.view[at].get() as i32, pace);
            r.view[at].set(offset as u32);
        }
    }
}
