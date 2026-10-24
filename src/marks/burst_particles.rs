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
//! Callers: `sub_4CFB49` and code at 0x4D032B. Refuses a null record or ring (when an
//! entry is to be taken), before anything. Self-test: a ring of the test's, the original
//! and the Rust version at several fillings and randoms; put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::rva;
use crate::marks::blocks::{RING_FREE_RVA, RING_USED_RVA};
use crate::marks::setup::SEED_RVA;
use crate::marks::store::{Slot, game_read, game_write};
use crate::math::x87;
use crate::sound::store::StageSlot;

const FLAG_RVA: StageSlot = StageSlot::at(0x5165E8); // dword_5165E8
const RADIUS_RVA: Slot = Slot::at(0x93A5E4); // g_marksBurstRadius
const KIND_RVA: Slot = Slot::at(0x93A5D4); // dword_93A5D4
const NINE_RVA: Slot = Slot::at(0x93A5D0); // dword_93A5D0
const SCALE_RVA: [Slot; 3] = [Slot::at(0x8EB969), Slot::at(0x93C674), Slot::at(0x93C678)];
const LIFE_RVA: Slot = Slot::at(0x93C67C); // flt_93C67C
const SINES_RVA: usize = rva(0xA23AE4); // g_sines (g_cosines 0x400 further)
const COSINES_RVA: usize = rva(0xA24AE4); // g_cosines
const ENTRIES: u32 = 0x100;
const FULL: u32 = 0xFFF;
const LIFE: u32 = 0xBB8;

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

/// The entries the burst takes, from `g_marksRingFree` along +0.
unsafe fn entries_taken() -> Vec<u32> {
    unsafe {
        let used: u32 = game_read(RING_USED_RVA);
        let count = if used <= FULL {
            (FULL - used).min(ENTRIES)
        } else {
            ENTRIES
        };
        let mut e: u32 = game_read(RING_FREE_RVA);
        let mut out = Vec::new();
        for _ in 0..count {
            if e == 0 {
                break;
            }
            e = (e as *const u32).read_unaligned();
            out.push(e);
        }
        out
    }
}

/// The Rust version (see the header): `record` the particle it bursts from (esi).
pub unsafe fn burst_particles(record: u32) -> Result<(), Refused> {
    unsafe {
        if game_read::<u32>(RING_USED_RVA) != FULL {
            if record == 0 {
                return Err(BadInput("a null record").into());
            }
            if entries_taken().contains(&0) || game_read::<u32>(RING_FREE_RVA) == 0 {
                return Err(BadInput("a null ring entry").into());
            }
        }
        crate::store::game_write(FLAG_RVA, 1u32);
        let s = next_seed();
        let r = s & 0xFFFF;
        let [m, c, k] = SCALE_RVA.map(|r| game_read::<u32>(r));
        let radius = x87::int_times_plus_times(r as i32, m, c, k);
        game_write(RADIUS_RVA, radius);
        game_write(KIND_RVA, (s & 7) * 0x400);
        game_write(NINE_RVA, (s.rotate_right(16) & 1) + 8);
        let sine = |n: u32| game_read::<u32>(SINES_RVA + 4 * n as usize);
        let cosine = |n: u32| game_read::<u32>(COSINES_RVA + 4 * n as usize);
        for _ in 0..ENTRIES {
            let used: u32 = game_read(RING_USED_RVA);
            if used == FULL {
                continue;
            }
            let e: u32 = (game_read::<u32>(RING_FREE_RVA) as *const u32).read_unaligned();
            game_write(RING_FREE_RVA, e);
            game_write(RING_USED_RVA, used.wrapping_add(1));
            let put = |o: u32, v: u32| ((e + o) as *mut u32).write_unaligned(v);
            put(8, game_read(NINE_RVA));
            put(0x48, game_read(KIND_RVA));
            for o in [0xC, 0x10, 0x14] {
                put(
                    o,
                    x87::loaded(((record + o) as *const u32).read_unaligned()),
                );
            }
            let s = next_seed();
            let a = s.rotate_right(16) & 0xFFF;
            let b = s & 0xFFF;
            let radius: u32 = game_read(RADIUS_RVA);
            put(0x34, x87::mul(sine(a), radius));
            put(
                0x28,
                x87::zero_times_plus_times(cosine(a), cosine(b), sine(b), radius, false),
            );
            put(
                0x2C,
                x87::zero_times_plus_times(cosine(a), sine(b), cosine(b), radius, true),
            );
            put(0x30, 0);
            put(0x38, 0x1_0000);
            put(0x3C, 0xFF00_0000);
            put(0x44, LIFE);
            put(0x40, x87::over_int(game_read(LIFE_RVA), LIFE as i32));
        }
    }
    Ok(())
}
