//! `sound::car_gear_sounds` (0x40E24F): a car's three timed sounds placed behind it: the
//! slots at +0x1C4 (armed by `sound::car_engine_state` when the throttle drops at high
//! revs), +0x1DC (armed by its countdown at low throttle) and +0x1F8 (armed with a gear
//! change; its pitch +0x210). Names by those readings.
//!
//! In:  ebx the car's sound object, esi the request
//! Out: every register preserved (`pusha` / `popa`)
//! Nothing in state 3 (+0x10C). The request's place = +0x2C .. +0x34 less +0x38 ..
//! +0x40; the slots at +0x1C4 and +0x1DC (`sound::play_timed_slot`); then, the
//! request's pitch factor (+0x20) times the object's +0x210 for it and put back after,
//! the slot at +0x1F8. One caller, `sound::car_sounds`.
//! Refuses (before anything) a null object or request; what the slots refuse puts back
//! what was written (sounds already sent stay sent).
//! Shadow mode runs the original alone (it sends sounds).
//! Self-test: a copy whose calls go to a stand-in, against the Rust version.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::physics::flow::{guard, x87_run};
use crate::service::harness::{Region, capture, restore};
use crate::sound::car_slot_sounds::SLOT;
use crate::sound::turn_parts::{
    OBJECT_LEN, Part, REQUEST, both, common_regions, copy_of, dispatch_turn, run_copy, test_object,
    turn_regions,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("sound::car_gear_sounds", rva(0x40E24F), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const LEN: usize = 0x5A;
const OUTS: [usize; 3] = [0x2B, 0x36, 0x50];
const ENTRY: [u8; 8] = [0x60, 0x83, 0xBB, 0x0C, 0x01, 0x00, 0x00, 0x03];

// ---- Rust version ----

pub fn car_gear_sounds_in(r: &mut Regs, part: Part) -> Result<(), Refused> {
    if r.ebx == 0 || r.esi == 0 {
        return Err(BadInput("no object or no request").into());
    }
    let o = r.ebx;
    if unsafe { ((o + 0x10C) as *const u32).read_unaligned() } == 3 {
        return Ok(());
    }
    let mut all = turn_regions();
    all.push((o, OBJECT_LEN));
    guard(&all, || {
        let mut c = *r;
        x87_run!(
            "fld dword ptr [{bx} + 0x2C]",
            "fsub dword ptr [{bx} + 0x38]",
            "fstp dword ptr [{si} + 0xC]",
            "fld dword ptr [{bx} + 0x30]",
            "fsub dword ptr [{bx} + 0x3C]",
            "fstp dword ptr [{si} + 0x10]",
            "fld dword ptr [{bx} + 0x34]",
            "fsub dword ptr [{bx} + 0x40]",
            "fstp dword ptr [{si} + 0x14]";
            bx = c.ebx, si = c.esi
        );
        for at in [0x1C4, 0x1DC] {
            c.ebp = o + at;
            part(SLOT, &mut c)?;
        }
        let factor = (c.esi + 0x20) as *mut u32;
        let kept = unsafe { factor.read_unaligned() };
        x87_run!(
            "fld dword ptr [{si} + 0x20]",
            "fmul dword ptr [{bx} + 0x210]",
            "fstp dword ptr [{si} + 0x20]";
            bx = c.ebx, si = c.esi
        );
        c.ebp = o + 0x1F8;
        part(SLOT, &mut c)?;
        unsafe { factor.write_unaligned(kept) };
        Ok(())
    })
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_turn(&HOOK, r, call_original, car_gear_sounds_in, None);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy_of(0x40E24F, LEN, &ENTRY, &OUTS, &[SLOT]) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let run = run_copy(&copy);
    let mut object = test_object(0x61, &[]);
    let o = object.as_mut_ptr() as u32;
    let mut all: Vec<Region> = vec![(o, OBJECT_LEN)];
    all.extend(common_regions());
    let saved = capture(&all);
    for state in [0u32, 1, 2, 3] {
        for factor in [0x3F80_0000u32, 0x3F40_0000, 0xBF00_0000] {
            let fill = || unsafe {
                let p = o as *mut u32;
                p.add(0x10C / 4).write(state);
                p.add(0x210 / 4).write(0x3F60_0000);
                game_write(rva(REQUEST + 0x20), factor);
            };
            both(
                &mut t,
                &format!("state {state}, factor {factor:#x}"),
                &run,
                Regs {
                    ebx: o,
                    esi: game_addr(rva(REQUEST)) as u32,
                    ..Regs::SENTINEL
                },
                &fill,
                &all,
                car_gear_sounds_in,
                false,
            );
        }
    }
    restore(&all, &saved);
    t.finish()
}
