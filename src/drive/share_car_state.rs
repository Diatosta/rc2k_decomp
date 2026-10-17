//! `drive::share_car_state` (0x45699C): one car's state from its block (esi, 0xFE0 bytes)
//! into its record (edi, `off_71B3D8[n]`, read by the drawing and the sounds) and back:
//! - a count at +0xFAC: when not 0, record +0x30 = 1 and the count less 1;
//! - record +0x2C = the sound object's (ebp) +0xF0, which is cleared (`xchg`);
//! - two levels (record +0x4C0, +0x4C4) from block +0x2B4, +0x2B8: 0 below
//!   `flt_63DB5C`, else (x - it) / `flt_5C5D44` `fistp`'d (through `dword_62400C`) held to
//!   0 .. 7 (the sign of the difference);
//! - the car file's light record (`g_lightRecords`, 0x134 each, by
//!   `race::car_file_number` of block +0x24) +0x2C .. +0x38 into record +0xC .. +0x18;
//! - about eighty fields moved as the original moves them (`MOVES`, the wheels' four
//!   each), four of them cleared by the block's switches (+0x600 .. and bits 5 .. 8 of
//!   +0x298), a part only when block +0xA34 is set;
//! - record +0x258 (6 dwords) into block +0x700, whose 0x80000000 entries are then
//!   filled from the next one that is not (round, at most six looked at); record +0x32C
//!   (8) into block +0xA3C, record +0x3F4 (0x11) into block +0xA5C.
//!
//! In:  esi the block, edi the record, ebp the sound object
//! Out: eax block +0x1A0 (with +0xA34 set) or +0x1BC; ebx +0x1A4, ecx +0x1AC, edx +0x92C;
//!      the rest kept (the copies are inside `pusha` / `popa`); no flags read
//!
//! Caller: `drive::share_all_cars` (0x456484), for each of the eight cars, inside
//! `pusha` / `popa`.
//! Refuses (before anything) a car file number above 0x21 (past `g_lightRecords`).
//! Self-test: a copy of the original (its scratch `dword_62400C` the test's, its calls to
//! `car_file_number` kept) on a test block, record and sound object filled by lot, the
//! switches and the holes varied, under three precisions; the refusal.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    EAX, EBX, ECX, EDX, RawHook, Regs, SelfTest, call_game, game_addr, game_read, hook_asm, rva,
};
use crate::math::x87;
use crate::math::x87_run::{TEST_CONTROL_WORDS, compare_full, sub_div_to_int, with_control_word};
use crate::race::mud::car_file_number;
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("drive::share_car_state", rva(0x45699C), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub const BLOCK_LEN: usize = 0xFE0;
pub const RECORD_LEN: usize = 0x440;
pub const SOUND_LEN: usize = 0x100;
const SCRATCH: usize = 0x62400C; // dword_62400C
const LEVEL_FROM_RVA: usize = rva(0x63DB5C); // flt_63DB5C
const LEVEL_STEP_RVA: usize = rva(0x5C5D44); // flt_5C5D44
const LIGHT_RECORDS: u32 = 0x22;

/// (record offset, block offset) moves in the original's order, after the light record.
const MOVES_A: [(u32, u32); 20] = [
    (0xF4, 0x95C),
    (0x13C, 0x960),
    (0x184, 0x964),
    (0x1CC, 0x968),
    (0xF8, 0x984),
    (0x140, 0x988),
    (0x188, 0x98C),
    (0x1D0, 0x990),
    (0x100, 0x970),
    (0x148, 0x974),
    (0x190, 0x978),
    (0x1D8, 0x97C),
    (0x104, 0x998),
    (0x14C, 0x99C),
    (0x194, 0x9A0),
    (0x1DC, 0x9A4),
    (0x108, 0x9AC),
    (0x150, 0x9B0),
    (0x198, 0x9B4),
    (0x1E0, 0x9B8),
];
/// The wheels' sums: (record offsets of the four, block offset of the first).
const QUADS: [([u32; 4], u32); 2] = [
    ([0xE0, 0xE4, 0xE8, 0xEC], 0x1B8),
    ([0x170, 0x174, 0x178, 0x17C], 0x1BC),
];
/// With block +0xA34 set: (record, block) moves.
const MOVES_B: [(u32, u32); 9] = [
    (0x214, 0x96C),
    (0x218, 0x994),
    (0x220, 0x980),
    (0x224, 0x9A8),
    (0x228, 0x9BC),
    (0x210, 0x930),
    (0x21C, 0x944),
    (0x204, 0x198),
    (0x20C, 0x1A0),
];

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

/// The light record's number for the block at `s`, or the refusal.
unsafe fn light_record(s: u32) -> Result<u32, Refused> {
    let n = car_file_number(unsafe { at(s + 0x24) });
    if n >= LIGHT_RECORDS {
        return Err(BadInput("a car file number above 0x21 (past g_lightRecords)").into());
    }
    Ok(n)
}

/// The Rust core: block `s`, record `o`, sound object `p`, the scratch at `scratch`.
pub unsafe fn share_car_state_at(s: u32, o: u32, p: u32, scratch: u32) -> Result<Left, Refused> {
    unsafe {
        let n = light_record(s)?;
        let count = at(s + 0xFAC);
        if count != 0 {
            put(o + 0x30, 1);
            put(s + 0xFAC, count - 1);
        }
        put(o + 0x2C, at(p + 0xF0));
        put(p + 0xF0, 0);
        let from = game_read(LEVEL_FROM_RVA);
        let step = game_read(LEVEL_STEP_RVA);
        for (out, field) in [(0x4C0, 0x2B4), (0x4C4, 0x2B8)] {
            put(o + out, 0);
            if compare_full(at(s + field), from) & 0x100 == 0 {
                let v = sub_div_to_int(at(s + field), from, step);
                put(scratch, v as u32);
                let v = if v < 0 { 0 } else { v };
                let v = if (v.wrapping_sub(7)) < 0 { v } else { 7 };
                put(o + out, v as u32);
            }
        }
        put(o + 0x1C, at(s + 0x240));
        let light = game_addr(crate::race::stage_prefs::LIGHT_RECORDS_RVA) as u32 + 0x134 * n;
        for (r, l) in [(0x14, 0x34), (0x18, 0x38), (0xC, 0x2C), (0x10, 0x30)] {
            put(o + r, at(light + l));
        }
        put(o + 0x24, s + 0x8C0);
        put(o + 0x28, s + 0x740);
        put(o + 8, x87::loaded(at(s + 0x1D8)));
        put(o + 4, n);
        for (r, b) in MOVES_A {
            put(o + r, at(s + b));
        }
        for (offsets, first) in QUADS {
            let (a, bb, c) = (at(s + first), at(s + 0x1A4), at(s + 0x1AC));
            for base in [offsets[0], offsets[0] + 0x48] {
                put(o + base, a);
                put(o + base + 4, bb);
                put(o + base + 8, 0);
                put(o + base + 0xC, c);
            }
        }
        for (k, r) in [0xF0u32, 0x138, 0x180, 0x1C8].into_iter().enumerate() {
            put(o + r, at(s + 0x920 + 4 * k as u32));
        }
        for (k, r) in [0xF0u32, 0x138, 0x180, 0x1C8].into_iter().enumerate() {
            let k = k as u32;
            if at(s + 0x600 + 4 * k) == 0 && at(s + 0x298) >> (5 + k) & 1 != 0 {
                put(o + r, 0);
            }
        }
        put(o + 0x210, 0);
        put(o + 0x21C, 0);
        let zero = x87::loaded(0);
        for r in [0x24C, 0x250, 0x254, 0x244] {
            put(o + r, zero);
        }
        let switched = at(s + 0xA34) != 0;
        if switched {
            for (r, b) in MOVES_B {
                put(o + r, at(s + b));
            }
            for (r, b) in [(0x230, 0xA28), (0x234, 0xA2C), (0x238, 0xA30)] {
                put(o + r, x87::from_int(at(s + b) as i32));
            }
            put(o + 0x22C, at(s + 0xA38));
        }
        for k in 0..8 {
            put(o + 0x2CC + 4 * k, at(s + 0x64 + 4 * k));
        }
        for k in 0..6 {
            put(s + 0x700 + 4 * k, at(o + 0x258 + 4 * k));
        }
        const HOLE: u32 = 0x8000_0000;
        for c in 0..6u32 {
            if at(s + 0x700 + 4 * c) != HOLE {
                continue;
            }
            let mut c2 = c;
            let mut tries = 0;
            loop {
                if at(s + 0x700 + 4 * c2) != HOLE {
                    break;
                }
                c2 += 1;
                if c2 == 6 {
                    c2 = 0;
                }
                tries += 1;
                if tries == 6 {
                    break;
                }
            }
            put(s + 0x700 + 4 * c, at(s + 0x700 + 4 * c2));
        }
        for k in 0..8 {
            put(s + 0xA3C + 4 * k, at(o + 0x32C + 4 * k));
        }
        for k in 0..0x11 {
            put(s + 0xA5C + 4 * k, at(o + 0x3F4 + 4 * k));
        }
        Ok(Left::nothing()
            .with(EAX, at(s + if switched { 0x1A0 } else { 0x1BC }))
            .with(EBX, at(s + 0x1A4))
            .with(ECX, at(s + 0x1AC))
            .with(EDX, at(s + 0x92C)))
    }
}

/// The Rust version.
pub unsafe fn share_car_state(r: &Regs) -> Result<Left, Refused> {
    unsafe { share_car_state_at(r.esi, r.edi, r.ebp, game_addr(rva(SCRATCH)) as u32) }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { share_car_state(r) }
}

pub fn regions(r: &Regs) -> Vec<Region> {
    vec![
        (r.esi, BLOCK_LEN),
        (r.edi, RECORD_LEN),
        (r.ebp + 0xF0, 4),
        (game_addr(rva(SCRATCH)) as u32, 4),
    ]
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(r), &rust);
}

