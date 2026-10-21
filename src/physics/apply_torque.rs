//! `physics::apply_torque` (0x45FB82): one force record (esi: the force at +0, the point
//! it acts at at +0xC) turning a car (edi, its block): the car's spin (+0x184 .. +0x18C)
//! copied to `dword_6D30B4` .. and turned into the car's frame
//! (`physics::turn_by_inverse_at`); the cross product of the record's two vectors
//! (`scene::cross_product`, into `flt_614274` ..); each part of the spin less that part
//! times the step `flt_61388C` times `flt_613FA8 + 4k` (`fld c; fmul; fmul; fchs; fadd;
//! fstp`); turned back into the world (`physics::turn_by_matrix_at`) and stored as the
//! spin. All in `pusha` / `popa` (eax pushed besides). A reading: the force's twist on
//! the car, `flt_613FA8` the inverse inertia.
//!
//! In:  esi the record, edi the car block
//! Out: every register kept; no flags read
//!
//! Caller: `physics::apply_torques` (in a loop over records).
//! Self-test: a copy of the original (its two scratch vectors the test's, its three calls
//! kept) on a test record and car block, against the converted version: exact at double
//! precision, within `SUM` at extended.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, game_read, hook_asm, rva};
use crate::math::x87_run::neg_product2_plus;
use crate::physics::apply_force::BLOCK_LEN;
use crate::physics::turn_by_inverse_at::turn_by_inverse_at;
use crate::physics::turn_by_matrix_at::turn_by_matrix_at;
use crate::scene::vector3::{read_floats, write_floats};
use crate::scene::view_matrix::cross_product;
use crate::screens::harness::Left;
use crate::service::floats::{
    Agreement, Floats, SUM, Tolerance, at_test_precisions, compare_memory, dispatch_state_floats,
};
use crate::service::harness::{Region, capture};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::apply_torque", rva(0x45FB82), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const WORK_RVA: usize = rva(0x6D30B4); // dword_6D30B4
const CROSS_RVA: usize = rva(0x614274); // flt_614274
const STEP_RVA: usize = rva(0x61388C); // flt_61388C
const INERTIA_RVA: usize = rva(0x613FA8); // flt_613FA8
/// Where a car block's spin is.
pub const SPIN_OFFSET: u32 = 0x184;

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

/// The Rust core: the record at `rec`, the car at `car`, the work vector at `work`, the
/// cross product at `cross`.
pub unsafe fn apply_torque_at(rec: u32, car: u32, work: u32, cross: u32) {
    unsafe {
        for k in 0..3 {
            put(work + 4 * k, at(car + SPIN_OFFSET + 4 * k));
        }
        turn_by_inverse_at(work, car);
        cross_product(rec as usize, rec as usize + 0xC, cross as usize);
        let step = game_read::<u32>(STEP_RVA);
        for k in 0..3 {
            let inertia = game_read::<u32>(INERTIA_RVA + 4 * k as usize);
            let w = work + 4 * k;
            put(
                w,
                neg_product2_plus(at(cross + 4 * k), step, inertia, at(w)),
            );
        }
        turn_by_matrix_at(work, car);
        for k in 0..3 {
            put(car + SPIN_OFFSET + 4 * k, at(work + 4 * k));
        }
    }
}

/// The converted version (`rc2k_core::physics::pushes`, plain floats): the record at
/// `rec`, the car at `car`, the work vector at `work`, the cross product at `cross`.
// Converted: `rc2k_core::physics::pushes::apply_torque`.
pub unsafe fn apply_torque_at_plain(rec: u32, car: u32, work: u32, cross: u32) {
    unsafe {
        let twist = rc2k_core::physics::pushes::apply_torque(
            read_floats(rec),
            read_floats(car + SPIN_OFFSET),
            f32::from_bits(game_read(STEP_RVA)),
            game_read::<[f32; 3]>(INERTIA_RVA),
            &read_floats(car + crate::physics::turn_by_inverse_at::INVERSE_OFFSET),
            &read_floats(car + crate::physics::turn_by_matrix_at::MATRIX_OFFSET),
        );
        write_floats(cross, &twist.cross);
        write_floats(work, &twist.spin);
        write_floats(car + SPIN_OFFSET, &twist.spin);
    }
}

/// The float places it writes on the car at `car`, the work vector at `work`, the cross
/// product at `cross`.
fn floats(car: u32, work: u32, cross: u32) -> [(Region, Tolerance); 3] {
    [
        ((work, 12), SUM),
        ((cross, 12), SUM),
        ((car + SPIN_OFFSET, 12), SUM),
    ]
}

