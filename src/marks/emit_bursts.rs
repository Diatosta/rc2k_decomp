//! `marks::emit_bursts` (0x4CD21A): two bursts of ring entries from a wheel (kinds 0x11
//! and 0x10), when the wheel's second flags (+0x24, kept in `dword_939BAA`) >> 6 are not
//! 0. Each half: a strength (`flt_935694`: the wheel's +0x10 times a factor; the second
//! `flt_935698`: a blend of +0xC and +0x18) and its rate; when the half's random's bits
//! 16 .. 23 are under the rate, a direction (`flt_935688` ..) and a count (a random's low
//! 12 bits modulo (flags >> 6) times 4); then that many entries: their motion the
//! direction spread (`marks::spread_point`), a size, a look (+0x58), a turn from the car's
//! +0x4C .. (+0x38 ..), a place at the wheel less the car's matrix's place plus a random
//! side, a fade (+0x5C), a life of 2000, +0x7C / +0x80 `dword_935476` / `dword_93547A`.
//!
//! In:  edi the wheel record, esi the car object
//! Out: every register kept
//!
//! Callers: `marks::emit_wheel_particles`' kinds 2 .. 4, 6 .. 9, 0xC. The original's
//! loops push esi; when the ring is full (0x3FE taken) inside one they leave by
//! 0x4CD741, which takes only that push off: `popa` restores shifted registers and
//! `retn` returns to a register's value (section 16; a crash candidate). Refuses (before
//! anything) a null wheel, a null object, car or matrix where read, and a ring that would
//! fill inside a burst. Self-test: a ring, wheel and car of the test's
//! (`marks::emit_particle_6::TestEmitter`), several strengths and fillings, the refusal.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::marks::emit_particle_6::{TEST_ENTRIES, TestEmitter};
use crate::marks::emitters::*;
use crate::math::x87;
use crate::scene::matrix3::{extended, rounded};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at as at_rva, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::emit_bursts", rva(0x4CD21A), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const FLAGS2_RVA: usize = rva(0x939BAA); // g_marksWheelFlags2: the wheel's +0x24 kept
const DIRECTION_RVA: usize = rva(0x935688); // flt_935688 .. flt_935690
const SCRATCH_RVA: usize = rva(0x765138); // dword_765138
const PICK_RVA: usize = rva(0x9356C0); // dword_9356C0
const RATE_SCALE_RVA: usize = rva(0x765190); // flt_765190
const HALF_K_RVA: usize = rva(0x76514C); // flt_76514C
const SPREAD_RVA: usize = rva(0x765158); // flt_765158
const TURN_K_RVA: usize = rva(0x765150); // flt_765150
const FADE_K_RVA: usize = rva(0x765214); // flt_765214
const LOOKS_RVA: [usize; 2] = [rva(0x935476), rva(0x93547A)];
const LIFE: u32 = 0x7D0;

/// One half's variables.
pub struct Half {
    pub kind: u32,
    pub strength: usize,
    pub rate_random: usize,
    pub count_random: usize,
    pub entry_random: usize,
    /// The direction's factors (x, y, z) and whether |s| is scaled by `flt_765158` in y.
    pub factors: [usize; 3],
    pub scaled: bool,
    /// The size's mask and base, the side's factors.
    pub size: (u32, u32),
    pub sides: [usize; 3],
}

/// One function's variables: its two halves, the strength's factor, the blend's two.
pub struct BurstSet {
    pub halves: [Half; 2],
    pub strength_k: usize,
    pub blend_k: [usize; 2],
}

/// `emit_bursts`'.
pub const FIRST: BurstSet = BurstSet {
    halves: [
        Half {
            kind: 0x11,
            strength: rva(0x935694),
            rate_random: rva(0x93A3D6),
            count_random: rva(0x93A3DA),
            entry_random: rva(0x93A3DE),
            factors: [rva(0x93545E), rva(0x935462), rva(0x935466)],
            scaled: false,
            size: (0x7FF, 0x400),
            sides: [rva(0x93543A), rva(0x93543E), rva(0x935442)],
        },
        Half {
            kind: 0x10,
            strength: rva(0x935698),
            rate_random: rva(0x93A3EA),
            count_random: rva(0x93A3EE),
            entry_random: rva(0x93A3F2),
            factors: [rva(0x93543A), rva(0x93543E), rva(0x935442)],
            scaled: true,
            size: (0x3FF, 0x600),
            sides: [rva(0x93545E), rva(0x935462), rva(0x935466)],
        },
    ],
    strength_k: rva(0x93A3D2),
    blend_k: [rva(0x93A3E2), rva(0x93A3E6)],
};

