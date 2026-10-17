//! `marks::emit_particle_21` (0x4CCA58, a chunk of `marks::emit_wheel_particles` entered
//! by `jnz` from 0x4CCCDE when `dword_93A1D6` is not 0, left by `popa; retn`): one ring
//! entry of kind 0x15 for the wheel kind 1, when the rate gate passes.
//!
//! In:  edi the wheel record, esi the car object
//! Out: every register kept
//!
//! The rate (`dword_9356B0`): (1 - 1 / (the wheel's +0xC by `flt_93A312`, `flt_93A316`,
//! plus `flt_93A31A`)) by `flt_93A31E`; then `marks/emitters.rs`' gate. The entry: its
//! place the wheel's (+0x3C ..) plus a random side (-1 or 0 times the car matrix's first
//! row, 0 or 1 times its third), less `flt_93A326` in y at +0x28; its motion per axis
//! the matrix's first row by the wheel's +0x10, its third by +0x18 and by +0xC, each by
//! a factor (`flt_93A32A` ..); a random spin and turn (+0x54, +0x50); +0x58 0x34; a life
//! of 1000 (+0x78 `flt_7651A4` over it); +0x74 `dword_76519C`; the car (+0x68) and its
//! +0x88, +0x90 plus the rate; +0x60 that sum by `flt_93A352` (after a first value by
//! the marks' random, which still steps). Refuses (before anything) a null wheel, a null
//! object, and a null car or matrix when an entry is made. Self-test: a ring, wheel and
//! car of the test's (`marks::emit_particle_6::TestEmitter`), several speeds and fillings.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::marks::emit_particle_6::{TEST_ENTRIES, TestEmitter};
use crate::marks::emitters::*;
use crate::marks::setup::SEED_RVA;
use crate::math::x87;
use crate::scene::matrix3::rounded;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at as at_rva, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::emit_particle_21", rva(0x4CCA58), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const RATE_K: [usize; 4] = [rva(0x93A312), rva(0x93A316), rva(0x93A31A), rva(0x93A31E)];
const SIDE_RANDOM_RVA: usize = rva(0x93A322);
const LIFT_RVA: usize = rva(0x93A326);
/// Per axis (x, y, z): the factors of the first row by +0x10, the third by +0x18 and by
/// +0xC (`flt_93A32A` .. `flt_93A34A`).
const MOTION_K: [[usize; 3]; 3] = [
    [rva(0x93A32A), rva(0x93A32E), rva(0x93A332)],
    [rva(0x93A336), rva(0x93A33A), rva(0x93A33E)],
    [rva(0x93A342), rva(0x93A346), rva(0x93A34A)],
];
const SPIN_RANDOM_RVA: usize = rva(0x93A34E);
const LIFE_SCALE_RVA: usize = rva(0x7651A4); // flt_7651A4
const FADE_RVA: usize = rva(0x76519C); // dword_76519C
const SIZE_K_RVA: usize = rva(0x93A352);
const LOOK: u32 = 0x34;
const LIFE: u32 = 0x3E8;

/// The rate, from the wheel's +0xC.
unsafe fn rate(wheel: u32) -> i32 {
    unsafe {
        let [k1, k2, k3, k4] = RATE_K.map(|r| game_read::<u32>(r));
        x87::speed_rate_to_int(at(wheel + 0xC), k1, k2, k3, game_read(ONE_RVA), k4)
    }
}

/// Why the Rust version refuses these registers, if it does, worked out without writing.
unsafe fn refusal(wheel: u32, object: u32) -> Option<&'static str> {
    unsafe {
        if wheel == 0 {
            return Some("a null wheel");
        }
        if object == 0 {
            return Some("a null object");
        }
        if !gate_with(wheel, object, rate(wheel)).passes {
            return None;
        }
        // The ring full: nothing more is read.
        next_entry()?;
        let car: u32 = game_read(OBJECT_RVA);
        if car == 0 || at(car + 0x24) == 0 {
            return Some("a null car object or matrix");
        }
        None
    }
}

