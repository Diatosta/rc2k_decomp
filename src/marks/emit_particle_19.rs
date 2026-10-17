//! `marks::emit_particle_19` (0x4CE283): by a chance one ring entry of kind 0x13 for a
//! car each frame; `marks::emit_particle_22` (0x4CE482) is the same code on its own
//! variables (`TwinSet`).
//!
//! In:  esi the car object
//! Out: every register kept (`pusha` .. `popa`)
//!
//! The level (`dword_93A1CE`) the car's +0x4C0 (+0x4C4); a random's bits 16 .. 18 must be
//! under it (signed). `g_marksWheelFlags` 0x3E (0x33); `dword_93A1C2` .. the car's +0x4C
//! .. +0x54. The entry: its place `flt_93A1B6` .. turned and placed by `g_marksCar`'s
//! matrix (`x87::turned_place`), less `flt_93A48E` in y at +0x28; its motion the matrix's
//! second row by `flt_93A492` .. moved by `marks::jitter_point` (spread `flt_93A49E`) plus
//! `dword_93A1C2` .. by `flt_764FE0` and `flt_93A4A2` ..; a random spin and turn; the
//! flags' low 6 bits; a size 0x4000 or 0x8000 by the random's bit 6 (`dword_9356C0`, +0x60
//! as a float); a life `dword_939B86[level]` (+0x78 `flt_7651A4` over it); +0x74
//! `dword_76519C`; +0x68 `g_marksCar`. Caller: `sub_4C9AF6` (the cars' loop). Refuses
//! (before anything) a null object, and when an entry is made a null `g_marksCar` or
//! matrix and a level past `dword_939B86`'s 9 entries (the original reads on). Self-test:
//! a ring and car of the test's (`marks::emit_particle_6::TestEmitter`), several levels,
//! motions and fillings.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::marks::emit_particle_6::{TEST_ENTRIES, TestEmitter};
use crate::marks::emitters::*;
use crate::marks::jitter_point::jitter_point;
use crate::marks::setup::SEED_RVA;
use crate::math::x87;
use crate::scene::matrix3::{extended, rounded};
use crate::screens::harness::{CallOriginal, Left};
use crate::service::harness::{Region, Rust, at as at_rva, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::emit_particle_19", rva(0x4CE283), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const LEVEL_RVA: usize = rva(0x93A1CE); // dword_93A1CE
const MOTION_RVA: usize = rva(0x93A1C2); // dword_93A1C2 .. dword_93A1CA
const POINT_RVA: usize = rva(0x93A1B6); // flt_93A1B6 .. flt_93A1BE
const STEP_RVA: usize = rva(0x764FE0); // flt_764FE0
const SIZE_RVA: usize = rva(0x9356C0); // dword_9356C0
const LIVES_RVA: usize = rva(0x939B86); // dword_939B86: by the level
const LEVELS: u32 = 9;
const LIFE_SCALE_RVA: usize = rva(0x7651A4); // flt_7651A4
const FADE_RVA: usize = rva(0x76519C); // dword_76519C

/// One twin's variables.
pub struct TwinSet {
    pub kind: u32,
    /// The car's field holding the level.
    pub level_at: u32,
    pub flags: u32,
    pub chance_random: usize,
    pub lift: usize,
    /// The second row's factors (z, y, x: +0x18, +0x14, +0x10), then the spread.
    pub row_k: [usize; 3],
    pub spread: usize,
    /// The motion's factors (x, y, z).
    pub motion_k: [usize; 3],
    pub spin_random: usize,
}

/// `emit_particle_19`'s.
pub const NINETEENTH: TwinSet = TwinSet {
    kind: 0x13,
    level_at: 0x4C0,
    flags: 0x3E,
    chance_random: rva(0x93A48A),
    lift: rva(0x93A48E),
    row_k: [rva(0x93A492), rva(0x93A496), rva(0x93A49A)],
    spread: rva(0x93A49E),
    motion_k: [rva(0x93A4A2), rva(0x93A4A6), rva(0x93A4AA)],
    spin_random: rva(0x93A4AE),
};

/// Whether an entry is to be made (the chance and a free entry), worked out without
/// writing.
unsafe fn makes(set: &TwinSet, object: u32) -> bool {
    unsafe {
        let level = at(object + set.level_at) as i32;
        let r = game_read::<u32>(set.chance_random)
            .wrapping_mul(0xC5)
            .wrapping_add(0x5D);
        ((r.rotate_right(16) & 7) as i32) < level && next_entry().is_some()
    }
}

/// Why the Rust version refuses, if it does, worked out without writing.
pub unsafe fn refusal(set: &TwinSet, object: u32) -> Option<&'static str> {
    unsafe {
        if object == 0 {
            return Some("a null object");
        }
        if !makes(set, object) {
            return None;
        }
        let car: u32 = game_read(OBJECT_RVA);
        if car == 0 || at(car + 0x24) == 0 {
            return Some("a null g_marksCar or matrix");
        }
        if at(object + set.level_at) >= LEVELS {
            return Some("a level past dword_939B86's 9 entries");
        }
        None
    }
}