/// The half's strength and rate (`n` 0 or 1), from the wheel.
unsafe fn strength(set: &BurstSet, n: usize, wheel: u32) -> (u32, i32) {
    unsafe {
        let scale: u32 = game_read(RATE_SCALE_RVA);
        if n == 0 {
            x87::times_kept_abs_to_int(at(wheel + 0x10), game_read(set.strength_k), scale)
        } else {
            x87::blend_kept_abs_to_int(
                at(wheel + 0xC),
                at(wheel + 0x18),
                game_read(set.blend_k[0]),
                game_read(set.blend_k[1]),
                scale,
            )
        }
    }
}

/// The entries each half would make, worked out without writing.
unsafe fn counts(set: &BurstSet, wheel: u32) -> [u32; 2] {
    unsafe {
        let flags2 = at(wheel + 0x24);
        if flags2 >> 6 == 0 {
            return [0; 2];
        }
        std::array::from_fn(|n| {
            let h = &set.halves[n];
            let (_, rate) = strength(set, n, wheel);
            let r = game_read::<u32>(h.rate_random)
                .wrapping_mul(0xC5)
                .wrapping_add(0x5D);
            if (r.rotate_right(16) & 0xFF) as i32 >= rate {
                return 0;
            }
            let c = game_read::<u32>(h.count_random)
                .wrapping_mul(0xC5)
                .wrapping_add(0x5D);
            (c & 0xFFF) % ((flags2 >> 6) << 2)
        })
    }
}

/// Why the Rust version refuses, if it does, worked out without writing.
pub unsafe fn refusal(set: &BurstSet, wheel: u32, object: u32) -> Option<&'static str> {
    unsafe {
        if wheel == 0 {
            return Some("a null wheel");
        }
        if at(wheel + 0x24) >> 6 == 0 {
            return None;
        }
        let [a, b] = counts(set, wheel);
        let car: u32 = game_read(OBJECT_RVA);
        if a + b > 0 && (object == 0 || car == 0 || at(car + 0x24) == 0) {
            return Some("a null object, car or matrix");
        }
        let taken: u32 = game_read(RING_COUNT_RVA);
        if taken <= MOST_TAKEN && taken + a + b > MOST_TAKEN {
            return Some(
                "the ring fills inside a burst: the original returns to a register's value",
            );
        }
        None
    }
}

/// The Rust version (see the header).
pub unsafe fn emit_bursts(wheel: u32, object: u32) -> Result<(), Refused> {
    unsafe { emit_bursts_with(&FIRST, wheel, object) }
}

