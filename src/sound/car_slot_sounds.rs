//! `sound::car_slot_sounds` (0x40E0F6): a player's car's timed sounds (the slots of
//! `sound::play_timed_slot` in its object).
//!
//! In:  ebx the car's sound object, esi the request
//! Out: every register preserved (`pusha` / `popa`)
//! Nothing for a computer's car (+0 negative). The request's place = +0x2C .. +0x34;
//! the slots at +0x164, +0x14C, +0x17C, +0x194 (only with +0xC 0), +0x260 and +0x1AC;
//! then the place = +0x214 .. +0x21C and the slot at +0x220. Each call gets the
//! registers the one before left. One caller, `sound::car_sounds`.
//! Refuses (before anything) a null object or request; what the slots refuse puts back
//! what was written (sounds already sent stay sent).
//! A near `jl` in its first five bytes: private copy (0x8C bytes, the seven calls
//! relocated), manual patch.
//! Shadow mode runs the original alone (it sends sounds).
//! Self-test: a copy whose calls go to a stand-in, against the Rust version.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::physics::flow::guard;
use crate::service::harness::{Region, capture, restore};
use crate::sound::turn_parts::{
    OBJECT_LEN, Part, REQUEST, both, common_regions, copy_of, dispatch_turn, run_copy, test_object,
    turn_regions,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("sound::car_slot_sounds", rva(0x40E0F6), thunk, &ORIGINAL)
    .private_copy(LEN)
    .relocate_calls(&OUTS)
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const LEN: usize = 0x8C;
const OUTS: [usize; 7] = [0x28, 0x33, 0x3E, 0x49, 0x54, 0x5F, 0x85];
const ENTRY: [u8; 10] = [0x60, 0x83, 0x3B, 0x00, 0x0F, 0x8C, 0x80, 0x00, 0x00, 0x00];
pub const SLOT: u32 = 0x40E59E;

// ---- Rust version ----

pub fn car_slot_sounds_in(r: &mut Regs, part: Part) -> Result<(), Refused> {
    if r.ebx == 0 || r.esi == 0 {
        return Err(BadInput("no object or no request").into());
    }
    let o = r.ebx;
    let w = |at: u32| unsafe { ((o + at) as *const u32).read_unaligned() };
    if (w(0) as i32) < 0 {
        return Ok(());
    }
    let q = r.esi as *mut u32;
    let mut all = turn_regions();
    all.push((o, OBJECT_LEN));
    guard(&all, || {
        let mut c = *r;
        let place = |c: &mut Regs, from: u32| {
            for k in 0..3 {
                c.eax = w(from + 4 * k);
                unsafe { q.add(3 + k as usize).write_unaligned(c.eax) };
            }
        };
        place(&mut c, 0x2C);
        let mut slots = Vec::new();
        if w(0xC) == 0 {
            slots.extend([0x164, 0x14C, 0x17C, 0x194]);
        }
        slots.extend([0x260, 0x1AC]);
        for at in slots {
            c.ebp = o + at;
            part(SLOT, &mut c)?;
        }
        place(&mut c, 0x214);
        c.ebp = o + 0x220;
        part(SLOT, &mut c)
    })
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_turn(&HOOK, r, call_original, car_slot_sounds_in, None);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy_of(0x40E0F6, LEN, &ENTRY, &OUTS, &[SLOT]) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let run = run_copy(&copy);
    let mut object = test_object(0x51, &[]);
    let o = object.as_mut_ptr() as u32;
    let mut all: Vec<Region> = vec![(o, OBJECT_LEN)];
    all.extend(common_regions());
    let saved = capture(&all);
    for id in [0u32, 3, !0] {
        for outside in [0u32, 1] {
            let fill = || unsafe {
                let p = o as *mut u32;
                p.write(id);
                p.add(3).write(outside);
            };
            both(
                &mut t,
                &format!("id {id:#x}, +0xC {outside}"),
                &run,
                Regs {
                    ebx: o,
                    esi: game_addr(rva(REQUEST)) as u32,
                    ..Regs::SENTINEL
                },
                &fill,
                &all,
                car_slot_sounds_in,
                false,
            );
        }
    }
    restore(&all, &saved);
    t.finish()
}
