//! `sound::car_sounds` (0x40D893): one car's sounds for the turn.
//!
//! In:  eax the car's number (`g_raceSoundObjects[eax]`)
//! Out: every register preserved (`pusha` / `popa`)
//! ebx the object, esi the request (`unk_5124BC`); `sound::car_engine_state`, then the
//! request's +0x20 and +0x24 (the pitch and volume factors) = the object's +0x13C and
//! +0x140 (eax left at +0x140), then `sound::revs_sound`, `sound::throttle_sound`,
//! `sound::speed_sound`, `sound::surface_sounds`, `sound::car_slot_sounds`,
//! `sound::car_hit_sound`, `sound::car_cue_sound`, `sound::car_gear_sounds`, each
//! getting the registers the one before left (the engine state leaves edi the car's
//! sound kind, which the next three read). Three calls, from `sound::car_sounds_all`.
//! Refuses (before anything) a number over 7 and a null object; what the callees refuse
//! puts back what was written (sounds already sent stay sent).
//! Shadow mode runs the original alone (it sends sounds).
//! Self-test: a copy whose nine calls go to stand-ins, against the Rust version.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::physics::flow::guard;
use crate::service::harness::{Region, capture, restore};
use crate::sound::turn_parts::{
    OBJECT_LEN, OBJECTS, Part, REQUEST, both, common_regions, copy_of, dispatch_turn, run_copy,
    test_object, turn_regions,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("sound::car_sounds", rva(0x40D893), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const LEN: usize = 0x4F;
const OUTS: [usize; 9] = [0x0E, 0x25, 0x2A, 0x2F, 0x34, 0x39, 0x3E, 0x43, 0x48];
const ENTRY: [u8; 8] = [0x60, 0x8B, 0x1C, 0x85, 0x10, 0x2A, 0x51, 0x00];
const ENGINE: u32 = 0x40D8E2;
/// After the factors, in the original's order.
const SOUNDS: [u32; 8] = [
    0x40DD19, 0x40DD96, 0x40DE09, 0x40DE9B, 0x40E0F6, 0x40E182, 0x40E1E5, 0x40E24F,
];

// ---- Rust version ----

pub fn car_sounds_in(r: &mut Regs, part: Part) -> Result<(), Refused> {
    if r.eax > 7 {
        return Err(BadInput("a car's sound object past 7").into());
    }
    let object: u32 = unsafe { game_read(rva(OBJECTS + 4 * r.eax as usize)) };
    if object == 0 {
        return Err(BadInput("a null sound object").into());
    }
    guard(&turn_regions(), || {
        let mut c = Regs {
            ebx: object,
            esi: game_addr(rva(REQUEST)) as u32,
            ..*r
        };
        part(ENGINE, &mut c)?;
        unsafe {
            let (o, q) = (c.ebx as *const u32, c.esi as *mut u32);
            let pitch = o.add(0x13C / 4).read_unaligned();
            q.add(8).write_unaligned(pitch);
            c.eax = o.add(0x140 / 4).read_unaligned();
            q.add(9).write_unaligned(c.eax);
        }
        for place in SOUNDS {
            part(place, &mut c)?;
        }
        Ok(())
    })
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_turn(&HOOK, r, call_original, car_sounds_in, None);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut places = vec![ENGINE];
    places.extend(SOUNDS);
    let copy = match copy_of(0x40D893, LEN, &ENTRY, &OUTS, &places) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let run = run_copy(&copy);
    let mut object = test_object(0x1234, &[]);
    let o = object.as_mut_ptr() as u32;
    let table = (game_addr(rva(OBJECTS)) as u32, 0x20);
    let mut all: Vec<Region> = vec![(o, OBJECT_LEN), table];
    all.extend(common_regions());
    let saved = capture(&all);
    for which in [0u32, 5, 7, 8] {
        let fill = || unsafe {
            for n in 0..8 {
                game_write(rva(OBJECTS + 4 * n), o);
            }
        };
        both(
            &mut t,
            &format!("car {which}"),
            &run,
            Regs {
                eax: which,
                ..Regs::SENTINEL
            },
            &fill,
            &all,
            car_sounds_in,
            which > 7,
        );
    }
    restore(&all, &saved);
    t.finish()
}