/// The shared body, on `set`'s variables.
pub unsafe fn emit_bursts_with(set: &BurstSet, wheel: u32, object: u32) -> Result<(), Refused> {
    unsafe {
        if let Some(why) = refusal(set, wheel, object) {
            return Err(BadInput(why).into());
        }
        let flags2 = at(wheel + 0x24);
        game_write(FLAGS2_RVA, flags2);
        if flags2 >> 6 == 0 {
            return Ok(());
        }
        for (n, h) in set.halves.iter().enumerate() {
            let (s, rate) = strength(set, n, wheel);
            game_write(h.strength, s);
            game_write(PICK_RVA, rate);
            let r = next_c5(h.rate_random);
            if (r.rotate_right(16) & 0xFF) as i32 >= rate {
                continue;
            }
            let car: u32 = game_read(OBJECT_RVA);
            let k: u32 = game_read(HALF_K_RVA);
            let s: u32 = game_read(h.strength);
            let scale = h.scaled.then(|| game_read::<u32>(SPREAD_RVA));
            let direction = [
                x87::mul2(game_read(h.factors[0]), s, k),
                x87::times_less_abs_times(game_read(h.factors[1]), s, scale, k),
                x87::mul2(game_read(h.factors[2]), s, k),
            ];
            game_write(DIRECTION_RVA, direction);
            let count = (next_c5(h.count_random) & 0xFFF) % ((flags2 >> 6) << 2);
            for _ in 0..count {
                let e = take_entry(at(object + 0x1C) & 0xFF_FFFF, h.kind);
                let put = |o: u32, v: u32| ((e + o) as *mut u32).write_unaligned(v);
                let d: [u32; 3] = game_read(DIRECTION_RVA);
                let (out, _) = crate::marks::spread_point::spread_point([
                    extended(game_read(SPREAD_RVA)),
                    extended(d[2]),
                    extended(d[1]),
                    extended(d[0]),
                ]);
                put(0x34, rounded(out[0]));
                put(0x30, rounded(out[1]));
                put(0x2C, rounded(out[2]));
                let r = next_c5(h.entry_random);
                let size = (r.rotate_right(8) & h.size.0) + h.size.1;
                game_write(SCRATCH_RVA, size);
                put(0x60, x87::from_int(size as i32));
                put(0x58, ((flags2 & 0x3F) << 4) + (r.rotate_right(17) & 3));
                let tk: u32 = game_read(TURN_K_RVA);
                put(0x40, x87::mul(at(car + 0x54), tk));
                put(0x3C, x87::mul(at(car + 0x50), tk));
                put(0x38, x87::mul(at(car + 0x4C), tk));
                for o in [0x44, 0x48, 0x4C] {
                    put(o, 0);
                }
                let m = at(car + 0x24);
                let side = (r.rotate_right(8) & 0x3FFF).wrapping_sub(0x2000);
                game_write(SCRATCH_RVA, side);
                let place: [u32; 3] = std::array::from_fn(|k| {
                    let k4 = 4 * k as u32;
                    x87::less_plus_int_times(
                        at(wheel + 0x3C + k4),
                        at(m + 0x30 + k4),
                        game_read(h.sides[k]),
                        side as i32,
                    )
                });
                put(0x10, place[0]);
                put(0x14, place[1]);
                put(0x28, place[1]);
                put(0x18, place[2]);
                let pick = r & 0x7_FFFF;
                game_write(PICK_RVA, pick);
                put(
                    0x5C,
                    x87::int_times_plus(pick as i32, game_read(FADE_K_RVA), game_read(ONE_RVA)),
                );
                put(0x84, LIFE);
                put(0x7C, game_read(LOOKS_RVA[0]));
                put(0x80, game_read(LOOKS_RVA[1]));
                put(0x68, car);
            }
        }
    }
    Ok(())
}

/// What it may write: the globals, the entries it would take, the wheel unchanged.
pub fn regions(set: &BurstSet, wheel: u32) -> Vec<Region> {
    let mut r: Vec<Region> = [
        FLAGS2_RVA,
        PICK_RVA,
        SCRATCH_RVA,
        RING_CURSOR_RVA,
        RING_COUNT_RVA,
        rva(0x923004), // g_marksSpreadRandom
    ]
    .map(at_rva)
    .to_vec();
    r.push((game_addr(DIRECTION_RVA) as u32, 12));
    for h in &set.halves {
        r.extend([h.strength, h.rate_random, h.count_random, h.entry_random].map(at_rva));
    }
    if wheel != 0 {
        let n: u32 = unsafe { counts(set, wheel) }.iter().sum();
        let mut e: u32 = unsafe { game_read(RING_CURSOR_RVA) };
        for _ in 0..n {
            if e == 0 {
                break;
            }
            e = unsafe { at(e) };
            r.push((e, ENTRY_LEN));
        }
    }
    r
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { emit_bursts(r.edi, r.esi) }?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(&FIRST, r.edi), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    test_with(&mut t, &FIRST, call_original, &rust);
    t.finish()
}

