//! `sound::surface_sound` (0x40E068): one of a car's sounds of what its wheels run on.
//!
//! In:  ebx the car's sound object, ebp the surface's sound set, esi the request
//! Out: eax the place's last dword (+0x34), ecx and edx the volume and rate
//!      (`sound::sample_settings` of the sample's own); everything else preserved
//! The set: +0 1 for a set of eight (+4 ..; the sound by the object's +0 & 3 and, 4
//! further for +0xC 1), else four (by +0 & 3). Its sound replaces the object's +0x144
//! (sets of eight) or +0x148; a changed one has the old one stopped first
//! (`sound::stop_sound`). Then the request: the sound, +4 the rate as a float, +8
//! `flt_516398` (`fld` / `fstp`), the place +0x2C .. +0x34, and
//! `sound::play_placed_sound` (a tail jump). Two callers, `sound::surface_sounds`.
//! Refuses (before anything) a null object, set or request, a set of eight with +0xC
//! over 1, and a sound past the sample records.
//! Shadow mode runs the original alone (it sends sounds).
//! Self-test: a copy whose calls go to stand-ins (the one of `sample_settings` left),
//! against the Rust version, on an object and sets of the test's.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::math::x87::from_int;
use crate::physics::flow::{base, x87_run};
use crate::service::harness::{Region, capture, restore};
use crate::sound::sample_settings::{OWN, record, sample_settings};
use crate::sound::turn_parts::{
    OBJECT_LEN, PLACED, Part, REQUEST, STOP, both, common_regions, copy_of, dispatch_turn,
    run_copy, test_object,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("sound::surface_sound", rva(0x40E068), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const LEN: usize = 0x8E;
/// Two calls of `stop_sound`, `sample_settings` (left), the jump to `play_placed_sound`.
const OUTS: [usize; 4] = [0x2B, 0x48, 0x5F, 0x89];
const ENTRY: [u8; 5] = [0x8B, 0x0B, 0x83, 0xE1, 0x03];

// ---- Rust version ----

pub fn surface_sound_in(c: &mut Regs, part: Part) -> Result<(), Refused> {
    if c.ebx == 0 || c.ebp == 0 || c.esi == 0 {
        return Err(BadInput("no object, set or request").into());
    }
    let o = c.ebx;
    let w = |at: u32| unsafe { ((o + at) as *const u32).read_unaligned() };
    let set_word = |at: u32| (o + at) as *mut u32;
    let set = c.ebp as *const u32;
    let q = c.esi as *mut u32;
    let eight = unsafe { set.read_unaligned() } == 1;
    c.ecx = w(0) & 3;
    if eight {
        c.edx = w(0xC);
        if c.edx > 1 {
            return Err(BadInput("a set of eight with +0xC over 1").into());
        }
        c.ecx += c.edx * 4;
    }
    c.eax = unsafe { set.add(1 + c.ecx as usize).read_unaligned() };
    record(c.eax)?;
    let slot = if eight { 0x144 } else { 0x148 };
    if c.eax != w(slot) {
        let old = w(slot);
        unsafe {
            set_word(slot).write_unaligned(c.eax);
            q.write_unaligned(old);
        }
        c.eax = old;
        part(STOP, c)?;
        c.eax = w(slot);
    }
    unsafe { q.write_unaligned(c.eax) };
    (c.ecx, c.edx) = sample_settings(c.eax, OWN, OWN)?;
    unsafe { q.add(1).write_unaligned(from_int(c.edx as i32)) };
    x87_run!(
        "fld dword ptr [{b} + 0x116398]",
        "fstp dword ptr [{si} + 0x8]";
        b = base(), si = c.esi
    );
    // `mov eax, [ebx+..]; mov [esi+..], eax`: eax is left at the last.
    for k in 0..3 {
        c.eax = w(0x2C + 4 * k as u32);
        unsafe { q.add(3 + k).write_unaligned(c.eax) };
    }
    part(PLACED, c)
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_turn(&HOOK, r, call_original, surface_sound_in, None);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy_of(0x40E068, LEN, &ENTRY, &OUTS, &[STOP, PLACED]) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let run = run_copy(&copy);
    let mut object = test_object(0x31, &[]);
    let o = object.as_mut_ptr() as u32;
    let mut sets = [0u32; 9];
    let s = sets.as_mut_ptr() as u32;
    let mut all: Vec<Region> = vec![(o, OBJECT_LEN), (s, 36)];
    all.extend(common_regions());
    let saved = capture(&all);
    for eight in [1u32, 0, 2] {
        for id in [0u32, 1, 2, 3, !0, !2] {
            for outside in [0u32, 1, 2] {
                for kept in [0u32, 0x14, 0x15, 0x1A] {
                    let fill = || unsafe {
                        let p = s as *mut u32;
                        p.write(eight);
                        for n in 1..9u32 {
                            p.add(n as usize).write(0x10 + n);
                        }
                        let q = o as *mut u32;
                        q.write(id);
                        q.add(3).write(outside);
                        q.add(0x144 / 4).write(kept);
                        q.add(0x148 / 4).write(kept ^ 1);
                    };
                    both(
                        &mut t,
                        &format!("set flag {eight}, id {id:#x}, +0xC {outside}, kept {kept:#x}"),
                        &run,
                        Regs {
                            ebx: o,
                            ebp: s,
                            esi: game_addr(rva(REQUEST)) as u32,
                            ..Regs::SENTINEL
                        },
                        &fill,
                        &all,
                        surface_sound_in,
                        eight == 1 && outside > 1,
                    );
                }
            }
        }
    }
    restore(&all, &saved);
    t.finish()
}
