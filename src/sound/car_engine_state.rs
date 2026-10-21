//! `sound::car_engine_state` (0x40D8E2): a car's sound object brought up to date for the
//! turn: the revs eased between the physics' readings, the pitch and volume inputs of
//! the engine sounds, the engine's state, the gear-change and back-fire cues, the
//! listener's distance and the Doppler-like factors.
//!
//! In:  ebx the object
//! Out: edi the object's sound kind (`off_514F90[+4]`, 0x88 bytes each), eax its +0x134
//!      (the new distance), ecx +0x18 (or the back-fire's wait), edx as it came or the
//!      remainder of the revs' `idiv`; ebx, esi, ebp preserved
//! The object's dwords by offset (a reading of the code, names by use):
//! - +0x10 1: a new revs reading (+0x14) came: +0x94/+0x98 the last two readings' times
//!   (`dword_5124E8`), +0x9C/+0xA0 their revs, +0x10 cleared. Else, unless
//!   `dword_5BEC94` is 1, +0x14 = the revs extrapolated (`imul` / `idiv` over the
//!   readings' gap held to 1000 ms), at least 1000.
//! - +0x100 = (+0x14 - kind +4) x kind +0xC; +0x104 = +0x18 as a float over 256 (the
//!   exponent less 8 unless it is 0); +0x108 = |+0x1C| x `flt_516370`.
//! - +0x110 = the last state, +0x10C = 0 (no revs), 1 (up to 1000), 2, or 3 when +0x20
//!   was set (cleared).
//! - +0x118 follows +0x18 (up: by (0x100 - it) >> 6 and one more, down: by it >> 7 and
//!   one more, never past), +0x11C the gap, +0x12C 0 / 1 up / 2 down, +0x128 / +0x130
//!   the turns going up / down (at 1 made 2 when level).
//! - +0xFC the last direction; up then down (or the reverse: the two add to 3): +0xF4
//!   += 5. +0xE8 (the gear) changed: +0xEC = it, +0xF4 += 7, and with kind bit 0x2000, a
//!   gear over 1 and +0x11C at least 0x32: +0x1FC and +0xF0 1, +0x210 = (clock & 0xF) x
//!   1/60 + 0.75 (through `dword_5183CC`). +0x24 = (eax is 0): the gear 0, or (in that
//!   last case) the clock's low four bits 0.
//! - Kind bit 0x1000 (back-fires): going up clears +0x1E0, +0x1E4, +0x1F4, +0x1EC; else
//!   with +0x118 at most 0xF0, +0x1F4 counts down and below 0 is set to (clock & 3) +
//!   (max(+0x14 - 2000, 0) >> 9) + 1, +0x1E0 1 and +0x1EC += 8 held to 0x20. Then going
//!   up clears +0xF8; else unless +0xF8 is 1, going down by more than 20: with +0x14 at
//!   least 4000 +0x1C4 = kind +0 & 0xFFF and +0x1C8 1 (the slot of
//!   `sound::play_timed_slot`), and +0xF8 1.
//! - +0xF4 counts down to 0. +0x134 the distance (the root of +0x2C .. +0x34 squared),
//!   +0x138 its change x the turns a second, +0x13C eased toward (+0x138 + 337.52) x
//!   0.00296, +0x140 = 1, or 1 / the distance when that is over 1 (compared as integers).
//!
//! One caller, `sound::car_sounds`.
//! Refuses (before writing) a kind past the 36 of the table and a revs extrapolation
//! whose quotient does not fit (the original's `idiv` traps).
//! Shadow mode: both, the object and the scratch dword compared.
//! Self-test: the original against the Rust version on objects of the test's, the
//! table's first kind pointing at a kind of the test's (put back).

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::math::x87::from_int;
use crate::physics::flow::{base, x87_run};
use crate::service::harness::{Region, at, capture, restore};
use crate::sound::turn_parts::{
    CLOCK, OBJECT_LEN, Part, SCRATCH, both, dispatch_turn, test_object,
};
use rc2k_core::sound::engine::{Left, Words, extrapolation, over_256, readings, state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("sound::car_engine_state", rva(0x40D8E2), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// off_514F90: the sound kinds, 36 pointers.
pub const KINDS: usize = 0x514F90;
pub const KIND_COUNT: u32 = 36;
pub const KIND_LEN: usize = 0x88;
/// dword_5BEC94: 1 holds the revs as they are.
const HOLD: usize = 0x5BEC94;

fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

/// The kind of object `o` (its +4), or a refusal.
pub fn kind_of(o: u32) -> Result<u32, BadInput> {
    let n = unsafe { ((o + 4) as *const u32).read_unaligned() };
    if n >= KIND_COUNT {
        return Err(BadInput("a sound kind past the 36 of the table"));
    }
    Ok(g(KINDS + 4 * n as usize))
}

// ---- Rust version ----

/// The object's dwords where the original reads and writes them.
struct Object(u32);

impl Words for Object {
    fn get(&self, at: u32) -> u32 {
        unsafe { ((self.0 + at) as *const u32).read_unaligned() }
    }
    fn set(&mut self, at: u32, value: u32) {
        unsafe { ((self.0 + at) as *mut u32).write_unaligned(value) }
    }
}

pub fn car_engine_state_in(c: &mut Regs, _: Part) -> Result<(), Refused> {
    // Converted: `rc2k_core::sound::engine` (`extrapolation`, `readings`, `state`,
    // `over_256`); the floats are the original's x87 still.
    let o = c.ebx;
    let kind = kind_of(o)?;
    let mut object = Object(o);
    let clock = g(CLOCK);
    let (b, bx, di) = (base(), o, kind);
    // The revs extrapolated, worked out first: its `idiv` may trap.
    let revs = extrapolation(&object, clock, || g(HOLD)).map_err(BadInput::from)?;
    c.edi = kind;
    let mut left = Left {
        eax: c.eax,
        ecx: c.ecx,
        edx: c.edx,
    };
    readings(&mut object, clock, revs, || g(HOLD), &mut left);
    x87_run!(
        "fild dword ptr [{bx} + 0x14]",
        "fsub dword ptr [{di} + 0x4]",
        "fmul dword ptr [{di} + 0xC]",
        "fstp dword ptr [{bx} + 0x100]";
        bx = bx, di = di
    );
    object.set(0x104, over_256(from_int(object.get(0x18) as i32)));
    x87_run!(
        "fld dword ptr [{bx} + 0x1C]",
        ".byte 0xD9, 0xE1",
        "fmul dword ptr [{b} + 0x116370]",
        "fstp dword ptr [{bx} + 0x108]";
        bx = bx, b = b
    );
    let flags = || unsafe { (kind as *const u32).read_unaligned() };
    if let Some(low) = state(&mut object, flags, clock, &mut left) {
        unsafe { game_write(rva(SCRATCH), low) };
        x87_run!(
            "fild dword ptr [{b} + 0x1183CC]",
            "mov dword ptr [{b} + 0x1183CC], 0x3C888889",
            "fmul dword ptr [{b} + 0x1183CC]",
            "mov dword ptr [{b} + 0x1183CC], 0x3F400000",
            "fadd dword ptr [{b} + 0x1183CC]",
            "fstp dword ptr [{bx} + 0x210]";
            b = b, bx = bx
        );
    }
    (c.ecx, c.edx) = (left.ecx, left.edx);
    x87_run!(
        "fld dword ptr [{bx} + 0x134]",
        "fld dword ptr [{bx} + 0x2C]",
        "fmul dword ptr [{bx} + 0x2C]",
        "fld dword ptr [{bx} + 0x30]",
        "fmul dword ptr [{bx} + 0x30]",
        "fld dword ptr [{bx} + 0x34]",
        "fmul dword ptr [{bx} + 0x34]",
        ".byte 0xDE, 0xC1",
        ".byte 0xDE, 0xC1",
        ".byte 0xD9, 0xFA",
        "fst dword ptr [{bx} + 0x134]",
        ".byte 0xDE, 0xE9",
        "fmul dword ptr [{b} + 0x1124EC]",
        "fstp dword ptr [{bx} + 0x138]";
        bx = bx, b = b
    );
    object.set(0x140, 0);
    x87_run!(
        "fld dword ptr [{bx} + 0x138]",
        "fadd dword ptr [{b} + 0x1124F8]",
        "fmul dword ptr [{b} + 0x1124FC]",
        "fsub dword ptr [{bx} + 0x13C]",
        "fmul dword ptr [{b} + 0x116360]",
        "fadd dword ptr [{bx} + 0x13C]",
        "fstp dword ptr [{bx} + 0x13C]";
        bx = bx, b = b
    );
    object.set(0x140, 0x3F80_0000);
    c.eax = object.get(0x134);
    if c.eax as i32 > 0x3F80_0000 {
        x87_run!(
            ".byte 0xD9, 0xE8",
            "fdiv dword ptr [{bx} + 0x134]",
            "fstp dword ptr [{bx} + 0x140]";
            bx = bx
        );
    }
    Ok(())
}

// ---- hook ----

fn regions(r: &Regs) -> Vec<Region> {
    vec![(r.ebx, OBJECT_LEN), at(rva(SCRATCH))]
}

extern "C" fn dispatch(r: &mut Regs) {
    if r.ebx == 0 {
        dispatch_turn(&HOOK, r, call_original, refuse_null, None);
        return;
    }
    dispatch_turn(&HOOK, r, call_original, car_engine_state_in, Some(&regions));
}

fn refuse_null(_: &mut Regs, _: Part) -> Result<(), Refused> {
    Err(BadInput("a null sound object").into())
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut kind = vec![0u32; KIND_LEN / 4];
    let k = kind.as_mut_ptr() as u32;
    let mut object = test_object(0x77, &[]);
    let o = object.as_mut_ptr() as u32;
    let a = |x: usize| game_addr(rva(x)) as u32;
    let all = [
        (o, OBJECT_LEN),
        (k, KIND_LEN),
        (a(KINDS), 4),
        at(rva(SCRATCH)),
        at(rva(CLOCK)),
        at(rva(HOLD)),
    ];
    let saved = capture(&all);
    let mut seed = 0x1357_9BDFu32;
    let mut pick = |choices: &[u32]| {
        seed = seed.wrapping_mul(0x41C6_4E6D).wrapping_add(0x3039);
        choices[(seed >> 16) as usize % choices.len()]
    };
    for case in 0..600u32 {
        let fields: Vec<(u32, u32)> = vec![
            (0x04, pick(&[0, 0, 0, 3, 35, 36])),
            (0x10, pick(&[0, 1])),
            (0x14, pick(&[0, 500, 1000, 1001, 3999, 4000, 9000])),
            (0x18, pick(&[0, 0x10, 0x80, 0xFF, 0x100, 0xFFFF_FF00])),
            (0x118, pick(&[0, 0x10, 0x80, 0xF0, 0xF1, 0x200])),
            (0x94, pick(&[0, 100, 1000])),
            (0x98, pick(&[0, 100, 120, 3000, 0x8000_0000])),
            (0x9C, pick(&[0, 2000, 0x7FFF_FFFF])),
            (0xA0, pick(&[0, 2500, 0x8000_0000])),
            (0x20, pick(&[0, 0, 1])),
            (0x128, pick(&[0, 1, 2])),
            (0x130, pick(&[0, 1, 2])),
            (0xFC, pick(&[0, 1, 2])),
            (0xE8, pick(&[0, 1, 2, 3])),
            (0xEC, pick(&[0, 1, 2, 3])),
            (0xF4, pick(&[0, 1, 9])),
            (0xF8, pick(&[0, 1])),
            (0x1F4, pick(&[0, 1, 0xFFFF_FFFF, 5])),
            (0x1EC, pick(&[0, 0x1C, 0x20])),
            (0x2C, pick(&[0, 0x3F00_0000, 0x4120_0000, 0xC2C8_0000])),
            (0x134, pick(&[0, 0x3F80_0000, 0x4120_0000])),
        ];
        let bits = pick(&[0, 0x1000, 0x2000, 0x3123, 0x1FFF]);
        let (clock, hold) = (pick(&[0, 105, 3333, 0x8000_0005]), pick(&[0, 1]));
        let fill = || unsafe {
            let p = o as *mut u32;
            for (n, v) in test_object(case, &[]).into_iter().enumerate() {
                p.add(n).write(v);
            }
            for &(at, v) in &fields {
                p.add(at as usize / 4).write(v);
            }
            let q = k as *mut u32;
            for n in 0..KIND_LEN / 4 {
                q.add(n).write(0x3E00_0000 + n as u32 * 0x11_1111);
            }
            q.write(bits);
            game_write(rva(KINDS), k);
            game_write(rva(CLOCK), clock);
            game_write(rva(HOLD), hold);
        };
        let refuses = fields[0].1 >= KIND_COUNT || {
            // The extrapolation's quotient, as the Rust version works it out.
            let w = |at: u32| fields.iter().find(|f| f.0 == at).unwrap().1;
            let gap = w(0x98).wrapping_sub(w(0x94));
            let gap = if gap as i32 > 1000 { 1000 } else { gap };
            w(0x10) != 1 && hold != 1 && gap != 0 && {
                let p = clock.wrapping_sub(w(0x98)) as i32 as i64
                    * w(0xA0).wrapping_sub(w(0x9C)) as i32 as i64;
                let q = p / gap as i32 as i64;
                q != q as i32 as i64
            }
        };
        both(
            &mut t,
            &format!(
                "case {case}: kind bits {bits:#x}, clock {clock:#x}, hold {hold}, fields {fields:x?}"
            ),
            &|r| unsafe { call_original(r) },
            Regs {
                ebx: o,
                ..Regs::SENTINEL
            },
            &fill,
            &all,
            car_engine_state_in,
            refuses,
        );
    }
    restore(&all, &saved);
    t.finish()
}