/// The Rust version (see the header).
pub unsafe fn emit_particle_19(object: u32) -> Result<(), Refused> {
    unsafe { emit_twin(&NINETEENTH, object) }
}

/// The shared body, on `set`'s variables.
pub unsafe fn emit_twin(set: &TwinSet, object: u32) -> Result<(), Refused> {
    unsafe {
        if let Some(why) = refusal(set, object) {
            return Err(BadInput(why).into());
        }
        let level = at(object + set.level_at);
        game_write(LEVEL_RVA, level);
        let r = next_c5(set.chance_random);
        if (r.rotate_right(16) & 7) as i32 >= level as i32 {
            return Ok(());
        }
        game_write(FLAGS_RVA, set.flags);
        let motion = [at(object + 0x4C), at(object + 0x50), at(object + 0x54)];
        game_write(MOTION_RVA, motion);
        if next_entry().is_none() {
            return Ok(());
        }
        let e = take_entry(at(object + 0x1C) & 0xFF_FFFF, set.kind);
        let put = |o: u32, v: u32| ((e + o) as *mut u32).write_unaligned(v);
        let car: u32 = game_read(OBJECT_RVA);
        let m = at(car + 0x24);
        let [x, y, y_less, z] = x87::turned_place(game_read(POINT_RVA), game_read(set.lift), m);
        put(0x10, x);
        put(0x14, y);
        put(0x28, y_less);
        put(0x18, z);
        let row = [0x18u32, 0x14, 0x10]
            .map(|o| at(m + o))
            .into_iter()
            .zip(set.row_k)
            .map(|(a, k)| x87::mul_80(a, game_read(k)))
            .collect::<Vec<_>>();
        let (moved, _) = jitter_point([extended(game_read(set.spread)), row[2], row[1], row[0]]);
        let step: u32 = game_read(STEP_RVA);
        for n in 0..3 {
            put(
                0x2C + 4 * n as u32,
                x87::product_plus(
                    motion[n],
                    step,
                    game_read(set.motion_k[n]),
                    rounded(moved[n]),
                ),
            );
        }
        let r = next_c5(set.spin_random);
        put(0x54, (r & 0x3F).wrapping_sub(0x20));
        put(0x50, r.rotate_right(6) & 0xFFF);
        put(0x58, set.flags & 0x3F);
        let size = (r.rotate_right(0x14) & 0x4000) + 0x4000;
        game_write(SIZE_RVA, size);
        put(0x60, x87::from_int(size as i32));
        let life: u32 = game_read(LIVES_RVA + 4 * level as usize);
        put(0x84, life);
        put(0x78, x87::over_int(game_read(LIFE_SCALE_RVA), life as i32));
        put(0x74, game_read(FADE_RVA));
        put(0x68, car);
    }
    Ok(())
}

