//! `physics::change_gear` (0x43C20D): the gear changed to the one asked for
//! (`dword_614C40`). Nothing when it is the gear already (eax the gear). From neutral:
//! the gear set and `physics::wheels_from_revs`. To neutral, or when `physics::gear_bit`
//! refuses the new gear (CF): the gear 0. Else `wheels_from_revs` (old gear), the gear
//! set, `physics::revs_from_wheels`. Every change sets `g_car` +0x238 and +0x23C
//! (`unk_71A210`, `unk_71A214`) to 1.
//!
//! In:  nothing
//! Out: eax the gear (or the new one; gear + 1 after a refusing `gear_bit`), ecx the new
//!      gear when `gear_bit` ran, ebx as the gearing calls leave it; the rest kept; no flags
//!      read (the caller returns)
//!
//! Caller: `sub_43C1E3` (its tail).
//! Refuses (before anything) a gear or a new gear outside -1 ..= 7 where the gearing reads
//! its ratio, and what `gear_bit` refuses.
//! Self-test: a copy of the original (its variables the test's; its calls to test copies
//! of `wheels_from_revs`, `revs_from_wheels` and `gear_bit` on the same) against the core:
//! gears and asked gears -1 .. 7, bit patterns, drive kinds, under three precisions.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    EAX, EBX, ECX, RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva,
};
use crate::math::x87_run::{TEST_CONTROL_WORDS, with_control_word};
use crate::physics::gear_bit::gear_bit_at;
use crate::physics::gearing::{GearPlaces, TestGearing, revs_from_wheels_at, wheels_from_revs_at};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::change_gear", rva(0x43C20D), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The other places: the asked gear, the gear bits, the two words set.
#[derive(Clone, Copy)]
pub struct ChangePlaces {
    pub wanted: u32,
    pub bits: u32,
    pub changed: u32,
    pub changed_2: u32,
}

impl ChangePlaces {
    pub fn game() -> Self {
        let g = |a: usize| game_addr(rva(a)) as u32;
        Self {
            wanted: g(0x614C40),
            bits: g(0x71A40C),
            changed: g(0x71A210),
            changed_2: g(0x71A214),
        }
    }
}

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

fn ratio_gear(g: u32) -> Result<(), Refused> {
    if (-1..=7).contains(&(g as i32)) {
        Ok(())
    } else {
        Err(BadInput("a gear outside g_gearRatios (-1 ..= 7)").into())
    }
}

/// The core.
pub unsafe fn change_gear_at(gp: &GearPlaces, c: &ChangePlaces) -> Result<Left, Refused> {
    unsafe {
        let gear = at(gp.gear);
        let w = at(c.wanted);
        if gear == w {
            return Ok(Left::nothing().with(EAX, gear));
        }
        let mark = || {
            put(c.changed, 1);
            put(c.changed_2, 1);
        };
        let neutral = |left: Left| {
            put(gp.gear, 0);
            mark();
            Ok(left)
        };
        if gear == 0 {
            ratio_gear(w)?;
            put(gp.gear, w);
            let l = wheels_from_revs_at(gp)?;
            mark();
            return Ok(l.with(EAX, w));
        }
        if w == 0 {
            return neutral(Left::nothing().with(EAX, gear));
        }
        let bit = gear_bit_at(c.bits, w)?;
        if bit.carry {
            return neutral(Left::nothing().with(EAX, bit.eax).with(ECX, w));
        }
        ratio_gear(gear)?;
        ratio_gear(w)?;
        wheels_from_revs_at(gp)?;
        put(gp.gear, w);
        revs_from_wheels_at(gp)?;
        mark();
        Ok(Left::nothing().with(EAX, w).with(ECX, w).with(EBX, w))
    }
}

