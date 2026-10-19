//! `input::end_force` (0x4095AE): a device's force feedback ended, when its controller has
//! it (the controller's entry, `g_controllers` + 0x50 its number, +0x1C 1): the
//! self-centring turned on (`DIPROP_AUTOCENTER` 1 in `dword_4F6AA4`; its result not looked
//! at) and the six effects' objects given back (`input::release_effect`), each effect's
//! +0xC (found) 0. (+0x28, effects on, is left: `input::release_device` clears it.)
//!
//! In:  esi = the device's record (+4 its DirectInput device)
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `input::release_device`. The controller's number is not checked (as the
//! original's). Shadow mode runs the original alone (it calls DirectInput). Self-test: the
//! original and the Rust version on DirectInput objects made in Rust (`input::di_harness`)
//! and a copy of the first device's record whose effects were made (`input::begin_force`):
//! a controller with no force feedback, the wheel.

use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva, shadow};
use crate::input::devices::{RECORD_LEN, RECORDS, at, put};
use crate::input::di_harness::{
    EFFECT_LEN, EFFECTS_AT, compare, entry_of, method, regs_text, shared_regions, slot,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("input::end_force", rva(0x4095AE), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version (see the header).
pub fn end_force(record: u32) {
    if at(entry_of(record), 0x1C) != 1 {
        return;
    }
    let prop = crate::input::begin_force::dword_property(1);
    method(at(record, 4), slot::SET_PROPERTY, &[9, prop]);
    for n in 0..6 {
        let e = record.wrapping_add(EFFECTS_AT + n * EFFECT_LEN);
        crate::input::effect_objects::release_effect(e);
        put(e, 0xC, 0);
    }
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    end_force(r.esi);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let world = crate::input::di_objects::TestWorld::begin(
        crate::input::di_objects::test_devices(),
        Default::default(),
    );
    let start =
        unsafe { std::slice::from_raw_parts(game_addr(RECORDS[0]) as *const u8, RECORD_LEN) }
            .to_vec();
    let mut record = start.clone();
    let rec = record.as_mut_ptr() as u32;
    let mut regions = shared_regions();
    regions.push((rec, RECORD_LEN));
    for (device, force) in [(0usize, false), (1, true)] {
        let entry = Regs {
            esi: rec,
            ..Regs::SENTINEL
        };
        compare(
            &mut t,
            &format!("device {device}, force feedback {force}"),
            &world,
            &regions,
            &|| {
                crate::input::begin_force::test_controller(rec, &start, device, force, 50);
                let made = crate::input::begin_force::begin_force(
                    rec,
                    &mut crate::input::make_effect::game_part,
                );
                debug_assert!(made.is_ok());
            },
            &mut || {
                let mut r = entry;
                unsafe { call_original(&mut r) };
                regs_text(&r)
            },
            &mut || {
                end_force(rec);
                regs_text(&entry)
            },
        );
    }
    drop(world);
    drop(record);
    t.finish()
}
