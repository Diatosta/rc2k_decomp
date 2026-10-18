//! `sound::car_hit_sound` (0x40E182): a car's sound of a hit, sent once (+0x238, which
//! the physics' contacts set: `physics::place_sound_object` and the like).
//!
//! In:  ebx the car's sound object, esi the request
//! Out: every register preserved (`pusha` / `popa`)
//! Nothing when +0x238 is -2 (none); else it is taken (+0x238 = -2) into the
//! request, with its volume +0x23C and the sample's own rate through
//! `sound::sample_settings`; a volume under 4: nothing; else the request's +8 and +4 the volume and rate
//! as floats, the place +0x240 .. +0x248, and `sound::play_placed_sound`.
//! One caller, `sound::car_sounds`.
//! Refuses (before anything) a null object or request and a sound past the sample
//! records.
//! Shadow mode runs the original alone (it sends a sound).
//! Self-test: a copy whose calls go to stand-ins (the one of `sample_settings` left),
//! against the Rust version, on an object of the test's, real samples.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::math::x87::from_int;
use crate::service::harness::{Region, capture, restore};
use crate::sound::sample_settings::{OWN, record, sample_settings};
use crate::sound::turn_parts::{
    OBJECT_LEN, PLACED, Part, REQUEST, both, common_regions, copy_of, dispatch_turn, run_copy,
    test_object,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("sound::car_hit_sound", rva(0x40E182), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const LEN: usize = 0x63;
const OUTS: [usize; 2] = [0x23, 0x5C];
const ENTRY: [u8; 7] = [0x60, 0x8B, 0x83, 0x38, 0x02, 0x00, 0x00];
const NONE: u32 = 0xFFFF_FFFE;

// ---- Rust version ----

pub fn car_hit_sound_in(r: &mut Regs, part: Part) -> Result<(), Refused> {
    if r.ebx == 0 || r.esi == 0 {
        return Err(BadInput("no object or no request").into());
    }
    let o = r.ebx;
    let w = |at: u32| unsafe { ((o + at) as *const u32).read_unaligned() };
    let q = r.esi as *mut u32;
    let mut c = *r;
    c.eax = w(0x238);
    if c.eax == NONE {
        return Ok(());
    }
    record(c.eax)?;
    unsafe {
        ((o + 0x238) as *mut u32).write_unaligned(NONE);
        q.write_unaligned(c.eax);
    }
    (c.ecx, c.edx) = sample_settings(c.eax, w(0x23C), OWN)?;
    if c.ecx < 4 {
        return Ok(());
    }
    unsafe {
        q.add(2).write_unaligned(from_int(c.ecx as i32));
        q.add(1).write_unaligned(from_int(c.edx as i32));
        for k in 0..3 {
            c.eax = w(0x240 + 4 * k);
            q.add(3 + k as usize).write_unaligned(c.eax);
        }
    }
    part(PLACED, &mut c)
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_turn(&HOOK, r, call_original, car_hit_sound_in, None);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy_of(0x40E182, LEN, &ENTRY, &OUTS, &[PLACED]) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let run = run_copy(&copy);
    let mut object = test_object(0x238, &[]);
    let o = object.as_mut_ptr() as u32;
    let mut all: Vec<Region> = vec![(o, OBJECT_LEN)];
    all.extend(common_regions());
    let saved = capture(&all);
    for sound in [NONE, 0, 0xE1, 0xE5, 0x13F, 0x1CC, 0x1CD] {
        for volume in [OWN, 0, 1, 3, 4, 0x40, 0x41, 0xFFFF_FFFF] {
            let fill = || unsafe {
                let p = o as *mut u32;
                p.add(0x238 / 4).write(sound);
                p.add(0x23C / 4).write(volume);
            };
            both(
                &mut t,
                &format!("sound {sound:#x}, volume {volume:#x}"),
                &run,
                Regs {
                    ebx: o,
                    esi: game_addr(rva(REQUEST)) as u32,
                    ..Regs::SENTINEL
                },
                &fill,
                &all,
                car_hit_sound_in,
                sound != NONE && sound >= 0x1CD,
            );
        }
    }
    restore(&all, &saved);
    t.finish()
}