/// The Rust version.
pub unsafe fn change_gear() -> Result<Left, Refused> {
    unsafe { change_gear_at(&GearPlaces::game(), &ChangePlaces::game()) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { change_gear() }
}

extern "C" fn dispatch(r: &mut Regs) {
    let gp = GearPlaces::game();
    let c = ChangePlaces::game();
    let mut regions: Vec<Region> = gp.regions();
    regions.extend([(gp.gear, 4), (c.changed, 8)]);
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let test = TestGearing::new();
    let p = test.places;
    let mut cells = vec![0u32; 4];
    let base = cells.as_mut_ptr() as u32;
    let c = ChangePlaces {
        wanted: base,
        bits: base + 4,
        changed: base + 8,
        changed_2: base + 12,
    };
    let make = |hook: &RawHook, address: usize, refs: usize| {
        crate::audio::test_copy::make(
            hook,
            &crate::audio::test_copy::Copy {
                address,
                len: 0x38,
                entry: &[0x83, 0x3D, 0xAC, 0xA1, 0x71, 0x00, 0x00],
                calls: &[],
                jumps_out: &[],
            },
            &|_| None,
            &test.variables(),
            refs,
        )
    };
    let copies = (
        make(&crate::physics::wheels_from_revs::HOOK, 0x43C124, 8),
        make(&crate::physics::revs_from_wheels::HOOK, 0x43C0C9, 7),
        crate::physics::gear_bit::test_copy(c.bits),
    );
    let (wheels, revs, bit) = match copies {
        (Ok(a), Ok(b), Ok(d)) => (a, b, d),
        (Err(why), _, _) | (_, Err(why), _) | (_, _, Err(why)) => return t.skip(&why),
    };
    let targets = (wheels.addr(), revs.addr(), bit.addr());
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x43C20D,
            len: 0x9B,
            entry: &[0xA1, 0xAC, 0xA1, 0x71, 0x00],
            calls: &[0x2B, 0x4B, 0x52, 0x61],
            jumps_out: &[],
        },
        &|r| {
            if r == rva(0x43C124) {
                Some(targets.0)
            } else if r == rva(0x43C0C9) {
                Some(targets.1)
            } else if r == rva(0x43C5B7) {
                Some(targets.2)
            } else {
                None
            }
        },
        &[
            (0x71A1AC, p.gear),
            (0x614C40, c.wanted),
            (0x71A210, c.changed),
            (0x71A214, c.changed_2),
        ],
        16,
    ) {
        Ok(x) => x,
        Err(why) => return t.skip(&why),
    };
    let mut regions = p.regions();
    regions.push((p.gear, 4));
    regions.push((base, 16));
    let gears = [u32::MAX, 0, 1, 2, 7];
    for word in TEST_CONTROL_WORDS {
        with_control_word(word, &mut || {
            for gear in gears {
                for w in gears {
                    for pattern in [0u32, 0x0AA, 0x1FF] {
                        let seed = gear.wrapping_mul(5) ^ w.wrapping_mul(3) ^ pattern;
                        let fill = || unsafe {
                            test.fill(gear, seed % 2, seed);
                            (base as *mut [u32; 4]).write([w, pattern, 0xAA, 0xBB]);
                        };
                        fill();
                        let mut ro = Regs::SENTINEL;
                        unsafe { call_game(&mut ro, copy.addr()) };
                        let theirs = capture(&regions);
                        fill();
                        let left = unsafe { change_gear_at(&p, &c) };
                        let mine = capture(&regions);
                        let regs_ok = left.as_ref().is_ok_and(|l| l.matches(&Regs::SENTINEL, &ro));
                        t.check(regs_ok && theirs == mine, || {
                            format!("control word {word:#x}, gear {gear:#x} -> {w:#x}, bits {pattern:#x}: regs_ok={regs_ok} (eax {:#x} ebx {:#x} ecx {:#x}) original {theirs:x?}; rust {mine:x?}", ro.eax, ro.ebx, ro.ecx)
                        });
                    }
                }
            }
        });
    }
    drop((copy, wheels, revs, bit, cells, test));
    t.finish()
}
