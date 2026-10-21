//! `physics::reset_car_grades` (0x440A22): resets a car block at esi (0xFE0 bytes, the
//! work copy's layout): +0x29C .. +0x367 zeroed (`rep stosd`); +0x3F4 .. +0x404 = 1.0
//! (`flt_5C5D38`); +0x448 .. +0x458 = `dword_71A00C` / 256 x 100, +0x438 .. +0x444 and
//! +0x45C = `dword_71A008` / 256 x 100 (both read from the work copy `g_car`, whatever
//! esi is: the setup's percentages, by the look of it); +0x408 .. +0x434 and +0x64 ..
//! +0x80 zeroed; +0x298 = -1. Stream A's reading: the block's grades and values.
//!
//! In:  esi the block
//! Out: eax 0, ecx 0, edi esi + 0x368; the rest kept; no flags read
//!
//! Callers: `physics::reset_current_car` (esi `g_car`), and unreferenced code at
//! 0x43FDDF that runs it over every car before `race::grade_all_cars`.
//! Self-test: a copy of the original on a test block (its two reads of the work copy
//! made the test's) against the converted version, fillings by lot at the test
//! precisions (exact at any).

use crate::errors::Refused;
use crate::framework::{
    EAX, ECX, EDI, RawHook, Regs, SelfTest, call_game, game_addr, game_read, hook_asm, rva,
};
use crate::math::x87_run::{TEST_CONTROL_WORDS, with_control_word};
use crate::screens::harness::Left;
use crate::service::harness::{capture, dispatch_state};
use rc2k_core::physics::grades::{percentage, reset_writes};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::reset_car_grades", rva(0x440A22), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub const BLOCK_LEN: usize = 0xFE0;
const ONE_RVA: usize = rva(0x5C5D38); // flt_5C5D38, 1.0
const K256_RVA: usize = rva(0x5C5E2C); // flt_5C5E2C, 256.0
const K100_RVA: usize = rva(0x5C5D14); // flt_5C5D14, 100.0

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

/// The Rust core: the block at `b`, the two settings read at `s0c`, `s08` (the same at
/// any precision).
// Converted: `rc2k_core::physics::grades::{percentage, reset_writes}`.
pub unsafe fn reset_grades_at(b: u32, s0c: u32, s08: u32) -> Left {
    unsafe {
        let k = |a: usize| f32::from_bits(game_read(a));
        let (k256, k100) = (k(K256_RVA), k(K100_RVA));
        let a = percentage((s0c as *const i32).read_unaligned(), k256, k100);
        let c = percentage((s08 as *const i32).read_unaligned(), k256, k100);
        for (offset, v) in reset_writes(k(ONE_RVA), a, c) {
            put(b + offset, v);
        }
        Left::nothing()
            .with(EAX, 0)
            .with(ECX, 0)
            .with(EDI, b + 0x368)
    }
}

/// The Rust version.
pub unsafe fn reset_car_grades(block: u32) -> Left {
    unsafe {
        reset_grades_at(
            block,
            game_addr(rva(0x71A00C)) as u32,
            game_addr(rva(0x71A008)) as u32,
        )
    }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { reset_car_grades(r.esi) })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[(r.esi, BLOCK_LEN)], &rust);
}

/// A copy of the original reading the two settings at `s0c`, `s08`.
pub fn test_copy(s0c: u32, s08: u32) -> Result<crate::redirect::RedirectedCopy, String> {
    crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x440A22,
            len: 0x15A,
            entry: &[0x8D, 0xBE, 0x9C, 0x02, 0x00, 0x00],
            calls: &[],
            jumps_out: &[],
        },
        &|_| None,
        &[(0x71A00C, s0c), (0x71A008, s08)],
        2,
    )
}

/// Fills a test block by `seed` (the two settings at +0x34 and +0x30 small whole
/// numbers).
pub fn fill_block(b: u32, seed: u32) {
    let mut s = seed | 1;
    for k in 0..(BLOCK_LEN as u32 / 4) {
        s = s.wrapping_mul(1_103_515_245).wrapping_add(12345);
        unsafe { put(b + 4 * k, s) };
    }
    unsafe {
        put(b + 0x34, (seed.wrapping_mul(37) % 300).wrapping_sub(20));
        put(b + 0x30, seed.wrapping_mul(91) % 256);
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut block = vec![0u32; BLOCK_LEN / 4];
    let b = block.as_mut_ptr() as u32;
    let copy = match test_copy(b + 0x34, b + 0x30) {
        Ok(copy) => copy,
        Err(why) => return t.skip(&why),
    };
    for word in TEST_CONTROL_WORDS {
        with_control_word(word, &mut || {
            for seed in 1..=8u32 {
                fill_block(b, seed);
                let before = Regs {
                    esi: b,
                    ..Regs::SENTINEL
                };
                let mut ro = before;
                unsafe { call_game(&mut ro, copy.addr()) };
                let theirs = capture(&[(b, BLOCK_LEN)]);
                fill_block(b, seed);
                let left = unsafe { reset_grades_at(b, b + 0x34, b + 0x30) };
                let mine = capture(&[(b, BLOCK_LEN)]);
                let regs_ok = left.matches(&before, &ro);
                t.check(regs_ok && theirs == mine, || {
                    let first = theirs.iter().zip(&mine).position(|(x, y)| x != y);
                    format!(
                        "control word {word:#x}, seed {seed}: regs_ok={regs_ok}, first difference at byte {first:?}"
                    )
                });
            }
        });
    }
    drop((copy, block));
    t.finish()
}