/// The Rust version (the exact one, for `physics::apply_torques`).
pub unsafe fn apply_torque(rec: u32, car: u32) {
    unsafe {
        apply_torque_at(
            rec,
            car,
            game_addr(WORK_RVA) as u32,
            game_addr(CROSS_RVA) as u32,
        )
    }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe {
        apply_torque_at_plain(
            r.esi,
            r.edi,
            game_addr(WORK_RVA) as u32,
            game_addr(CROSS_RVA) as u32,
        )
    };
    Ok(Left::nothing())
}

/// What the function writes on the car at `car`.
pub fn regions(car: u32) -> Vec<Region> {
    vec![
        (game_addr(WORK_RVA) as u32, 12),
        (game_addr(CROSS_RVA) as u32, 12),
        (car + SPIN_OFFSET, 12),
    ]
}

extern "C" fn dispatch(r: &mut Regs) {
    let memory = floats(
        r.edi,
        game_addr(WORK_RVA) as u32,
        game_addr(CROSS_RVA) as u32,
    );
    let floats = Floats {
        memory: &memory,
        regs: &[],
    };
    dispatch_state_floats(&HOOK, r, call_original, &regions(r.edi), &floats, &rust);
}

/// A copy of the original whose work vector and cross product are at `work`, `cross`.
pub fn test_copy(work: u32, cross: u32) -> Result<crate::redirect::RedirectedCopy, String> {
    crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x45FB82,
            len: 0xD3,
            entry: &[0x60, 0x60, 0x50, 0x8B, 0x87, 0x84, 0x01, 0x00, 0x00],
            calls: &[0x2B, 0x3C, 0xA8],
            jumps_out: &[],
        },
        &|_| None,
        &[
            (0x6D30B4, work),
            (0x6D30B8, work + 4),
            (0x6D30BC, work + 8),
            (0x614274, cross),
            (0x614278, cross + 4),
            (0x61427C, cross + 8),
        ],
        18,
    )
}

/// Fills a test car block (its spin and both matrices) and record by `seed`.
pub fn fill(rec: u32, car: u32, seed: u32) {
    unsafe {
        for k in 0..BLOCK_LEN / 4 {
            put(car + 4 * k, 0x5A00_0000 | k);
        }
        for k in 0..8 {
            put(
                rec + 4 * k,
                ((seed * 7 + k * 3) as f32 * 0.5 - 6.0).to_bits(),
            );
        }
        for k in 0..3 {
            put(
                car + SPIN_OFFSET + 4 * k,
                ((seed + k) as f32 * 0.75 - 1.5).to_bits(),
            );
        }
        let a = seed as f32 * 0.3;
        let (s, c) = a.sin_cos();
        let turn = [c, 0.0, -s, 0.0, 0.0, 1.0, 0.0, 0.0, s, 0.0, c, 0.0];
        let back = [c, 0.0, s, 0.0, 0.0, 1.0, 0.0, 0.0, -s, 0.0, c, 0.0];
        for k in 0..12 {
            put(car + 0x9CC + 4 * k as u32, turn[k as usize].to_bits());
            put(car + 0xB4 + 4 * k as u32, back[k as usize].to_bits());
        }
        for k in 12..16u32 {
            put(car + 0x9CC + 4 * k, (seed as f32 * 0.1).to_bits());
            put(car + 0xB4 + 4 * k, (k as f32 - seed as f32).to_bits());
        }
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut cells = vec![0u32; (BLOCK_LEN as usize) / 4 + 8 + 6];
    let car = cells.as_mut_ptr() as u32;
    let rec = car + BLOCK_LEN;
    let work = rec + 0x20;
    let cross = work + 12;
    let copy = match test_copy(work, cross) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let all = [(car, 4 * cells.len())];
    let memory = floats(car, work, cross);
    let tolerant = Floats {
        memory: &memory,
        regs: &[],
    };
    at_test_precisions(&tolerant, &mut |word, fl| {
        for seed in 1..=6u32 {
            fill(rec, car, seed);
            let before = Regs {
                esi: rec,
                edi: car,
                ..Regs::SENTINEL
            };
            let mut ro = before;
            unsafe { call_game(&mut ro, copy.addr()) };
            let theirs = capture(&all);
            fill(rec, car, seed);
            unsafe { apply_torque_at_plain(rec, car, work, cross) };
            let mine = capture(&all);
            let regs_ok = ro.preserved_except(&before, 0);
            let agreement = compare_memory(&all, &theirs, &mine, fl);
            t.check(regs_ok && agreement != Agreement::Differ, || {
                let first = theirs.iter().zip(&mine).position(|(a, b)| a != b);
                format!("control word {word:#x}, seed {seed}: regs_ok={regs_ok}, first difference at byte {first:?}")
            });
        }
    });
    drop((copy, cells));
    t.finish()
}