/// The globals it may write.
pub fn globals(set: &TwinSet) -> Vec<Region> {
    let mut r: Vec<Region> = [
        LEVEL_RVA,
        set.chance_random,
        FLAGS_RVA,
        SIZE_RVA,
        set.spin_random,
        SEED_RVA,
        RING_CURSOR_RVA,
        RING_COUNT_RVA,
    ]
    .map(at_rva)
    .to_vec();
    r.push((crate::framework::game_addr(MOTION_RVA) as u32, 12));
    r
}

/// What it may write: the globals and, when one is to be taken, the next entry.
pub fn regions(set: &TwinSet) -> Vec<Region> {
    let mut r = globals(set);
    if let Some(e) = unsafe { next_entry() } {
        r.push((e, ENTRY_LEN));
    }
    r
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { emit_particle_19(r.esi) }?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(&NINETEENTH), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    test_with(&mut t, &NINETEENTH, call_original, &rust);
    t.finish()
}

/// The test of either twin.
pub fn test_with(t: &mut SelfTest, set: &TwinSet, call_original: CallOriginal, rust: Rust) {
    let mut test = TestEmitter::new(&[
        LEVEL_RVA,
        set.chance_random,
        SIZE_RVA,
        set.spin_random,
        MOTION_RVA,
        MOTION_RVA + 4,
        MOTION_RVA + 8,
    ]);
    let mut regions = test.regions();
    // The car's level sits past the test object: a car of its own, 0x500 bytes.
    let mut car = vec![0u32; 0x140];
    regions.push((car.as_ptr() as u32, 4 * car.len()));
    let mut made = 0;
    let mut case = 0u32;
    // (the level, the car's +0x4C, taken)
    for (level, speed, taken) in [
        (0u32, 1.0f32, 0u32),
        (8, 30.0, 3),
        (4, -2.5, 0),
        (1, 1e5, 1),
        (7, 0.0, MOST_TAKEN),
        (8, f32::NAN, 0),
        (0xFFFF_FFFF, 3.0, 0),
    ] {
        case += 1;
        let (c, m) = (car.as_mut_ptr(), test.matrix.as_mut_ptr());
        let seed = case.wrapping_mul(0x2545_F491);
        test.prepare(taken);
        check_state(
            t,
            &format!("level {level:#x}, speed {speed}, taken {taken:#x}"),
            call_original,
            Regs {
                esi: c as u32,
                ..Regs::SENTINEL
            },
            &|| unsafe {
                restore(&regions, &vec![0xAA; regions.iter().map(|r| r.1).sum()]);
                let base = test.ring.as_ptr() as u32;
                for n in 0..TEST_ENTRIES {
                    let e = (base + n * ENTRY_LEN as u32) as *mut u32;
                    e.write_unaligned(base + ((n + 1) % TEST_ENTRIES) * ENTRY_LEN as u32);
                }
                game_write(RING_CURSOR_RVA, base + 3 * ENTRY_LEN as u32);
                game_write(RING_COUNT_RVA, taken);
                game_write(OBJECT_RVA, c as u32);
                game_write(SEED_RVA, seed);
                game_write(set.chance_random, seed ^ 0x33);
                game_write(set.spin_random, seed ^ 0x55);
                for n in 0..0x140 {
                    *c.add(n) = (n as f32 * 0.5 - 20.0).to_bits();
                }
                *c.add(0x1C / 4) = 0x1234_5678;
                *c.add(0x24 / 4) = m as u32;
                *c.add(0x4C / 4) = speed.to_bits();
                *c.add(set.level_at as usize / 4) = level;
                for n in 0..0x10 {
                    *m.add(n) = ((n as f32 - 5.0) * 0.3).to_bits();
                }
            },
            &regions,
            rust,
        );
        if unsafe { game_read::<u32>(RING_COUNT_RVA) } != taken {
            made += 1;
        }
    }
    t.check(made > 0, || "no case made an entry".into());
    drop(test);
    drop(car);
}
