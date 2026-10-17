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

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::marks::setup::{CLOCK_RVA, SEED_RVA};
use crate::math::x87;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::drift_offsets", rva(0x4C1BA9), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

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

fn regions() -> Vec<Region> {
    let mut r: Vec<Region> = [SINCE_RVA, WAIT_RVA, SEED_RVA].map(at).to_vec();
    for set in [TARGET_RVA, PACE_RVA, HALVES_RVA, OFFSET_RVA] {
        r.extend(set.map(at));
    }
    r
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { drift_offsets(r) }))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

/// clock, since, wait, seed, paces, offsets, eases.
type Case = (u32, u32, u32, u32, [f32; 2], [i32; 2], [f32; 2]);

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    let mut inputs = regions.clone();
    inputs.extend(
        [
            CLOCK_RVA,
            EASE_RVA[0],
            EASE_RVA[1],
            SCALE_RVA[0],
            SCALE_RVA[1],
        ]
        .map(at),
    );
    let saved = capture(&inputs);
    let cases: [Case; 6] = [
        (
            1000,
            900,
            100,
            0x1234_5678,
            [0.5, -0.25],
            [10, -10],
            [0.1, 0.2],
        ),
        (
            1000,
            901,
            100,
            0x1234_5678,
            [0.5, -0.25],
            [10, -10],
            [0.1, 0.2],
        ),
        (
            5,
            0xFFFF_FFF0,
            0x16,
            0xDEAD_BEEF,
            [3.7, 1e6],
            [0x7FFF_0000, -3],
            [0.9, 1.0],
        ),
        (
            5,
            0xFFFF_FFF0,
            0x15,
            0xDEAD_BEEF,
            [-2.5, 2.5],
            [0, 0],
            [0.5, 0.5],
        ),
        (0, 0, 0, 0, [0.0, -0.0], [1, 2], [0.03, 0.07]),
        (
            77,
            7,
            0xFFFF_FFFF,
            0xFFFF_FFFF,
            [1.5, -1.5],
            [-7, 7],
            [0.25, 0.75],
        ),
    ];
    for (clock, since, wait, seed, paces, offsets, eases) in cases {
        check_state(
            &mut t,
            &format!(
                "clock {clock:#x}, since {since:#x}, wait {wait:#x}, seed {seed:#x}, paces {paces:?}"
            ),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&inputs, &saved);
                game_write(CLOCK_RVA, clock);
                game_write(SINCE_RVA, since);
                game_write(WAIT_RVA, wait);
                game_write(SEED_RVA, seed);
                for n in 0..2 {
                    game_write(PACE_RVA[n], paces[n].to_bits());
                    game_write(OFFSET_RVA[n], offsets[n]);
                    game_write(EASE_RVA[n], eases[n].to_bits());
                    game_write(TARGET_RVA[n], (n as f32 * 3.0 - 1.0).to_bits());
                }
            },
            &regions,
            &rust,
        );
    }
    restore(&inputs, &saved);
    t.finish()
}