/// Fills a test block, record and sound object by `seed`.
pub fn fill(s: u32, o: u32, p: u32, seed: u32) {
    let mut x = seed.wrapping_mul(0x9E37_79B9) | 1;
    let mut next = move || {
        x = x.wrapping_mul(1_103_515_245).wrapping_add(12345);
        x
    };
    unsafe {
        for (base, len) in [(s, BLOCK_LEN), (o, RECORD_LEN), (p, SOUND_LEN)] {
            for k in 0..(len as u32 / 4) {
                put(base + 4 * k, next());
            }
        }
        put(s + 0x24, next() % 0x22);
        put(s + 0xFAC, next() % 3);
        let from = f32::from_bits(game_read(LEVEL_FROM_RVA));
        for f in [0x2B4, 0x2B8, 0x1D8] {
            put(s + f, (from + (next() % 80) as f32 - 20.0).to_bits());
        }
        for k in 0..4 {
            put(s + 0x600 + 4 * k, next() % 2);
        }
        put(s + 0xA34, next() % 2);
        for k in 0..6 {
            if next() % 3 == 0 {
                put(o + 0x258 + 4 * k, 0x8000_0000);
            }
        }
        if seed.is_multiple_of(4) {
            for k in 0..6 {
                put(o + 0x258 + 4 * k, 0x8000_0000);
            }
        }
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut cells = vec![0u32; (BLOCK_LEN + RECORD_LEN + SOUND_LEN) / 4 + 1];
    let s = cells.as_mut_ptr() as u32;
    let (o, p) = (s + BLOCK_LEN as u32, s + (BLOCK_LEN + RECORD_LEN) as u32);
    let scratch = p + SOUND_LEN as u32;
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x45699C,
            len: 0x50B,
            entry: &[0x8B, 0x86, 0xAC, 0x0F, 0x00, 0x00],
            calls: &[0xD9, 0x120],
            jumps_out: &[],
        },
        &|_| None,
        &[(0x62400C, scratch)],
        4,
    ) {
        Ok(copy) => copy,
        Err(why) => return t.skip(&why),
    };
    let all: [Region; 1] = [(s, 4 * cells.len())];
    for word in TEST_CONTROL_WORDS {
        with_control_word(word, &mut || {
            for seed in 1..=12u32 {
                fill(s, o, p, seed);
                let input = capture(&all);
                let before = Regs {
                    esi: s,
                    edi: o,
                    ebp: p,
                    ..Regs::SENTINEL
                };
                let mut ro = before;
                unsafe { call_game(&mut ro, copy.addr()) };
                let theirs = capture(&all);
                restore(&all, &input);
                let mine = unsafe { share_car_state_at(s, o, p, scratch) };
                let state = capture(&all);
                let regs_ok = mine.as_ref().is_ok_and(|l| l.matches(&before, &ro));
                t.check(regs_ok && theirs == state, || {
                    let first = theirs.iter().zip(&state).position(|(a, b)| a != b);
                    format!(
                        "control word {word:#x}, seed {seed}: regs_ok={regs_ok} {mine:?}, first difference at byte {first:?}"
                    )
                });
            }
        });
    }
    fill(s, o, p, 3);
    unsafe { put(s + 0x24, 0x7777) };
    let refused = car_file_number(0x7777) < LIGHT_RECORDS
        || unsafe { share_car_state_at(s, o, p, scratch) }.is_err();
    t.check(refused, || "a file number above 0x21 not refused".into());
    drop((copy, cells));
    t.finish()
}