/// The Rust version (see the header).
pub unsafe fn emit_particle_21(wheel: u32, object: u32) -> Result<(), Refused> {
    unsafe {
        if let Some(why) = refusal(wheel, object) {
            return Err(BadInput(why).into());
        }
        let g = gate_with(wheel, object, rate(wheel));
        game_write(RATE_RVA, g.rate);
        if let Some(count) = g.count {
            ((wheel + 4) as *mut u32).write_unaligned(count);
        }
        if !g.passes || next_entry().is_none() {
            return Ok(());
        }
        let e = take_entry(at(object + 0x1C) & 0xFF_FFFF, 0x15);
        let put = |o: u32, v: u32| ((e + o) as *mut u32).write_unaligned(v);
        let car: u32 = game_read(OBJECT_RVA);
        let m = at(car + 0x24);
        let r = next_c5(SIDE_RANDOM_RVA);
        let (a, b) = ((r & 1) as i32 - 1, (r & 1) as i32);
        game_write(SIDES_RVA, [a, b]);
        let place: [[u8; 10]; 3] = std::array::from_fn(|k| {
            let k = 4 * k as u32;
            x87::pair_sum_80(a, at(m + k), b, at(m + 0x20 + k), at(wheel + 0x3C + k))
        });
        put(0x18, rounded(place[2]));
        put(0x14, rounded(place[1]));
        put(0x28, x87::ext_less(place[1], game_read(LIFT_RVA)));
        put(0x10, rounded(place[0]));
        let motion: [u32; 3] = std::array::from_fn(|k| {
            let (first, third) = (at(m + 4 * k as u32), at(m + 0x20 + 4 * k as u32));
            let f = MOTION_K[k].map(|r| game_read::<u32>(r));
            x87::three_products([
                [first, at(wheel + 0x10), f[0]],
                [third, at(wheel + 0x18), f[1]],
                [third, at(wheel + 0xC), f[2]],
            ])
        });
        put(0x34, motion[2]);
        put(0x30, motion[1]);
        put(0x2C, motion[0]);
        let r = next_c5(SPIN_RANDOM_RVA);
        put(0x54, (r & 0x3F).wrapping_sub(0x20));
        put(0x50, r.rotate_right(6) & 0xFFF);
        put(0x58, LOOK);
        let s = game_read::<u32>(SEED_RVA)
            .wrapping_mul(0x9D)
            .wrapping_add(0x47);
        game_write(SEED_RVA, s);
        put(0x60, x87::from_int(((s & 0x3FFF) + 0x4000) as i32));
        put(0x84, LIFE);
        put(0x78, x87::over_int(game_read(LIFE_SCALE_RVA), LIFE as i32));
        put(0x74, game_read(FADE_RVA));
        put(0x68, car);
        let size = at(car + 0x90).wrapping_add(g.rate as u32);
        put(0x70, size);
        put(0x6C, at(car + 0x88));
        put(0x60, x87::int_times(size as i32, game_read(SIZE_K_RVA)));
    }
    Ok(())
}

/// What it may write: the globals, the wheel's +4 and, when one is to be taken, the next
/// entry.
pub fn regions(wheel: u32) -> Vec<Region> {
    let mut r: Vec<Region> = [
        RATE_RVA,
        SIDE_RANDOM_RVA,
        SPIN_RANDOM_RVA,
        SEED_RVA,
        RING_CURSOR_RVA,
        RING_COUNT_RVA,
    ]
    .map(at_rva)
    .to_vec();
    r.push((crate::framework::game_addr(SIDES_RVA) as u32, 8));
    if wheel != 0 {
        r.push((wheel + 4, 4));
    }
    if let Some(e) = unsafe { next_entry() } {
        r.push((e, ENTRY_LEN));
    }
    r
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { emit_particle_21(r.edi, r.esi) }?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(r.edi), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut test = TestEmitter::new(&[SIDE_RANDOM_RVA, SPIN_RANDOM_RVA]);
    let regions = test.regions();
    let mut made = 0;
    let mut case = 0u32;
    // (speeds +0xC, +0x10, +0x18, the object's +0x90, the wheel's +4, taken)
    for (speeds, size, count, taken) in [
        ([3.0f32, 0.5, -2.0], 0x2000u32, 0x5000u32, 0u32),
        ([-10.0, 1.0, 4.0], 0x1800, 0x7000, 5),
        ([0.0, 0.0, 0.0], 0x4000, 0x9000, 0),
        ([1.0, 0.0, 0.0], 0x4000, 0x9000, 0),
        ([2.0, -0.0, 0.0], 0x0800, 0x9000, 0),
        ([5.0, 2.0, 1.0], 0x2000, 0x10, 0),
        ([5.0, 2.0, 1.0], 0x2000, 0x5000, MOST_TAKEN),
        ([0.25, 7.0, -3.0], 0x7FFF_F000, 0x7FFF_FFFF, 1),
        ([40.0, -3.0, 0.5], 0x1000, 0xFFFF_0000, 2),
    ] {
        case += 1;
        let ptrs = (
            test.wheel.as_mut_ptr(),
            test.object.as_mut_ptr(),
            test.matrix.as_mut_ptr(),
        );
        let seed = case.wrapping_mul(0x2545_F491);
        test.prepare(taken);
        check_state(
            &mut t,
            &format!("speeds {speeds:?}, size {size:#x}, count {count:#x}, taken {taken:#x}"),
            call_original,
            Regs {
                edi: ptrs.0 as u32,
                esi: ptrs.1 as u32,
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
                game_write(OBJECT_RVA, ptrs.1 as u32);
                game_write(SEED_RVA, seed);
                game_write(SIDE_RANDOM_RVA, seed ^ 0x55);
                game_write(SPIN_RANDOM_RVA, seed ^ 0xAA);
                let w = ptrs.0;
                for n in 0..0x20 {
                    *w.add(n) = (n as f32 * 0.75 - 3.0).to_bits();
                }
                *w.add(1) = count;
                *w.add(3) = speeds[0].to_bits();
                *w.add(4) = speeds[1].to_bits();
                *w.add(6) = speeds[2].to_bits();
                let o = ptrs.1;
                for n in 0..0x30 {
                    *o.add(n) = (n as f32 * 1.25 + 0.5).to_bits();
                }
                *o.add(0x1C / 4) = 0x1234_5678;
                *o.add(0x24 / 4) = ptrs.2 as u32;
                *o.add(0x88 / 4) = 0x77;
                *o.add(0x90 / 4) = size;
                for n in 0..0x10 {
                    *ptrs.2.add(n) = ((n as f32 - 5.0) * 0.3).to_bits();
                }
            },
            &regions,
            &rust,
        );
        if unsafe { game_read::<u32>(RING_COUNT_RVA) } != taken {
            made += 1;
        }
    }
    t.check(made > 0, || "no case made an entry".into());
    drop(test);
    t.finish()
}
