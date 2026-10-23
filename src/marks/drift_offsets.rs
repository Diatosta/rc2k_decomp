//! `marks::drift_offsets` (0x4C1BA9): two whole-number offsets drifting by a wandering
//! pace. When `dword_8BC40D` ticks of `g_sceneClock` have passed since `dword_8BC409`
//! (unsigned), new targets by the marks' random (its low 12 bits and the next 12, less
//! 0x7FF, times `flt_8C5FD1` / `flt_8C5FD5`), a new wait ((its top byte + 0x40) times 5),
//! the time kept, and a second random's halves times 0x100 in `dword_8BC429` /
//! `dword_8BC431`. Then each pace eased toward its target (`x87::ease`, by `flt_8C5FD9`
//! / `flt_8C5FDD`) and added to its offset (`dword_8BDD7D`, `dword_8BDD85`, rounded).
//!
//! In:  nothing
//! Out: eax the ticks passed; after new targets eax, ebx the second random's halves times
//!      0x100, ecx the wait
//!
//! One caller, `sub_4C16F9` (the marks' frame). Self-test: the original and the Rust
//! version, the wait passed and not, at several randoms and paces; put back afterwards.
//!
//! Converted 2026-10-21: the hook's Rust version is `drift_offsets_plain`, through
//! `rc2k_core::marks::drift` (plain floats; the x87 version stays for the Rust callers in
//! unconverted code; the offsets' `fistp` stays the x87's). Shadow mode and the self-test
//! hold the targets and paces to `service::floats`' tolerances and the offsets to one
//! either way (the self-test at both test precisions, exact at double).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{Regs, game_read, game_write, rva};
use crate::marks::setup::{CLOCK_RVA, SEED_RVA};
use crate::math::x87;

const SINCE_RVA: usize = rva(0x8BC409); // dword_8BC409: the clock at the last targets
const WAIT_RVA: usize = rva(0x8BC40D); // dword_8BC40D
const TARGET_RVA: [usize; 2] = [rva(0x8BC411), rva(0x8BC419)]; // flt_8BC411, flt_8BC419
const PACE_RVA: [usize; 2] = [rva(0x8BC41D), rva(0x8BC425)]; // flt_8BC41D, flt_8BC425
const HALVES_RVA: [usize; 2] = [rva(0x8BC429), rva(0x8BC431)]; // dword_8BC429, dword_8BC431
const OFFSET_RVA: [usize; 2] = [rva(0x8BDD7D), rva(0x8BDD85)]; // dword_8BDD7D, dword_8BDD85
const SCALE_RVA: [usize; 2] = [rva(0x8C5FD1), rva(0x8C5FD5)]; // flt_8C5FD1, flt_8C5FD5
const EASE_RVA: [usize; 2] = [rva(0x8C5FD9), rva(0x8C5FDD)]; // flt_8C5FD9, flt_8C5FDD

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
pub unsafe fn drift_offsets(entry: &Regs) -> Regs {
    let mut g = *entry;
    unsafe {
        g.eax = game_read::<u32>(CLOCK_RVA).wrapping_sub(game_read(SINCE_RVA));
        if g.eax >= game_read::<u32>(WAIT_RVA) {
            let s = next_seed();
            g.eax = (s & 0xFFF).wrapping_sub(0x7FF);
            game_write(
                TARGET_RVA[0],
                x87::int_times(g.eax as i32, game_read(SCALE_RVA[0])),
            );
            g.ebx = ((s >> 12) & 0xFFF).wrapping_sub(0x7FF);
            game_write(
                TARGET_RVA[1],
                x87::int_times(g.ebx as i32, game_read(SCALE_RVA[1])),
            );
            g.ecx = ((s >> 24) + 0x40) * 5;
            game_write(WAIT_RVA, g.ecx);
            game_write(SINCE_RVA, game_read::<u32>(CLOCK_RVA));
            let s = next_seed();
            g.eax = (s & 0xFFFF) << 8;
            g.ebx = (s >> 16) << 8;
            game_write(HALVES_RVA[0], g.eax);
            game_write(HALVES_RVA[1], g.ebx);
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
    g
}
