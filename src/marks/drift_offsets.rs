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

use crate::marks::setup::{CLOCK_RVA, SEED_RVA};
use crate::marks::store::{Slot, game_read, game_write};
use crate::math::x87;

const SINCE_RVA: Slot = Slot::at(0x8BC409); // dword_8BC409: the clock at the last targets
const WAIT_RVA: Slot = Slot::at(0x8BC40D); // dword_8BC40D
const TARGET_RVA: [Slot; 2] = [Slot::at(0x8BC411), Slot::at(0x8BC419)]; // flt_8BC411, flt_8BC419
const PACE_RVA: [Slot; 2] = [Slot::at(0x8BC41D), Slot::at(0x8BC425)]; // flt_8BC41D, flt_8BC425
const HALVES_RVA: [Slot; 2] = [Slot::at(0x8BC429), Slot::at(0x8BC431)]; // dword_8BC429, dword_8BC431
const OFFSET_RVA: [Slot; 2] = [Slot::at(0x8BDD7D), Slot::at(0x8BDD85)]; // dword_8BDD7D, dword_8BDD85
const SCALE_RVA: [Slot; 2] = [Slot::at(0x8C5FD1), Slot::at(0x8C5FD5)]; // flt_8C5FD1, flt_8C5FD5
const EASE_RVA: [Slot; 2] = [Slot::at(0x8C5FD9), Slot::at(0x8C5FDD)]; // flt_8C5FD9, flt_8C5FDD

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
    unsafe {
        let since = game_read::<u32>(CLOCK_RVA).wrapping_sub(game_read(SINCE_RVA));
        if since >= game_read::<u32>(WAIT_RVA) {
            let s = next_seed();
            let x = (s & 0xFFF).wrapping_sub(0x7FF);
            game_write(
                TARGET_RVA[0],
                x87::int_times(x as i32, game_read(SCALE_RVA[0])),
            );
            let y = ((s >> 12) & 0xFFF).wrapping_sub(0x7FF);
            game_write(
                TARGET_RVA[1],
                x87::int_times(y as i32, game_read(SCALE_RVA[1])),
            );
            game_write(WAIT_RVA, ((s >> 24) + 0x40) * 5);
            game_write(SINCE_RVA, game_read::<u32>(CLOCK_RVA));
            let s = next_seed();
            game_write(HALVES_RVA[0], (s & 0xFFFF) << 8);
            game_write(HALVES_RVA[1], (s >> 16) << 8);
        }
        for n in 0..2 {
            let pace = x87::ease(
                game_read(TARGET_RVA[n]),
                game_read(PACE_RVA[n]),
                game_read(EASE_RVA[n]),
            );
            game_write(PACE_RVA[n], pace);
            let offset = x87::int_plus_to_int(game_read(OFFSET_RVA[n]), pace);
            game_write(OFFSET_RVA[n], offset);
        }
    }
}
