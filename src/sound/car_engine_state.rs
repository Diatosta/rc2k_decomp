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
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::math::x87::from_int;
use crate::physics::flow::x87_run;
use crate::session::MACHINE;
use crate::sound::turn_parts::CLOCK;
use rc2k_core::sound::engine::{Left, Words, extrapolation, over_256, readings, state};

/// off_514F90: the sound kinds, 36 pointers.
pub const KINDS: usize = 0x514F90;
pub const KIND_COUNT: u32 = 36;
/// dword_5BEC94: 1 holds the revs as they are.
const HOLD: usize = 0x5BEC94;

fn g(x: impl crate::store::Ida) -> u32 {
    unsafe { crate::store::ida_read(x) }
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

/// The object `o` brought up to date; its sound kind returned (the original's edi).
pub fn car_engine_state(o: u32) -> Result<u32, Refused> {
    // Converted: `rc2k_core::sound::engine` (`extrapolation`, `readings`, `state`,
    // `over_256`); the floats are the original's x87 still.
    if o == 0 {
        return Err(BadInput("a null sound object").into());
    }
    let kind = kind_of(o)?;
    let mut object = Object(o);
    let clock = g(CLOCK);
    let (bx, di) = (o, kind);
    // The revs extrapolated, worked out first: its `idiv` may trap.
    let revs = extrapolation(&object, clock, || g(HOLD)).map_err(BadInput::from)?;
    // The registers the core models (written before they are read).
    let mut left = Left::default();
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
        "fmul dword ptr [{sf} + 0x18]",
        "fstp dword ptr [{bx} + 0x108]";
        bx = bx, sf = crate::sound::store::surface::base()
    );
    let flags = || unsafe { (kind as *const u32).read_unaligned() };
    if let Some(low) = state(&mut object, flags, clock, &mut left) {
        MACHINE.scratch.set(low);
        x87_run!(
            "fild dword ptr [{t}]",
            "mov dword ptr [{t}], 0x3C888889",
            "fmul dword ptr [{t}]",
            "mov dword ptr [{t}], 0x3F400000",
            "fadd dword ptr [{t}]",
            "fstp dword ptr [{bx} + 0x210]";
            bx = bx, t = crate::session::MACHINE.scratch.addr()
        );
    }
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
    "fmul dword ptr [{ck} + 8]",
    "fstp dword ptr [{bx} + 0x138]";
    bx = bx, ck = crate::sound::store::clock::base()
    );
    object.set(0x140, 0);
    x87_run!(
    "fld dword ptr [{bx} + 0x138]",
    "fadd dword ptr [{ck} + 0x14]",
    "fmul dword ptr [{ck} + 0x18]",
    "fsub dword ptr [{bx} + 0x13C]",
    "fmul dword ptr [{sf} + 0x8]",
    "fadd dword ptr [{bx} + 0x13C]",
    "fstp dword ptr [{bx} + 0x13C]";
        bx = bx, ck = crate::sound::store::clock::base(), sf = crate::sound::store::surface::base()
    );
    object.set(0x140, 0x3F80_0000);
    if object.get(0x134) as i32 > 0x3F80_0000 {
        x87_run!(
            ".byte 0xD9, 0xE8",
            "fdiv dword ptr [{bx} + 0x134]",
            "fstp dword ptr [{bx} + 0x140]";
            bx = bx
        );
    }
    Ok(kind)
}