/// The test of either copy.
pub fn test_with(
    t: &mut SelfTest,
    set: &BurstSet,
    call_original: crate::screens::harness::CallOriginal,
    rust: crate::service::harness::Rust,
) {
    let mut extra: Vec<usize> = vec![
        FLAGS2_RVA,
        PICK_RVA,
        SCRATCH_RVA,
        rva(0x923004),
        DIRECTION_RVA,
        DIRECTION_RVA + 4,
        DIRECTION_RVA + 8,
    ];
    for h in &set.halves {
        extra.extend([h.strength, h.rate_random, h.count_random, h.entry_random]);
    }
    let mut test = TestEmitter::new(&extra);
    let regions = test.regions();
    let mut made = 0;
    let mut case = 0u32;
    // (flags2, speeds +0xC, +0x10, +0x18, taken)
    for (flags2, speeds, taken) in [
        (0x40u32, [3.0f32, 50.0, -2.0], 0u32),
        (0xC5, [-10.0, -80.0, 40.0], 5),
        (0x3F, [1.0, 100.0, 1.0], 0),
        (0x200, [200.0, 0.5, 300.0], 0x10),
        (0x7FC0, [90.0, 90.0, -90.0], 0x400),
        (0x80, [0.0, 0.0, 0.0], 0),
    ] {
        case += 1;
        let ptrs = (
            test.wheel.as_mut_ptr(),
            test.object.as_mut_ptr(),
            test.matrix.as_mut_ptr(),
        );
        let seed = case.wrapping_mul(0x2545_F491);
        test.prepare(taken);
        let prepare = || unsafe {
            restore(&regions, &vec![0xAA; regions.iter().map(|r| r.1).sum()]);
            let base = test.ring.as_ptr() as u32;
            for n in 0..TEST_ENTRIES {
                let e = (base + n * ENTRY_LEN as u32) as *mut u32;
                e.write_unaligned(base + ((n + 1) % TEST_ENTRIES) * ENTRY_LEN as u32);
            }
            game_write(RING_CURSOR_RVA, base + 3 * ENTRY_LEN as u32);
            game_write(RING_COUNT_RVA, taken);
            game_write(OBJECT_RVA, ptrs.1 as u32);
            for (k, h) in set.halves.iter().enumerate() {
                for (j, r) in [h.rate_random, h.count_random, h.entry_random]
                    .iter()
                    .enumerate()
                {
                    game_write(*r, seed ^ ((k * 3 + j) as u32 * 0x1111_1111));
                }
            }
            game_write(rva(0x923004), seed ^ 0x77);
            for n in 0..0x20 {
                *ptrs.0.add(n) = (n as f32 * 0.75 - 3.0).to_bits();
            }
            *ptrs.0.add(3) = speeds[0].to_bits();
            *ptrs.0.add(4) = speeds[1].to_bits();
            *ptrs.0.add(6) = speeds[2].to_bits();
            *ptrs.0.add(9) = flags2;
            for n in 0..0x30 {
                *ptrs.1.add(n) = (n as f32 * 1.25 + 0.5).to_bits();
            }
            *ptrs.1.add(0x1C / 4) = 0x1234_5678;
            *ptrs.1.add(0x24 / 4) = ptrs.2 as u32;
            for n in 0..0x10 {
                *ptrs.2.add(n) = ((n as f32 - 5.0) * 0.3).to_bits();
            }
        };
        prepare();
        let refused = unsafe { refusal(set, ptrs.0 as u32, ptrs.1 as u32) }.is_some();
        if refused {
            // The original would fill the ring inside a burst: not run.
            t.check(
                unsafe { emit_bursts_with(set, ptrs.0 as u32, ptrs.1 as u32) }.is_err(),
                || format!("flags2 {flags2:#x}, taken {taken:#x}: not refused"),
            );
            continue;
        }
        check_state(
            t,
            &format!("flags2 {flags2:#x}, speeds {speeds:?}, taken {taken:#x}"),
            call_original,
            Regs {
                edi: ptrs.0 as u32,
                esi: ptrs.1 as u32,
                ..Regs::SENTINEL
            },
            &prepare,
            &regions,
            rust,
        );
        let now: u32 = unsafe { game_read(RING_COUNT_RVA) };
        if now != taken && now <= TEST_ENTRIES + taken {
            made += 1;
        }
    }
    t.check(made > 0, || "no case made an entry".into());
    drop(test);
}
