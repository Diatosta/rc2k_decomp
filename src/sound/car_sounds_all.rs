//! `sound::car_sounds_all` (0x40D7AB): the turn's sounds of every car: the players' cars
//! each, and of the computer's only the two nearest the listener.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` / `popa`)
//! Unless `dword_514F7C` (the objects in use) is 0: for each object whose +8 is 1, a
//! player's (+0 not negative) gets `sound::car_sounds` (eax its number) at once; of the
//! others the squared distance (+0x2C .. +0x34 squared and added, through `flt_514F88`)
//! is compared as an integer (signed, the bits of a float that is not negative) with
//! the nearest and the next (both starting at the bits of +inf), which keep their
//! numbers in eax and ebx (-1 for none). When the nearest is last turn's second
//! (`dword_514F84`) or the next last turn's nearest (`dword_514F80`), the two swap, so
//! that each keeps its voice; they are kept in the two dwords and each gets
//! `sound::car_sounds` with its +0 set to -2 (the nearest) or -1 for the call and put
//! back after. One caller, `sound::race_turn`.
//! Refuses (before anything) a count over 8 and a null object; what the callees refuse
//! puts back what was written (sounds already sent stay sent).
//! Shadow mode runs the original alone (it sends sounds).
//! Self-test: a copy whose calls go to a stand-in, the tables pointing at the test's
//! objects (put back), against the Rust version.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::physics::flow::{base, guard, x87_run};
use crate::service::harness::{Region, at, capture, restore};
use crate::sound::turn_parts::{
    COUNT, OBJECT_LEN, OBJECTS, Part, both, copy_of, dispatch_turn, f, run_copy, test_object,
    turn_regions,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("sound::car_sounds_all", rva(0x40D7AB), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const LEN: usize = 0xE8;
/// The three calls of `sound::car_sounds`.
const OUTS: [usize; 3] = [0x39, 0xC0, 0xDF];
const ENTRY: [u8; 8] = [0x60, 0x83, 0x3D, 0x7C, 0x4F, 0x51, 0x00, 0x00];
const CAR_SOUNDS: u32 = 0x40D893;
/// dword_514F80, dword_514F84: the nearest and the next of the computer's cars.
const NEAREST: usize = 0x514F80;
const NEXT: usize = 0x514F84;
/// flt_514F88: the squared distance.
const DISTANCE: usize = 0x514F88;

fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

fn word(o: u32, at: u32) -> u32 {
    unsafe { ((o + at) as *const u32).read_unaligned() }
}

// ---- Rust version ----

pub fn car_sounds_all_in(r: &mut Regs, part: Part) -> Result<(), Refused> {
    let count = g(COUNT);
    if count == 0 {
        return Ok(());
    }
    if count > 8 {
        return Err(BadInput("a sound object count over 8").into());
    }
    let object = |n: u32| g(OBJECTS + 4 * n as usize);
    if (0..count).any(|n| object(n) == 0) {
        return Err(BadInput("a null sound object").into());
    }
    let all = turn_regions();
    guard(&all, || {
        let mut c = *r;
        (c.eax, c.ebx, c.ecx, c.edx) = (u32::MAX, u32::MAX, 0x7F80_0000, 0x7F80_0000);
        for n in 0..count {
            c.ebp = n;
            c.esi = object(n);
            if word(c.esi, 8) != 1 {
                continue;
            }
            if word(c.esi, 0) as i32 >= 0 {
                let kept = c.eax;
                c.eax = n;
                part(CAR_SOUNDS, &mut c)?;
                c.eax = kept;
                continue;
            }
            x87_run!(
                "fld dword ptr [{si} + 0x2C]",
                "fmul dword ptr [{si} + 0x2C]",
                "fld dword ptr [{si} + 0x30]",
                "fmul dword ptr [{si} + 0x30]",
                "fld dword ptr [{si} + 0x34]",
                "fmul dword ptr [{si} + 0x34]",
                ".byte 0xDE, 0xC1",
                ".byte 0xDE, 0xC1",
                "fstp dword ptr [{b} + 0x114F88]";
                si = c.esi, b = base()
            );
            let d = g(DISTANCE);
            if c.ecx as i32 > d as i32 {
                (c.edx, c.ebx, c.ecx, c.eax) = (c.ecx, c.eax, d, n);
            } else if c.edx as i32 > d as i32 {
                (c.edx, c.ebx) = (d, n);
            }
        }
        c.ebp = count;
        if c.eax == g(NEXT) || c.ebx == g(NEAREST) {
            (c.eax, c.ebx) = (c.ebx, c.eax);
        }
        unsafe {
            game_write(rva(NEAREST), c.eax);
            game_write(rva(NEXT), c.ebx);
        }
        for (which, mark) in [(NEAREST, 0xFFFF_FFFEu32), (NEXT, 0xFFFF_FFFF)] {
            c.eax = g(which);
            if (c.eax as i32) < 0 {
                continue;
            }
            c.esi = object(c.eax);
            let kept = word(c.esi, 0);
            unsafe { (c.esi as *mut u32).write_unaligned(mark) };
            part(CAR_SOUNDS, &mut c)?;
            unsafe { (c.esi as *mut u32).write_unaligned(kept) };
        }
        Ok(())
    })
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_turn(&HOOK, r, call_original, car_sounds_all_in, None);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy_of(0x40D7AB, LEN, &ENTRY, &OUTS, &[CAR_SOUNDS]) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let run = run_copy(&copy);
    let mut objects: Vec<Vec<u32>> = (0..8).map(|k| test_object(k, &[])).collect();
    let o: Vec<u32> = objects.iter_mut().map(|v| v.as_mut_ptr() as u32).collect();
    let a = |x: usize| game_addr(rva(x)) as u32;
    let mut all: Vec<Region> = o.iter().map(|&p| (p, OBJECT_LEN)).collect();
    all.extend([(a(OBJECTS), 0x20), at(rva(COUNT)), (a(NEAREST), 12)]);
    let saved = capture(&all);
    // Each car: (+0, +8, distance along x).
    let layouts: [[(u32, u32, f32); 8]; 4] = [
        [
            (0, 1, 5.0),
            (!1, 1, 3.0),
            (!2, 1, 1.0),
            (!3, 1, 2.0),
            (4, 0, 0.0),
            (!5, 0, 0.5),
            (!6, 1, 9.0),
            (7, 1, 1.0),
        ],
        [
            (!0, 1, 1.0),
            (!1, 1, 1.0),
            (!2, 1, 1.0),
            (3, 1, 0.0),
            (!4, 1, 0.25),
            (!5, 1, -7.0),
            (!6, 1, 1e30),
            (!7, 1, 2.0),
        ],
        [(0, 1, 1.0); 8],
        [(!0, 1, f32::INFINITY); 8],
    ];
    for (k, layout) in layouts.iter().enumerate() {
        for count in [8u32, 3, 1, 0, 9] {
            for (nearest, next) in [(u32::MAX, u32::MAX), (2, 1), (1, 2), (3, 0)] {
                let fill = || unsafe {
                    for (n, &(id, on, x)) in layout.iter().enumerate() {
                        game_write(rva(OBJECTS + 4 * n), o[n]);
                        let p = o[n] as *mut u32;
                        p.write(id);
                        p.add(2).write(on);
                        p.add(11).write(f(x));
                        p.add(12).write(f(0.5));
                        p.add(13).write(f(-x));
                    }
                    game_write(rva(COUNT), count);
                    game_write(rva(NEAREST), nearest);
                    game_write(rva(NEXT), next);
                };
                both(
                    &mut t,
                    &format!("layout {k}, count {count}, last {nearest:#x} {next:#x}"),
                    &run,
                    Regs::SENTINEL,
                    &fill,
                    &all,
                    car_sounds_all_in,
                    count > 8,
                );
            }
        }
    }
    restore(&all, &saved);
    t.finish()
}
