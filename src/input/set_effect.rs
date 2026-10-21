//! `input::set_effect` (0x40961F): a request for one of a device's six force-feedback
//! effects: with the record's effects on (+0x28 1), and for a "set" (ebx 3) only while the
//! race runs (`dword_518090` 1), `ebx | (ecx & 0xFFF) << 8 | (edx & 0xFFF) << 20` into
//! the effect's +0x2C (effect eax at +0x3C + eax * 0xA4), where the input thread takes it
//! (`input::device_effects`).
//!
//! In:  eax = the effect (0 .. 5), ebx = what to do (2 start, 3 set, 4 stop; not masked),
//!      ecx, edx = two numbers (their low 12 bits), esi = the device's record
//! Out: every register kept (`pusha` .. `popa`)
//!
//! See `input/effects.rs`. 21 callers (`start_device_effects`, `stop_device_effects`,
//! `update_car_effects`). Refuses (before anything) an effect past 5 (the original writes
//! wherever eax * 0xA4 lands). Shadow mode runs the original alone (the input thread takes
//! the requests). Self-test: a copy whose `dword_518090` is the test's, on records of the
//! test's, against the Rust version: effects 0 .. 5, the four kinds and others, the
//! switches both ways, numbers past 12 bits.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, hook_asm, rva};
use crate::input::devices::{at, put, test_record};
use crate::input::effects::{EFFECT_LEN, EFFECTS, HAS_EFFECTS, Part, REQUEST, dispatch_effects};
use crate::system::api_stand_ins::copy_with;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("input::set_effect", rva(0x40961F), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version: `race_on` is `dword_518090`.
/// Converted: `rc2k_core::input::effects::wanted` and `request`.
pub fn set_effect(r: &Regs, race_on: u32) -> Result<(), Refused> {
    use rc2k_core::input::effects::{request, wanted};
    if wanted(at(r.esi, HAS_EFFECTS) == 1, race_on == 1, r.eax, r.ebx)? {
        put(
            r.esi,
            EFFECTS + EFFECT_LEN * r.eax + REQUEST,
            request(r.ebx, r.ecx, r.edx),
        );
    }
    Ok(())
}

fn in_game(r: &mut Regs, part: Part) -> Result<(), Refused> {
    part(crate::input::effects::SET_EFFECT, r)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_effects(&HOOK, r, call_original, in_game);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut on = 0u32;
    let on_at = &raw mut on as u32;
    // pusha; cmp dword ptr [esi+28h], 1; jnz
    let entry = [0x60, 0x83, 0x7E, 0x28, 0x01, 0x75, 0x31];
    let copy = match copy_with(
        0x40961F,
        0x3A,
        &entry,
        &[],
        &[],
        &[],
        &[(0x518090, 4, on_at)],
        1,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let mut case = 0u32;
    for effect in 0..6u32 {
        for what in [2u32, 3, 4, 0x1234_5603] {
            for (has, race) in [(1u32, 1u32), (1, 0), (0, 1), (2, 1)] {
                case += 1;
                let mut mine = test_record(case);
                let mut theirs = mine.clone();
                let r = Regs {
                    eax: effect,
                    ebx: what,
                    ecx: 0xFEDC_BA98u32.rotate_left(case),
                    edx: 0x0123_4567u32.rotate_right(case),
                    esi: mine.as_mut_ptr() as u32,
                    ..Regs::SENTINEL
                };
                put(r.esi, HAS_EFFECTS, has);
                let theirs_at = theirs.as_mut_ptr() as u32;
                put(theirs_at, HAS_EFFECTS, has);
                let ok = set_effect(&r, race).is_ok();
                unsafe { (on_at as *mut u32).write_volatile(race) };
                let mut o = Regs {
                    esi: theirs_at,
                    ..r
                };
                unsafe { call_game(&mut o, copy.addr()) };
                t.check(
                    ok && mine == theirs && o.preserved_except(&Regs { esi: theirs_at, ..r }, 0),
                    || {
                        format!(
                            "effect {effect}, what {what:#x}, +0x28 {has}, race {race}: the records differ at {:x?}, registers {:x?}",
                            mine.iter().zip(&theirs).position(|(a, b)| a != b),
                            crate::sound::turn_parts::regs7(&o)
                        )
                    },
                );
            }
        }
    }
    // An effect past 5 is refused.
    let mut record = test_record(99);
    let r = Regs {
        eax: 6,
        ebx: 2,
        esi: record.as_mut_ptr() as u32,
        ..Regs::SENTINEL
    };
    put(r.esi, HAS_EFFECTS, 1);
    t.check(set_effect(&r, 1).is_err(), || {
        "effect 6 was not refused".into()
    });
    t.finish()
}
