//! `physics::apply_force` (0x45FAE1): one force record (esi, its first three floats) on a
//! car (edi, its block): copied to `flt_614B30` .., times the step `flt_61388C`
//! (`physics::scale_vector`; the step, left in st(0), then rounded into `dword_62400C`),
//! turned into the world by `g_crashMatrix` (`physics::turn_by_car_matrix`), added to the
//! car's motion (+0x198 .. +0x1A0); then the motion turned by the car's inverse matrix
//! (+0x9CC, `scene::rotate_point`) into +0x1A4 .. +0x1AC (the last `g_carSpeed`). All in
//! `pusha` / `popa`. A reading: a force's push on the car.
//!
//! In:  esi the record, edi the car block
//! Out: every register kept; no flags read
//!
//! Caller: `physics::apply_forces` (in a loop over records).
//! Self-test: a copy of the original (its scratch the test's, its three calls kept) on a
//! test record and car block, against the converted version: exact at double precision,
//! within `SUM` at extended.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, game_read, hook_asm, rva};
use crate::math::x87;
use crate::physics::scale_vector::scale_vector;
use crate::physics::turn_by_car_matrix::turn_by_car_matrix;
use crate::scene::matrix3::{extended, rounded};
use crate::scene::rotate_point::rotate_point;
use crate::scene::vector3::{read_floats, write_floats};
use crate::screens::harness::Left;
use crate::service::floats::{
    Agreement, Floats, SUM, Tolerance, at_test_precisions, compare_memory, dispatch_state_floats,
};
use crate::service::harness::{Region, capture};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::apply_force", rva(0x45FAE1), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub const BLOCK_LEN: u32 = 0xFE0;

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

/// The Rust core: the record at `rec`, the car at `car`, the work vector at `work`, the
/// scratch at `scratch`.
pub unsafe fn apply_at(rec: u32, car: u32, work: u32, scratch: u32) {
    unsafe {
        for k in 0..3 {
            put(work + 4 * k, at(rec + 4 * k));
        }
        let dt = extended(game_read(rva(0x61388C)));
        scale_vector(work, dt);
        put(scratch, rounded(dt));
        turn_by_car_matrix(work);
        for k in 0..3 {
            let o = car + 0x198 + 4 * k;
            put(o, x87::add(at(work + 4 * k), at(o)));
        }
        let p = [
            extended(at(car + 0x198)),
            extended(at(car + 0x19C)),
            extended(at(car + 0x1A0)),
        ];
        let out = rotate_point(car + 0x9CC, p);
        for (k, e) in out.iter().enumerate() {
            put(car + 0x1A4 + 4 * k as u32, rounded(*e));
        }
    }
}

/// The converted version (`rc2k_core::physics::pushes`, plain floats): the record at
/// `rec`, the car at `car`, the work vector at `work`, the scratch at `scratch`.
// Converted: `rc2k_core::physics::pushes::apply_force`.
pub unsafe fn apply_at_plain(rec: u32, car: u32, work: u32, scratch: u32) {
    unsafe {
        let crash = game_addr(rva(0x71A08C)) as u32; // g_crashMatrix
        let push = rc2k_core::physics::pushes::apply_force(
            read_floats(rec),
            f32::from_bits(game_read(rva(0x61388C))),
            &read_floats(crash),
            read_floats(car + 0x198),
            &read_floats(car + 0x9CC),
        );
        write_floats(work, &push.work);
        write_floats(scratch, &[push.step]);
        write_floats(car + 0x198, &push.motion);
        write_floats(car + 0x1A4, &push.local);
    }
}

/// The float places it writes on the car at `car`, the work vector at `work`, the scratch
/// at `scratch`.
fn floats(car: u32, work: u32, scratch: u32) -> [(Region, Tolerance); 3] {
    [
        ((work, 12), SUM),
        ((scratch, 4), SUM),
        ((car + 0x198, 0x18), SUM),
    ]
}

/// The Rust version (the exact one, for `physics::apply_forces`).
pub unsafe fn apply_force(rec: u32, car: u32) {
    unsafe {
        apply_at(
            rec,
            car,
            game_addr(rva(0x614B30)) as u32,
            game_addr(rva(0x62400C)) as u32,
        )
    }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe {
        apply_at_plain(
            r.esi,
            r.edi,
            game_addr(rva(0x614B30)) as u32,
            game_addr(rva(0x62400C)) as u32,
        )
    };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    let (work, scratch) = (
        game_addr(rva(0x614B30)) as u32,
        game_addr(rva(0x62400C)) as u32,
    );
    let regions = [(work, 12), (scratch, 4), (r.edi + 0x198, 0x18)];
    let memory = floats(r.edi, work, scratch);
    let floats = Floats {
        memory: &memory,
        regs: &[],
    };
    dispatch_state_floats(&HOOK, r, call_original, &regions, &floats, &rust);
}

/// A copy of the original whose work vector and scratch are at `work`, `scratch`.
pub fn test_copy(work: u32, scratch: u32) -> Result<crate::redirect::RedirectedCopy, String> {
    crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x45FAE1,
            len: 0xA1,
            entry: &[0x60, 0x60, 0x50, 0x8B, 0x06, 0xA3, 0x30, 0x4B, 0x61, 0x00],
            calls: &[0x27, 0x32, 0x87],
            jumps_out: &[],
        },
        &|_| None,
        &[
            (0x614B30, work),
            (0x614B34, work + 4),
            (0x614B38, work + 8),
            (0x62400C, scratch),
        ],
        8,
    )
}

/// Fills a test car block (its motion and inverse matrix) and record by `seed`.
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
                car + 0x198 + 4 * k,
                ((seed + k) as f32 * 1.25 - 2.0).to_bits(),
            );
        }
        let a = seed as f32 * 0.3;
        let m = [
            a.cos(),
            0.0,
            -a.sin(),
            0.0,
            0.0,
            1.0,
            0.0,
            0.0,
            a.sin(),
            0.0,
            a.cos(),
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
        ];
        for (k, v) in m.iter().enumerate() {
            put(car + 0x9CC + 4 * k as u32, v.to_bits());
        }
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut cells = vec![0u32; (BLOCK_LEN as usize) / 4 + 8 + 4];
    let car = cells.as_mut_ptr() as u32;
    let rec = car + BLOCK_LEN;
    let work = rec + 0x20;
    let scratch = work + 12;
    let copy = match test_copy(work, scratch) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let all = [(car, 4 * cells.len())];
    let memory = floats(car, work, scratch);
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
            unsafe { apply_at_plain(rec, car, work, scratch) };
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
